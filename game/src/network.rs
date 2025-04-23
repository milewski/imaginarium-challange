use std::collections::VecDeque;
use std::io::Read;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use crate::player::{Player};
use bevy::app::{App, Plugin};
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use bevy::utils::{info, HashMap};
use bincode::{config, Encode};
use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use gloo_timers::future::TimeoutFuture;
use serde::{Deserialize, Serialize};
use shared::{Coordinate, PlayerData, PlayerId, SystemMessages};
use tokio::sync::{Mutex, TryLockError};
use tokio::task::spawn_local;
use tokio::time::sleep;
use tokio_tungstenite_wasm::{Message, WebSocketStream};
use wasm_timer::Delay;
use crate::fox_plugin::ROBOT_GLB_PATH;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Connected;

pub struct NetworkPlugin;

#[derive(Resource, Debug, Default, Clone)]
pub struct WebsocketResource {
    read_queue: Arc<Mutex<Vec<SystemMessages>>>,
    write_queue: Arc<Mutex<Vec<SystemMessages>>>,
}

impl WebsocketResource {
    pub fn read(&self) -> Option<SystemMessages> {
        self.read_queue
            .try_lock()
            .ok()
            .map(|mut queue| queue.pop())
            .flatten()
    }

    pub fn send(&self, payload: SystemMessages) {
        let queue_clone = self.write_queue.clone();

        wasm_bindgen_futures::spawn_local(async move {
            let mut queue = queue_clone.lock().await;
            queue.push(payload);
        });
    }
}

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (receive_websocket_message_system, send_player_position),
        );

        app.insert_resource(PlayerEntities::default());

        let queue = WebsocketResource::default();
        let queue_clone = queue.clone();

        app.world_mut().insert_resource(queue);

        wasm_bindgen_futures::spawn_local(async move {
            let url = url::Url::parse("ws://127.0.0.1:9001").unwrap();
            let mut stream = tokio_tungstenite_wasm::connect(url)
                .await
                .expect("Failed to connect");

            let (mut write, mut read) = stream.split();

            wasm_bindgen_futures::spawn_local(async move {
                loop {
                    {
                        let mut messages = queue_clone.write_queue.lock().await;

                        for payload in messages.drain(..) {
                            let encoded =
                                bincode::encode_to_vec(payload, config::standard()).unwrap();
                            write.send(Message::binary(encoded)).await.unwrap();
                        }
                    }

                    TimeoutFuture::new(10).await;
                }
            });

            wasm_bindgen_futures::spawn_local(async move {
                loop {
                    if let Some(Ok(message)) = read.next().await {
                        if let mut messages = queue_clone.read_queue.lock().await {
                            if message.is_binary() {
                                let decoded = bincode::decode_from_slice(
                                    message.into_data().as_ref(),
                                    config::standard(),
                                );

                                if let Ok((SystemMessages, _)) = decoded {
                                    messages.push(SystemMessages);
                                }
                            }
                        }
                    }

                    TimeoutFuture::new(10).await;
                }
            });
        });
    }
}

#[derive(Resource, Default)]
struct PlayerEntities(HashMap<PlayerId, (PlayerData, Entity)>);

fn receive_websocket_message_system(
    mut commands: Commands,
    mut messages: ResMut<WebsocketResource>,
    mut player_entities: ResMut<PlayerEntities>,
    asset_server: Res<AssetServer>,
) {
    if let Some(message) = messages.read() {
        match message {
            SystemMessages::Connected { .. } => {}
            SystemMessages::Welcome { data } => {
                commands.insert_resource(data);
            }
            SystemMessages::PlayerPosition { .. } => {}
            SystemMessages::PlayerSpawn { data } => {
                // let entity = commands
                //     .spawn((
                //         SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(ROBOT_GLB_PATH))),
                //         Transform {
                //             scale: Vec3::splat(0.5),
                //             translation: data.position.to_vec3(),
                //             ..default()
                //         },
                //         data.id,
                //         Player::default(),
                //     ))
                //     .id();
                //
                // player_entities.0.insert(data.id, (data, entity));
            }
        }

        info!("Received {:?}", message)
    }
}

// fn player_movement_system(
//     mut query: Query<(&PlayerId, &mut Transform), With<Player>>,
// ) {
//     for (id, mut transform) in query.iter_mut() {
//         // Move based on player id or other logic
//     }
// }

fn send_player_position(mut player: Query<&Player>, mut messages: ResMut<WebsocketResource>) {
    // let player = player.single();
    // let (x, y) = player.current_position;
    // let payload = SystemMessages::PlayerPosition {
    //     coordinate: Coordinate {
    //         x: x as i32,
    //         y: y as i32,
    //     },
    // };
    //
    // if let Err(error) = messages.send(payload) {
    //     info!("Received {:?}", error)
    // }
}
