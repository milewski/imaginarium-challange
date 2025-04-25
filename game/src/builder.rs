use crate::robot::Player;
use crate::AssetsCache;
use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_sprite3d::{Sprite3dBuilder, Sprite3dParams};
use shared::{Monument, SystemMessages};
use crate::network::{SendWebSocketMessage, WebSocketMessageReceived};

pub struct BuilderPlugin;

impl Plugin for BuilderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, build_monument_system.run_if(should_run));
        app.add_systems(Update, sync_monument_system);
    }
}

fn should_run(mouse: Res<ButtonInput<MouseButton>>) -> bool {
    mouse.just_pressed(MouseButton::Right)
}

fn sync_monument_system(
    mut commands: Commands,
    assets: Res<AssetsCache>,
    mut sprite_params: Sprite3dParams,
    mut events: EventReader<WebSocketMessageReceived>,
    asset_server: Res<AssetServer>,
    mut queue: Local<HashMap<Monument, Handle<Image>>>,
) {
    for event in events.read() {
        if let SystemMessages::BuildMonument { coordinate } = event.0 {
            let handle: Handle<Image> = asset_server.load("funny-guy.png");
            let monument = Monument { position: coordinate };
            queue.entry(monument).or_insert(handle);
        }
    }

    let mut ready_to_spawn = Vec::new();

    for (monument, handle) in queue.iter_mut() {
        if asset_server.is_loaded(&mut *handle) {
            ready_to_spawn.push((monument.clone(), handle.clone()));
        }
    }

    for (monument, handle) in ready_to_spawn {
        spawn_monument(&mut commands, &assets, &mut sprite_params, &monument, handle.clone());
        queue.remove(&monument);
    }
}

fn build_monument_system(
    mut commands: Commands,
    assets: Res<AssetsCache>,
    mut sprite_params: Sprite3dParams,
    player: Query<&Transform, With<Player>>,
    mut event: EventWriter<SendWebSocketMessage>,
) {
    // spawn_monument(&mut commands, &assets, &mut sprite_params, player.single().translation);
    event.send(SendWebSocketMessage(SystemMessages::BuildMonument { coordinate: player.single().translation.into() }));
}

fn spawn_monument(
    commands: &mut Commands,
    assets: &Res<AssetsCache>,
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