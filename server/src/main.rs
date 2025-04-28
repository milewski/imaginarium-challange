use std::error::Error;

use futures_util::{SinkExt, StreamExt};
use futures_util::task::SpawnExt;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio_tungstenite::accept_async;

use manager::{Manager, ScopedManager};
use shared::{Coordinate, Monument, PlayerData, PlayerId, SystemMessages};
use world::World;

use crate::api::build_server;
use crate::comfyui::ComfyUI;

mod vector;
mod comfyui;
mod api;
mod world;
mod manager;

type Sender = mpsc::UnboundedSender<SystemMessages>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("0.0.0.0:9001").await?;
    let manager = Manager::new();
    let world = World::default();
    // let engine = Engine::new().await?;

    if let Err(error) = world.restore_monuments_from_cache().await {
        println!("failed to restore monuments {:?}", error)
    };

    let world_clone = world.clone();

    tokio::spawn(build_server(manager.clone(), world.clone()));

    println!("websocket server starting at {}", "http://0.0.0.0:9001");

    while let Ok((stream, _)) = listener.accept().await {
        let Ok(websocket) = accept_async(stream).await else {
            println!("failed to accept stream connection...");
            continue
        };

        let (mut websocket_writer, mut websocket_reader) = websocket.split();

        let (sender, mut receiver) = mpsc::unbounded_channel();
        let player_id = PlayerId::random();

        let player_data = PlayerData {
            id: player_id,
            balance: 0,
            position: Coordinate::default(),
        };

        world_clone.add(player_data).await;
        manager.add(player_id, sender.clone()).await;

        println!("player {:?} connected", player_id);

        // Task: send to client
        let write_task = tokio::spawn(async move {
            while let Some(message) = receiver.recv().await {
                if websocket_writer.send(message.try_into().unwrap()).await.is_err() {
                    break;
                }
            }
        });

        // Task: receive from client
        let scoped = ScopedManager::new(player_id, manager.clone());
        let scoped_clone = scoped.clone();
        let world_clone_2 = world_clone.clone();

        let read_task = tokio::spawn(async move {
            while let Some(Ok(message)) = websocket_reader.next().await {
                if let Ok(message) = SystemMessages::try_from(message) {
                    println!("{:?} -> {:?}", player_id, message);
                    handle_player_communication(scoped_clone.clone(), world_clone_2.clone(), message).await;
                }
            }

            println!("player {:?} disconnected", player_id);

            scoped_clone.remove(player_id).await;
            world_clone_2.remove(player_id).await;
            scoped_clone.broadcast_except_self(SystemMessages::EnemyDisconnected { id: player_id }).await;
        });

        on_player_connect(scoped.clone(), world.clone()).await;

        // Auto-cleanup when either task ends
        tokio::spawn(async move {
            let _ = tokio::join!(write_task, read_task);
        });
    }

    Ok(())
}

async fn handle_player_communication(scope: ScopedManager, world: World, message: SystemMessages) {
    match message {
        SystemMessages::Ping => scope.broadcast_to_self(SystemMessages::Pong).await,
        SystemMessages::Pong => {}
        SystemMessages::Connected { .. } => {}
        SystemMessages::Welcome { .. } => {}
        SystemMessages::PlayerPosition { coordinate } => {
            world.update_coordinate(scope.id, coordinate).await;
            scope.broadcast_except_self(SystemMessages::EnemyPosition { id: scope.id, coordinate }).await
        }
        SystemMessages::MainPlayerSpawn { .. } => {}
        SystemMessages::EnemyPlayerSpawn { .. } => {}
        SystemMessages::BuildMonumentRequest { prompt } => {
            if let Some(data) = world.get(scope.id).await {
                match ComfyUI::new().generate(prompt.as_str()).await {
                    Ok(id) => {
                        let monument = Monument {
                            id,
                            description: prompt.into(),
                            asset: "under-construction.png".into(),
                            position: data.position.drift_by(3),
                            under_construction: true,
                        };

                        let balance = world.decrement_balance_by(scope.id, 5).await;
                        world.add_monument(monument.clone()).await;

                        scope.broadcast_to_self(SystemMessages::MainPlayerCurrentBalance { balance }).await;
                        scope.broadcast_to_all(SystemMessages::BuildMonument { monument }).await;
                    }
                    Err(_) => {
                        // notify client that his generation failed...
                    }
                }
            }
        }
        SystemMessages::MainPlayerPickedUpToken => {
            let balance = world.increment_balance(scope.id).await;
            scope.broadcast_to_self(SystemMessages::MainPlayerCurrentBalance { balance }).await;
        }
        _ => {}
    }
}

async fn on_player_connect(scoped: ScopedManager, world: World) {
    if let Some(data) = world.get(scoped.id).await {

        // Spawn the main player
        let player = scoped.broadcast_to_self(SystemMessages::MainPlayerSpawn { data: data.clone() });

        // Then notify everyone that there is a new boss in town
        let enemy = scoped.broadcast_except_self(SystemMessages::EnemyPlayerSpawn { data: data.clone() });

        for data in world.players().await {
            if data.id != scoped.id {
                scoped.broadcast_to_self(SystemMessages::EnemyPlayerSpawn { data }).await;
            }
        }

        for monument in world.monuments().await {
            scoped.broadcast_to_self(SystemMessages::BuildMonument { monument }).await;
        }

        tokio::join!(player, enemy);
    }
}
