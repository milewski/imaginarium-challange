use std::time::Duration;

use bevy::animation::{AnimationClip, AnimationPlayer};
use bevy::app::{App, Startup, Update};
use bevy::asset::{AssetPath, AssetServer, Assets, Handle};
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

use crate::network::{SendWebSocketMessage, WebSocketMessageReceived};
use crate::tokens::Token;
use crate::ui::{UiInputBlocker};

pub const ROBOT_GLB_PATH: &str = "RobotExpressive.glb";

pub struct RobotPlugin;

#[derive(Component, Default, Debug)]
pub struct Robot {
    target: Option<Vec3>,
    animation_timer: Option<Timer>,
    animation: Option<PlayerAnimation>,
}

#[derive(Component, Debug, Clone)]
pub enum PlayerKind {
    MainPlayer(PlayerData),
    Enemy(PlayerData),
}

impl Plugin for RobotPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(Startup, initialize_animations_system);
        app.add_systems(Update, listen_to_player_spawn_events_system);
        app.add_systems(Update, listen_to_player_balance_update);

        app.add_systems(Update, remove_disconnected_players_system);
        // app.add_systems(Update, start_robot_idle_animation);
        app.add_systems(Update, robots_movement_system);
        app.add_systems(Update, listen_for_enemy_movement_system);
        app.add_systems(Update, move_robot_animation_system);
        // app.add_systems(Update, robot_animation_movement_system);
        app.add_systems(
            Update, calculate_player_movement_target_system.run_if(should_run),
        );
    }
}

fn move_robot_animation_system(
    time: Res<Time>,
    children: Query<&Children>,
    mut robots: Query<(Entity, &mut Robot)>,
    mut query: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
) {
    'root: for (entity, mut robot) in robots.iter_mut() {
        if let Some(animation) = robot.animation {
            for child in children.iter_descendants(entity) {
                if let Ok((mut player, mut transitions)) = query.get_mut(child) {

                    // Hack to reset the pose of the model... no time to figure out what's wrong with it
                    if animation == PlayerAnimation::Idle && robot.animation_timer.is_none() {
                        transitions.play(
                            &mut player,
                            PlayerAnimation::Jumping.to_index(),
                            Duration::from_millis(250),
                        );

                        robot.animation_timer = Some(Timer::from_seconds(0.708, TimerMode::Once));
                    }

                    if let Some(timer) = robot.animation_timer.as_mut() {
                        timer.tick(time.delta());

                        if !timer.finished() {
                            continue 'root;
                        }

                        robot.animation_timer = None;
                        robot.animation = None;
                    }

                    if player.is_playing_animation(animation.to_index()) == false {
                        transitions.play(&mut player, animation.to_index(), Duration::from_millis(250)).repeat();
                    }
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
                    .play(&mut player, animations.index[0], Duration::ZERO)
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
    mut events: EventReader<WebSocketMessageReceived>,
) {
    for event in events.read() {
        match &event.0 {
            SystemMessages::EnemyPlayerSpawn { data } => {
                spawn_player(&asset_server, &mut commands, &mut graphs, PlayerKind::Enemy(data.clone()))
            }
            SystemMessages::MainPlayerSpawn { data } => {
                spawn_player(&asset_server, &mut commands, &mut graphs, PlayerKind::MainPlayer(data.clone()))
            }
            _ => continue
        }
    }
}

fn listen_to_player_balance_update(
    mut events: EventReader<WebSocketMessageReceived>,
    mut robots: Query<&mut PlayerKind, With<Player>>,
) {
    for event in events.read() {
        match event.0 {
            SystemMessages::MainPlayerCurrentBalance { balance } => {
                if let PlayerKind::MainPlayer(ref mut data) = *robots.single_mut() {
                    info!("mutated {}", balance);
                    data.balance = balance;
                }
            }
            _ => continue
        }
    }
}

fn spawn_player(
    asset_server: &Res<AssetServer>,
    commands: &mut Commands,
    graphs: &mut ResMut<Assets<AnimationGraph>>,
    player_kind: PlayerKind,
) {
    let (graph, index) = AnimationGraph::from_clips(
        PlayerAnimation::clips().map(|clip| asset_server.load(clip))
    );

    let animations = Animations {
        index,
        graph: graphs.add(graph),
    };

    let mesh = SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(ROBOT_GLB_PATH)));

    let robot = Robot::default();

    let transform = Transform {
        scale: Vec3::splat(0.8),
        translation: match &player_kind {
            PlayerKind::MainPlayer(data) => data.position.to_vec3(),
            PlayerKind::Enemy(data) => data.position.to_vec3(),
        },
        ..default()
    };

    commands
        .spawn((animations, robot, transform, mesh, player_kind.clone()))
        .insert_if(Player::default(), || match player_kind {
            PlayerKind::MainPlayer(_) => true,
            PlayerKind::Enemy(_) => false
        })
        .observe(initialize_animations_observer);
}

/// Only run when user click and there is a player spawn in the world
fn should_run(mouse: Res<ButtonInput<MouseButton>>, query: Query<(), With<Player>>, blocker: Res<UiInputBlocker>) -> bool {
    mouse.just_pressed(MouseButton::Left) && query.is_empty() == false && blocker.0 == false
}

fn remove_disconnected_players_system(
    mut commands: Commands,
    mut robots: Query<(Entity, &PlayerKind)>,
    mut events: EventReader<WebSocketMessageReceived>,
) {
    for event in events.read() {
        if let SystemMessages::EnemyDisconnected { id } = event.0 {
            for (entity, kind) in &mut robots {
                if let PlayerKind::Enemy(data) = kind {
                    if data.id == id {
                        commands.entity(entity).despawn_recursive();
                    }
                }
            }
        }
    }
}

fn listen_for_enemy_movement_system(
    mut robots: Query<(&PlayerKind, &mut Robot)>,
    mut events: EventReader<WebSocketMessageReceived>,
) {
    for event in events.read() {
        if let SystemMessages::EnemyPosition { id, coordinate } = event.0 {
            for (kind, mut robot) in &mut robots {
                if let PlayerKind::Enemy(data) = kind {
                    if data.id == id {
                        robot.animation = Some(PlayerAnimation::Running);
                        robot.target = Some(coordinate.to_vec3())
                    }
                }
            }
        }
    }
}

fn robots_movement_system(
    mut robots: Query<(&mut Robot, &mut Transform)>,
    time: Res<Time>,
    blocker: Res<UiInputBlocker>,
) {
    if blocker.0 {
        return;
    }

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
    mut event: EventWriter<SendWebSocketMessage>,
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

                let movement_target = find_closest_clear_path(
                    &player_transform.translation,
                    &intersection_point,
                    &obstacles,
                );

                robot.animation = Some(PlayerAnimation::Running);
                robot.target = Some(movement_target);

                event.send(SendWebSocketMessage(SystemMessages::PlayerPosition { coordinate: movement_target.into() }));

                info!("target -> {:?}", intersection_point);
            }
        }
    }
}

#[derive(Component)]
struct Animations {
    index: Vec<AnimationNodeIndex>,
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
    let grid_size = 1.0;
    let mut blocked = false;
    let mut closest_safe_point = end.clone();

    let band_half_length = 3.0 * grid_size;
    let band_thickness = 1.0 * grid_size;

    for obstacle in elements {
        let delta = end - start;
        let direction = delta.normalize_or_zero();
        let steps = delta.length().ceil() as i32;

        for index in 1..=steps {
            let step_point = start + direction * (index as f32 * grid_size);

            // Offset from obstacle center
            let dx = step_point.x - obstacle.x;
            let dz = step_point.z - obstacle.z;

            // Project onto (1, -1) direction for isometric horizontal line
            let iso_horizontal = (dx - dz) / 2.0;   // movement along the horizontal isometric line
            let iso_vertical = (dx + dz) / 2.0;   // perpendicular offset (vertical band thickness)

            // Check if inside the isometric-aligned rectangle
            if iso_horizontal.abs() <= band_half_length &&
                iso_vertical.abs() <= band_thickness {
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
        closest_safe_point
    } else {
        end.clone()
    }
}

#[derive(Component, Default, Debug)]
pub struct Player;

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum PlayerAnimation {
    #[default]
    Idle = 2,
    Jumping = 3,
    Running = 6,
    Standing = 8,
    Walking = 10,
}

impl PlayerAnimation {
    pub fn clips() -> [AssetPath<'static>; 5] {
        [
            GltfAssetLabel::Animation(Self::Idle as usize).from_asset(ROBOT_GLB_PATH),
            GltfAssetLabel::Animation(Self::Jumping as usize).from_asset(ROBOT_GLB_PATH),
            GltfAssetLabel::Animation(Self::Running as usize).from_asset(ROBOT_GLB_PATH),
            GltfAssetLabel::Animation(Self::Walking as usize).from_asset(ROBOT_GLB_PATH),
            GltfAssetLabel::Animation(Self::Standing as usize).from_asset(ROBOT_GLB_PATH),
        ]
    }

    pub fn to_index(&self) -> AnimationNodeIndex {
        match self {
            PlayerAnimation::Idle => 1.into(),
            PlayerAnimation::Jumping => 2.into(),
            PlayerAnimation::Running => 3.into(),
            PlayerAnimation::Walking => 4.into(),
            PlayerAnimation::Standing => 5.into(),
        }
    }
}

