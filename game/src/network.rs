use bevy::prelude::*;
use bincode::config::standard;
use futures_util::{SinkExt, StreamExt};
use gloo_timers::future::TimeoutFuture;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio_tungstenite_wasm::Message;

use shared::SystemMessages;

#[derive(Resource)]
pub struct WebSocketReceiver(pub UnboundedReceiver<SystemMessages>);

#[derive(Resource, Clone)]
pub struct WebSocketSender(pub UnboundedSender<SystemMessages>);

#[derive(Event, Debug, Clone)]
pub struct WebSocketMessageReceived(pub SystemMessages);

#[derive(Event, Debug)]
pub struct SendWebSocketMessage(pub SystemMessages);

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        let (downstream_sender, downstream_receiver) = unbounded_channel::<SystemMessages>();
        let (upstream_sender, mut upstream_receiver) = unbounded_channel::<SystemMessages>();

        app.add_systems(Update, debug_websocket_messages_system);
        app.add_systems(Update, send_ping_system);

        // Server to Client
        app.add_event::<WebSocketMessageReceived>();
        app.add_systems(Update, websocket_event_bridge_system);
        app.insert_resource(WebSocketReceiver(downstream_receiver));

        // Client to Server
        app.add_event::<SendWebSocketMessage>();
        app.insert_resource(WebSocketSender(upstream_sender.clone()));
        app.add_systems(Update, websocket_send_event_system);

        wasm_bindgen_futures::spawn_local(async move {
            let url = url::Url::parse("ws://127.0.0.1:9001").unwrap();
            let mut stream = tokio_tungstenite_wasm::connect(url)
                .await
                .expect("failed to connect");

            let (mut write, mut read) = stream.split();

            wasm_bindgen_futures::spawn_local(async move {
                while let Some(message) = upstream_receiver.recv().await {
                    if let Err(error) = write.send(message.into()).await {
                        error!("failed to send: {:?}", error);
                    }
                }
            });

            wasm_bindgen_futures::spawn_local(async move {
                while let Some(Ok(message)) = read.next().await {
                    if let Ok(message) = message.try_into() {
                        if let Err(error) = downstream_sender.send(message) {
                            error!("failed to send message {:?}", error)
                        }
                    }
                }
            });
        });
    }
}

/// Send a ping to the server whenever the pressing the letter P (for debugging purpose)
/// Server will reply with a pong
fn send_ping_system(keyboard: Res<ButtonInput<KeyCode>>, mut event: EventWriter<SendWebSocketMessage>) {
    if keyboard.just_pressed(KeyCode::KeyP) {
        event.send(SendWebSocketMessage(SystemMessages::Ping));
    }
}

fn debug_websocket_messages_system(mut events: EventReader<WebSocketMessageReceived>) {
    for event in events.read() {
        info!("received: {:?}", event.0);
    }
}

fn websocket_send_event_system(mut events: EventReader<SendWebSocketMessage>, sender: Res<WebSocketSender>) {
    for event in events.read() {
        if let Err(error) = sender.0.send(event.0.clone()) {
            error!("failed to send WebSocket message: {:?}", error);
        }
    }
}

fn websocket_event_bridge_system(mut receiver: ResMut<WebSocketReceiver>, mut writer: EventWriter<WebSocketMessageReceived>, ) {
    while let Ok(message) = receiver.0.try_recv() {
        writer.send(WebSocketMessageReceived(message));
    }
}
