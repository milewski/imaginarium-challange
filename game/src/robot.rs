use std::time::Duration;

use bevy::animation::{AnimationClip, AnimationPlayer};
use bevy::app::{App, Startup, Update};
use bevy::asset::{Assets, AssetServer, Handle};
use bevy::ecs::bundle::DynamicBundle;
use bevy::gltf::GltfAssetLabel;
use bevy::input::ButtonInput;
use bevy::log::info;
use bevy::math::{Quat, Vec3};
use bevy::pbr::StandardMaterial;
use bevy::prelude::*;
use bevy::scene::SceneInstanceReady;
use bevy_rapier2d::geometry::Collider;
use bevy_sprite3d::Sprite3d;

use shared::{PlayerData, PlayerId, SystemMessages};

use crate::fox_plugin::ROBOT_GLB_PATH;
use crate::network::{SendWebSocketMessage, WebSocketMessageReceived};
use crate::player::{Player, PlayerAnimation};
use crate::tokens::Token;

pub struct RobotPlugin;

#[derive(Component, Debug)]
pub struct Robot {
    target: Option<Vec3>,
    animation: Option<PlayerAnimation>,
}

#[derive(Component, Debug, Copy, Clone)]
enum PlayerKind {
    MainPlayer(PlayerData),
    Enemy(PlayerData)
}

impl Plugin for RobotPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(Startup, initialize_animations_system);
        app.add_systems(Update, listen_to_player_spawn_events_system);
        // app.add_systems(Update, start_robot_idle_animation);
        app.add_systems(Update, robots_movement_system);
        app.add_systems(Update, listen_for_enemy_movement_system);
        app.add_systems(Update, move_robot_animation_system);
        // app.add_systems(Update, robot_animation_movement_system);
        app.add_systems(
            Update,
            calculate_player_movement_target_system.run_if(should_run),
        );
    }
}

fn move_robot_animation_system(
    children: Query<&Children>,
    mut robots: Query<(Entity, &mut Robot)>,
    mut query: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
) {
    for (entity, mut robot) in robots.iter_mut() {
        if let Some(animation) = robot.animation {
            for child in children.iter_descendants(entity) {
                if let Ok((mut player, mut transitions)) = query.get_mut(child) {
                    if player.is_playing_animation(animation.to_animation()) == false {
                        transitions.play(&mut player, animation.to_animation(), Duration::from_millis(250)).repeat();
                    }

                    robot.animation = None;
                }
            }
        }
    }
}

fn initialize_animations_observer(
    trigger: Trigger<SceneInstanceReady>,
    children: Query<&Children>,
    animations: Query<&Animations, With<Robot>>,
    mut commands: Commands,
    mut players: Query<&mut AnimationPlayer>,
) {
    if let Ok(animations) = animations.get(trigger.entity()) {
        for child in children.iter_descendants(trigger.entity()) {
            if let Ok(mut player) = players.get_mut(child) {
                let mut transitions = AnimationTransitions::new();

                transitions
                    .play(&mut player, animations.animations[0], Duration::ZERO)
                    .repeat();

                commands
                    .entity(child)
                    .insert(AnimationGraphHandle(animations.graph.clone()))
                    .insert(transitions);
            }
        }
    }
}

fn listen_to_player_spawn_events_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    mut events: EventReader<WebSocketMessageReceived>
) {
    for event in events.read() {
        match event.0 {
            SystemMessages::EnemyPlayerSpawn { data } => {
                spawn_player(&asset_server, &mut commands, &mut graphs, PlayerKind::Enemy(data))
            }
            SystemMessages::MainPlayerSpawn { data } => {
                spawn_player(&asset_server, &mut commands, &mut graphs, PlayerKind::MainPlayer(data))
            },
            _ => continue
        }
    }
}

fn spawn_player(
    asset_server: &Res<AssetServer>,
    commands: &mut Commands,
    graphs: &mut ResMut<Assets<AnimationGraph>>,
    player_kind: PlayerKind
) {
    let (graph, index) = AnimationGraph::from_clips(
        PlayerAnimation::clips().map(|clip| asset_server.load(clip))
    );

    let animation_to_play = Animations {
        animations: index,
        graph: graphs.add(graph),
    };

    let mesh = SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(ROBOT_GLB_PATH)));

    let robot = Robot {
        animation: None,
        target: None,
    };

    let transform = Transform {
        scale: Vec3::splat(0.5),
        ..default()
    };

    commands
        .spawn((animation_to_play, robot, transform, mesh, player_kind))
        .insert_if(Player::default(), || match player_kind {
            PlayerKind::MainPlayer(_) => true,
            PlayerKind::Enemy(_) => false
        })
        .observe(initialize_animations_observer);
}

/// Only run when user click and there is a player spawn in the world
fn should_run(mouse: Res<ButtonInput<MouseButton>>, query: Query<(), With<Player>>) -> bool {
    mouse.just_pressed(MouseButton::Left) && query.is_empty() == false
}

fn listen_for_enemy_movement_system(
    mut robots: Query<(&PlayerKind, &mut Robot)>,
    mut events: EventReader<WebSocketMessageReceived>
) {
    for event in events.read() {
        match event.0 {
            SystemMessages::EnemyPosition { id, coordinate } => {
                info!("got here {:?}", id);
                for (kind, mut robot) in &mut robots {
                    match *kind {
                        PlayerKind::Enemy(mut data) => {
                            info!("{:?} {:?} {:?}", data.id, id, data.id == id);
                            if data.id == id {
                                robot.target = Some(coordinate.to_vec3())
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}

fn robots_movement_system(mut robots: Query<(&mut Robot, &mut Transform)>, time: Res<Time>) {
    for (mut robot, mut transform) in robots.iter_mut() {
        if let Some(target) = robot.target {
            let to_target = target - transform.translation;
            let distance = to_target.length();

            if distance < 0.01 {
                transform.translation = target;
                robot.target = None;
                robot.animation = Some(PlayerAnimation::Idle);
                continue;
            }

            let direction = to_target.normalize();
            let desired_rotation = Quat::from_rotation_arc(Vec3::Z, direction);
            let rotation = transform
                .rotation
                .slerp(desired_rotation, (time.delta_secs() * 8.0).min(1.0));

            if rotation.is_nan() == false {
                transform.rotation = rotation;
            }

            let max_step = 6.0 * time.delta_secs();
            let step = direction * distance.min(max_step);

            transform.translation += step;

            if distance <= max_step {
                transform.translation = target;
                robot.target = None;
                robot.animation = Some(PlayerAnimation::Idle);
            }
        }
    }
}

fn calculate_player_movement_target_system(
    mut query: Query<(&mut Robot, &Transform), With<Player>>,
    mut obstacles_query: Query<(&mut Sprite3d, &Transform), Without<Token>>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform), With<Projection>>,
    mut event: EventWriter<SendWebSocketMessage>
) {
    let (mut robot, player_transform) = query.single_mut();
    let (camera, camera_transform) = camera.single();
    let window = windows.single();

    if let Some(cursor_position) = window.cursor_position() {
        if let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) {
            let plane_origin = Vec3::ZERO;
            let plane = InfinitePlane3d::new(Vec3::Y);

            if let Some(distance) = ray.intersect_plane(plane_origin, plane) {
                let mut intersection_point = ray.origin + ray.direction * distance;

                let grid_size = 1.0;

                intersection_point.x = (intersection_point.x / grid_size).round() * grid_size;
                intersection_point.z = (intersection_point.z / grid_size).round() * grid_size;

                let obstacles = obstacles_query
                    .iter()
                    .map(|(_, transform)| &transform.translation)
                    .collect::<Vec<_>>();

                robot.animation = Some(PlayerAnimation::Running);
                robot.target = Some(find_closest_clear_path(
                    &player_transform.translation,
                    &intersection_point,
                    &obstacles,
                ));

                event.send(SendWebSocketMessage(SystemMessages::PlayerPosition { coordinate: intersection_point.into() }));

                info!("target -> {:?}", intersection_point);
            }
        }
    }
}

#[derive(Component)]
struct Animations {
    animations: Vec<AnimationNodeIndex>,
    graph: Handle<AnimationGraph>,
}

// fn initialize_animations_system(
//     asset_server: Res<AssetServer>,
//     mut commands: Commands,
//     mut graphs: ResMut<Assets<AnimationGraph>>,
// ) {
//     let (graph, node_indices) =
//         AnimationGraph::from_clips(PlayerAnimation::clips().map(|clip| asset_server.load(clip)));
//
//     commands.insert_resource(Animations {
//         animations: node_indices,
//         graph: graphs.add(graph),
//     });
// }

// fn start_robot_idle_animation(
//     mut commands: Commands,
//     animations: Res<Animations>,
//     mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
// ) {
//     for (entity, mut player) in &mut players {
//         let mut transitions = AnimationTransitions::new();
//
//         transitions
//             .play(&mut player, animations.animations[0], Duration::ZERO)
//             .repeat();
//
//         commands
//             .entity(entity)
//             .insert(AnimationGraphHandle(animations.graph.clone()))
//             .insert(transitions);
//     }
// }

fn find_closest_clear_path(start: &Vec3, end: &Vec3, elements: &Vec<&Vec3>) -> Vec3 {
    let radius = 1.0;
    let grid_size = 1.0;
    let mut blocked = false;
    let mut closest_safe_point = end.clone();

    for obstacle in elements {
        let delta = end - start;
        let direction = delta.normalize_or_zero();
        let steps = delta.length().ceil() as i32;

        for index in 1..=steps {
            let step_point = start + direction * (index as f32 * grid_size);
            let distance = step_point.distance(**obstacle);

            if distance <= radius {
                blocked = true;
                closest_safe_point = start + direction * ((index - 1) as f32 * grid_size);
                closest_safe_point.x = (closest_safe_point.x / grid_size).round() * grid_size;
                closest_safe_point.z = (closest_safe_point.z / grid_size).round() * grid_size;
                break;
            }
        }

        if blocked {
            break;
        }
    }

    if blocked {
        closest_safe_point.clone()
    } else {
        end.clone()
    }
}
