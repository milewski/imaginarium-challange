use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::accept_async;

use shared::{Coordinate, PlayerData, PlayerId, SystemMessages};

type Sender = mpsc::UnboundedSender<SystemMessages>;

#[derive(Default, Clone)]
struct World {
    inner: Arc<Mutex<HashMap<PlayerId, PlayerData>>>,
}

impl World {
    pub async fn get(&self, id: PlayerId) -> Option<PlayerData> {
        self.inner.lock().await.get(&id).copied()
    }

    pub async fn players(&self) -> Vec<PlayerData> {
        self.inner.lock().await.values().copied().collect()
    }

    pub async fn add(&self, data: PlayerData) {
        self.inner.lock().await.insert(data.id, data);
    }

    pub async fn remove(&self, id: PlayerId) {
        self.inner.lock().await.remove(&id);
    }

    pub async fn update_coordinate(&self, id: PlayerId, coordinate: Coordinate) {
        if let Some(data) = self.inner.lock().await.get_mut(&id) {
            data.position = coordinate
        }
    }
}

#[derive(Default, Clone)]
struct Manager {
    inner: Arc<Mutex<HashMap<PlayerId, Sender>>>,
}

impl Manager {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn broadcast(&self, message: SystemMessages) {
        for sender in self.inner.lock().await.values() {
            let _ = sender.send(message.clone());
        }
    }

    async fn broadcast_except(&self, id: PlayerId, message: SystemMessages) {
        for (client_id, sender) in self.inner.lock().await.iter() {
            if *client_id != id {
                let _ = sender.send(message.clone());
            }
        }
    }

    async fn broadcast_to(&self, id: PlayerId, message: SystemMessages) {
        if let Some(sender) = self.inner.lock().await.get(&id) {
            let _ = sender.send(message);
        }
    }

    async fn add(&self, id: PlayerId, sender: Sender) {
        self.inner.lock().await.insert(id, sender);
    }

    async fn remove(&self, id: PlayerId) {
        self.inner.lock().await.remove(&id);
    }
}

#[derive(Clone)]
struct ScopedManager {
    id: PlayerId,
    inner: Manager,
}

impl ScopedManager {
    pub fn new(id: PlayerId, parent: Manager) -> Self {
        Self { id, inner: parent }
    }

    pub async fn reply(&self, message: SystemMessages) {
        self.inner.broadcast_to(self.id, message).await
    }

    pub async fn broadcast(&self, message: SystemMessages) {
        self.inner.broadcast_except(self.id, message).await
    }

    async fn remove(&self, id: PlayerId) {
        self.inner.remove(id).await
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:9001").await?;
    let manager = Manager::new();
    let world = World::default();

    let world_clone = world.clone();

    while let Ok((stream, _)) = listener.accept().await {
        let websocket = accept_async(stream).await?;
        let (mut websocket_writer, mut websocket_reader) = websocket.split();

        let (sender, mut receiver) = mpsc::unbounded_channel();
        let player_id = PlayerId::random();

        let player_data = PlayerData {
            id: player_id,
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
            scoped_clone.broadcast(SystemMessages::EnemyDisconnected { id: player_id }).await;
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
        SystemMessages::Ping => scope.reply(SystemMessages::Pong).await,
        SystemMessages::Pong => {}
        SystemMessages::Connected { .. } => {}
        SystemMessages::Welcome { .. } => {}
        SystemMessages::PlayerPosition { coordinate } => {
            world.update_coordinate(scope.id, coordinate).await;
            scope.broadcast(SystemMessages::EnemyPosition { id: scope.id, coordinate }).await
        }
        SystemMessages::MainPlayerSpawn { .. } => {}
        SystemMessages::EnemyPlayerSpawn { .. } => {}
        _ => {}
    }
}

async fn on_player_connect(scoped: ScopedManager, world: World) {
    if let Some(data) = world.get(scoped.id).await {

        // Spawn the main player
        let player = scoped.reply(SystemMessages::MainPlayerSpawn { data });

        // Then notify everyone that there is a new boss in town
        let enemy = scoped.broadcast(SystemMessages::EnemyPlayerSpawn { data });

        for data in world.players().await {
            if data.id != scoped.id {
                scoped.reply(SystemMessages::EnemyPlayerSpawn { data }).await;
            }
        }

        tokio::join!(player, enemy);
    }
}
