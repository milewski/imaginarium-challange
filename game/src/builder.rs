use crate::js_bridge_plugin::{JSBridgeMessages, JsBridgeMessageReceived};
use crate::network::{SendWebSocketMessage, WebSocketMessageReceived};
use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_kira_audio::{Audio, AudioControl};
use bevy_sprite3d::{Sprite3dBuilder, Sprite3dParams};
use futures_util::SinkExt;
use shared::{Monument, SystemMessages};
use crate::sound_effects::AudioCache;

pub struct BuilderPlugin;

impl Plugin for BuilderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, build_monument_system);
        app.add_systems(Update, sync_monument_system);
        app.add_systems(Update, update_under_construction_monument_system);
        app.add_systems(Update, animate_monument_system);
    }
}

fn update_under_construction_monument_system(
    mut commands: Commands,
    mut sprite_params: Sprite3dParams,
    mut monuments: Query<(Entity, &mut Monument)>,
    mut events: EventReader<WebSocketMessageReceived>,
    asset_server: Res<AssetServer>,
    mut queue: Local<HashMap<Entity, Handle<Image>>>,
) {
    for event in events.read() {
        if let SystemMessages::MonumentCompleted { id, asset } = &event.0 {
            if let Some((entity, mut monument)) = monuments.iter_mut().find(|(_, monument)| &monument.id == id) {
                monument.asset = asset.to_string();
                monument.under_construction = false;
                queue.entry(entity).or_insert(asset_server.load(asset.to_string()));
            }
        }
    }

    let mut ready_to_spawn = Vec::new();

    for (entity, handle) in queue.iter() {
        if asset_server.is_loaded(&*handle) {
            ready_to_spawn.push((entity.clone(), handle.clone()));
        }
    }

    for (entity, image) in ready_to_spawn {
        queue.remove(&entity);
        commands.entity(entity).insert(
            Sprite3dBuilder {
                image,
                pixels_per_metre: 100.,
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                pivot: Some(Vec2::new(0.5, 0.0)),
                ..default()
            }.bundle(&mut sprite_params)
        );
    }
}

fn sync_monument_system(
    mut commands: Commands,
    mut sprite_params: Sprite3dParams,
    mut events: EventReader<WebSocketMessageReceived>,
    asset_server: Res<AssetServer>,
    mut queue: Local<HashMap<u32, (Monument, Handle<Image>)>>,
    audio_cache: Res<AudioCache>,
    audio: Res<Audio>,
) {
    for event in events.read() {
        if let SystemMessages::BuildMonument { monument } = &event.0 {
            queue.entry(monument.id).or_insert((monument.clone(), asset_server.load(&monument.asset)));
        }
    }

    let mut ready_to_spawn = Vec::new();

    for (id, (monument, handle)) in queue.iter_mut() {
        if asset_server.is_loaded(&mut *handle) {
            ready_to_spawn.push((monument.clone(), handle.clone()));
        }
    }

    for (monument, handle) in ready_to_spawn {
        queue.remove(&monument.id);
        spawn_monument(&mut commands, &mut sprite_params, monument, handle.clone());
        audio.play(audio_cache.falling.clone());
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
    monument: Monument,
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
            translation: monument.position.to_vec3().with_y(fastrand::u32(5..15) as f32),
            scale: Vec3::splat(1.3),
            rotation: Quat::from_rotation_y(45f32.to_radians()),
            ..default()
        },
        monument,
    ));

}

fn animate_monument_system(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Monument>>,
    mut shake_timer: Local<f32>,
) {
    let gravity = 30.0;
    let shake_duration = 0.5;
    let shake_magnitude = 0.2;

    for mut transform in query.iter_mut() {
        if *shake_timer > 0.0 {
            *shake_timer -= time.delta_secs();

            let progress = 1.0 - (*shake_timer / shake_duration);
            let shake = (progress * std::f32::consts::PI * 6.0).sin() * shake_magnitude * (1.0 - progress);

            transform.translation.y = shake.max(0.0);

            if *shake_timer <= 0.0 {
                transform.translation.y = 0.0;
            }
        } else if transform.translation.y > 0.0 {
            transform.translation.y -= gravity * time.delta_secs();

            if transform.translation.y <= 0.0 {
                transform.translation.y = 0.0;
                *shake_timer = shake_duration;
            }
        }
    }
}