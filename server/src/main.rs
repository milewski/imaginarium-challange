use bincode::config;
use futures_util::{SinkExt, StreamExt};
use shared::{Coordinate, PlayerData, PlayerId, SystemMessages};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;

#[derive(Default)]
struct Cache {
    clients: HashMap<PlayerId, PlayerData>,
}

impl Cache {
    pub fn create_player(&mut self) -> PlayerId {
        let id = PlayerId::random();
        self.clients.insert(
            id,
            PlayerData {
                id,
                position: Coordinate::default(),
            },
        );
        id
    }

    pub fn get(&self, id: &PlayerId) -> Option<&PlayerData> {
        self.clients.get(id)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let try_socket = TcpListener::bind(&"127.0.0.1:9001").await;
    let listener = try_socket.expect("Failed to bind");
    let state = Arc::new(Mutex::new(Cache::default()));

    while let Ok((stream, _)) = listener.accept().await {
        let player_id = { state.lock().await.create_player() };

        tokio::spawn(accept_connection(stream, player_id, state.clone()));
    }

    Ok(())
}

async fn accept_connection(stream: TcpStream, player_id: PlayerId, state: Arc<Mutex<Cache>>) {
    let addr = stream
        .peer_addr()
        .expect("connected streams should have a peer address");
    println!("Peer address: {}", addr);

    let ws_stream = accept_async(stream)
        .await
        .expect("WebSocket handshake failed");
    println!("New WebSocket connection: {}", addr);

    let (mut write, mut read) = ws_stream.split();

    let welcome_message = {
        SystemMessages::Welcome {
            data: *state.lock().await.get(&player_id).unwrap(),
        }
    };

    let encoded = bincode::encode_to_vec(welcome_message, config::standard()).unwrap();
    let _ = write.send(Message::binary(encoded)).await;

    let player_spawn_message = {
        SystemMessages::PlayerSpawn {
            data: *state.lock().await.get(&player_id).unwrap(),
        }
    };

    let encoded = bincode::encode_to_vec(player_spawn_message, config::standard()).unwrap();
    let _ = write.send(Message::binary(encoded)).await;

    while let Some(Ok(message)) = read.next().await {
        if message.is_binary() {
            let config = config::standard();
            let (message, _) = bincode::decode_from_slice::<SystemMessages, _>(
                message.into_data().as_ref(),
                config,
            )
            .unwrap();

            match message {
                SystemMessages::Connected { id } => {
                    println!("player_connected: {:?}", id);
                    // state.lock().await.clients.insert(id, PlayerData::default());
                }
                SystemMessages::PlayerPosition { .. } => {}
                SystemMessages::PlayerSpawn { .. } => {}
                SystemMessages::Welcome { .. } => {}
            }
        }
    }

    println!("Connection closed: {}", addr);
}
