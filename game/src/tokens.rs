use crate::robot::Player;
use bevy::app::{App, Startup, Update};
use bevy::asset::{AssetServer, Handle};
use bevy::hierarchy::ChildBuild;
use bevy::image::Image;
use bevy::math::{Quat, Vec2, Vec3};
use bevy::prelude::{default, resource_exists, AlphaMode, Commands, Component, Entity, EventWriter, IntoSystemConfigs, Local, Plugin, Query, Res, Resource, Transform, With};
use bevy_sprite3d::{Sprite3dBuilder, Sprite3dParams};
use shared::SystemMessages;
use crate::network::SendWebSocketMessage;

pub struct TokensPlugin;

impl Plugin for TokensPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_token);
        app.add_systems(Update, setup.run_if(resource_exists::<TokenHandle>));
        app.add_systems(Update, pickup_token_system);
    }
}

#[derive(Resource)]
struct TokenHandle(Handle<Image>);

#[derive(Component)]
pub struct Token;

fn load_token(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(TokenHandle(asset_server.load("scrap_002.png")));
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    token_handle: Res<TokenHandle>,
    mut sprite_params: Sprite3dParams,
    mut run: Local<bool>,
) {
    if *run {
        return;
    }

    if asset_server.is_loaded(&token_handle.0) {
        *run = true;

        spawn_tokens_around_center(
            &mut commands,
            &mut sprite_params,
            &token_handle,
            Vec2::new(0.0, 0.0), // center
            10.0,                // inner radius (no-spawn zone)
            100.0,               // outer radius (max spawn distance)
            5.0,                 // minimum spacing between tokens
            100,                 // how many to spawn
        );
    }
}

fn pickup_token_system(
    mut commands: Commands,
    query: Query<(Entity, &Transform), With<Token>>,
    player_query: Query<&Transform, With<Player>>,
    mut event: EventWriter<SendWebSocketMessage>
) {
    for player_transform in player_query.iter() {
        for (token_entity, token_transform) in query.iter() {
            let distance = player_transform.translation.distance(token_transform.translation);
            let pickup_radius = 2.0;

            if distance < pickup_radius {
                commands.entity(token_entity).despawn();
                event.send(SendWebSocketMessage(SystemMessages::MainPlayerPickedUpToken));
            }
        }
    }
}

fn create_token(
    commands: &mut Commands,
    sprite_params: &mut Sprite3dParams,
    token_handle: &Res<TokenHandle>,
    position: Vec3,
) {
    commands.spawn((
        Token,
        Sprite3dBuilder {
            image: token_handle.0.clone(),
            pixels_per_metre: 100.,
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            pivot: Some(Vec2::new(0.5, 0.0)),
            double_sided: true,
            ..default()
        }.bundle(sprite_params),
        Transform {
            translation: position,
            scale: Vec3::splat(0.8),
            rotation: Quat::from_rotation_y(45f32.to_radians()),
            ..default()
        },
    ));
}

fn spawn_tokens_around_center(
    commands: &mut Commands,
    sprite_params: &mut Sprite3dParams,
    token_handle: &Res<TokenHandle>,
    center: Vec2,
    inner_radius: f32,
    outer_radius: f32,
    min_distance_between: f32,
    max_tokens: usize,
) {
    let mut candidates = Vec::new();

    // 1. Generate all valid candidates
    let grid_radius = outer_radius.ceil() as i32;
    for dz in -grid_radius..=grid_radius {
        for dx in -grid_radius..=grid_radius {
            let offset = Vec2::new(dx as f32, dz as f32);
            let distance = offset.length();

            if distance >= inner_radius && distance <= outer_radius {
                candidates.push(center + offset);
            }
        }
    }

    // 2. Shuffle them randomly
    fastrand::shuffle(&mut candidates);

    let mut placed = Vec::new();

    // 3. Place only if it's far enough from existing tokens
    for candidate in candidates {
        if placed.len() >= max_tokens {
            break;
        }

        let too_close = placed
            .iter()
            .any(|existing| candidate.distance(*existing) < min_distance_between);

        if !too_close {
            let world_pos = iso_to_world(candidate);
            create_token(
                commands,
                sprite_params,
                token_handle,
                Vec3::new(world_pos.x, 0.0, world_pos.y),
            );
            placed.push(candidate);
        }
    }
}

fn iso_to_world(grid: Vec2) -> Vec2 {
    let tile_width = 1.0;
    let tile_height = 1.0;

    Vec2::new(
        (grid.x - grid.y) * tile_width / 2.0,
        (grid.x + grid.y) * tile_height / 2.0,
    )
}
