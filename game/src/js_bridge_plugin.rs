use crate::network::{SendWebSocketMessage, WebSocketSender};
use crate::robot::Player;
use bevy::app::{App, Plugin, Update};
use bevy::math::Vec2;
use bevy::prelude::*;
use bevy_sprite3d::{Sprite3dBuilder, Sprite3dParams};
use shared::SystemMessages;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen_futures::{spawn_local, JsFuture};

pub struct JsBridgePlugin;

#[wasm_bindgen]
extern "C" {
    pub fn show_modal() -> js_sys::Promise;
}

pub async fn call_show_modal() -> JSBridgeMessages {
    JSBridgeMessages::CallOpenModalResponse(
        JsFuture::from(show_modal()).await.ok().map(|value| value.as_string()).flatten()
    )
}

#[derive(Debug, Clone)]
pub enum JSBridgeMessages {
    CallOpenModal,
    CallOpenModalResponse(Option<String>),
    None,
}

#[derive(Resource)]
pub struct JsBridgeReceiver(pub UnboundedReceiver<JSBridgeMessages>);

#[derive(Resource, Clone)]
pub struct JsBridgeSender(pub UnboundedSender<JSBridgeMessages>);

#[derive(Event, Debug, Clone)]
pub struct JsBridgeMessageReceived(pub JSBridgeMessages);

#[derive(Event, Debug)]
pub struct SendJsBridgeMessage(pub JSBridgeMessages);

impl Plugin for JsBridgePlugin {
    fn build(&self, app: &mut App) {
        let (downstream_sender, downstream_receiver) = unbounded_channel::<JSBridgeMessages>();
        let (upstream_sender, mut upstream_receiver) = unbounded_channel::<JSBridgeMessages>();

        app.add_systems(Update, debug_js_bridge_messages_system);

        // Server to Client
        app.add_event::<JsBridgeMessageReceived>();
        app.add_systems(Update, js_bridge_event_bridge_system);
        app.insert_resource(JsBridgeReceiver(downstream_receiver));

        // Client to Server
        app.add_event::<SendJsBridgeMessage>();
        app.insert_resource(JsBridgeSender(upstream_sender.clone()));
        app.add_systems(Update, js_bridge_send_event_system);

        spawn_local(async move {
            while let Some(message) = upstream_receiver.recv().await {
                let downstream_sender = downstream_sender.clone();

                spawn_local(async move {
                    let response: JSBridgeMessages = match message {
                        JSBridgeMessages::CallOpenModal => call_show_modal().await,
                        _ => JSBridgeMessages::None
                    };

                    if let Err(error) = downstream_sender.send(response) {
                        error!("failed to send: {:?}", error);
                    }
                });
            }
        });

        app.add_systems(Update, button_system);
    }
}

fn js_event_bridge_system(mut receiver: ResMut<JsBridgeReceiver>, mut writer: EventWriter<JsBridgeMessageReceived>) {
    while let Ok(message) = receiver.0.try_recv() {
        writer.send(JsBridgeMessageReceived(message));
    }
}

fn button_system(
    mut event: EventWriter<SendJsBridgeMessage>,
    mouse: Res<ButtonInput<MouseButton>>,
) {
    if mouse.just_pressed(MouseButton::Right) {
        info!("sending");
        event.send(SendJsBridgeMessage(JSBridgeMessages::CallOpenModal));
    }
}

fn debug_js_bridge_messages_system(mut events: EventReader<JsBridgeMessageReceived>) {
    for event in events.read() {
        info!("received: {:?}", event.0);
    }
}

fn js_bridge_send_event_system(mut events: EventReader<SendJsBridgeMessage>, sender: Res<JsBridgeSender>) {
    for event in events.read() {
        if let Err(error) = sender.0.send(event.0.clone()) {
            error!("failed to send JS bridge message: {:?}", error);
        }
    }
}

fn js_bridge_event_bridge_system(mut receiver: ResMut<JsBridgeReceiver>, mut writer: EventWriter<JsBridgeMessageReceived>) {
    while let Ok(message) = receiver.0.try_recv() {
        writer.send(JsBridgeMessageReceived(message));
    }
}