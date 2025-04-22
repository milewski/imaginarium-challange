use bincode::config;
use futures_util::{SinkExt, StreamExt};
use shared::{Coordinate, SystemMessages};
use std::collections::HashMap;
use std::error::Error;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;

#[derive(Default)]
struct PlayerData {
    position: Coordinate,
}

#[derive(Default)]
struct Cache {
    clients: HashMap<u32, PlayerData>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let try_socket = TcpListener::bind(&"127.0.0.1:9001").await;
    let listener = try_socket.expect("Failed to bind");

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(accept_connection(stream));
    }

    Ok(())
}

async fn accept_connection(stream: TcpStream) {
    let addr = stream
        .peer_addr()
        .expect("connected streams should have a peer address");
    println!("Peer address: {}", addr);

    let ws_stream = accept_async(stream)
        .await
        .expect("WebSocket handshake failed");
    println!("New WebSocket connection: {}", addr);

    let (mut write, mut read) = ws_stream.split();

    let message = SystemMessages::PlayerPosition {
        coordinate: Coordinate::default(),
    };

    let encoded = bincode::encode_to_vec(message, config::standard()).unwrap();

    let _ = write.send(Message::binary(encoded)).await;

    while let Some(Ok(message)) = read.next().await {
        if message.is_binary() {
            let config = config::standard();
            let text: (SystemMessages, _) = bincode::decode_from_slice(message.into_data().as_ref(), config).unwrap();

            println!("received {:?}", text)
        }
    }

    println!("Connection closed: {}", addr);
}
