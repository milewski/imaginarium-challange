use crate::js_bridge_plugin::{JSBridgeMessages, JsBridgeMessageReceived};
use crate::network::{SendWebSocketMessage, WebSocketMessageReceived};
use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_sprite3d::{Sprite3dBuilder, Sprite3dParams};
use futures_util::SinkExt;
use shared::{Monument, SystemMessages};

pub struct BuilderPlugin;

impl Plugin for BuilderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, build_monument_system);
        app.add_systems(Update, sync_monument_system);
    }
}

fn sync_monument_system(
    mut commands: Commands,
    mut sprite_params: Sprite3dParams,
    mut events: EventReader<WebSocketMessageReceived>,
    asset_server: Res<AssetServer>,
    mut queue: Local<HashMap<Monument, Handle<Image>>>,
) {
    for event in events.read() {
        if let SystemMessages::BuildMonument { monument } = &event.0 {
            queue.entry(monument.clone()).or_insert(asset_server.load(&monument.asset));
        }
    }

    let mut ready_to_spawn = Vec::new();

    for (monument, handle) in queue.iter_mut() {
        if asset_server.is_loaded(&mut *handle) {
            ready_to_spawn.push((monument.clone(), handle.clone()));
        }
    }

    for (monument, handle) in ready_to_spawn {
        spawn_monument(&mut commands, &mut sprite_params, &monument, handle.clone());
        queue.remove(&monument);
    }
}

fn build_monument_system(
    mut websocket: EventWriter<SendWebSocketMessage>,
    mut js_bridge_events: EventReader<JsBridgeMessageReceived>,
) {
    for js_bridge_event in js_bridge_events.read() {
        if let JSBridgeMessages::CallOpenModalResponse(Some(prompt)) = &js_bridge_event.0 {
            websocket.send(SendWebSocketMessage(SystemMessages::BuildMonumentRequest { prompt: prompt.clone() }));
        }
    }
}

fn spawn_monument(
    commands: &mut Commands,
    sprite_params: &mut Sprite3dParams,
    monument: &Monument,
    image_handle: Handle<Image>,
) {
    commands.spawn((
        Sprite3dBuilder {
            image: image_handle,
            pixels_per_metre: 100.,
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            pivot: Some(Vec2::new(0.5, 0.0)),
            ..default()
        }.bundle(sprite_params),
        Transform {
            translation: monument.position.to_vec3(),
            scale: Vec3::splat(2.0),
            rotation: Quat::from_rotation_y(45f32.to_radians()),
            ..default()
        },
    ));
}