use bevy::app::{App, Plugin, Update};
use bevy::input::ButtonInput;
use bevy::log::info;
use bevy::math::{Quat, Vec2, Vec3};
use bevy::prelude::{
    AnimationNodeIndex, Camera, Component, GlobalTransform, InfinitePlane3d, MouseButton,
    Projection, Query, Res, Time, Transform, Window, With,
};
use bevy_rapier2d::prelude::Collider;
use bevy_sprite3d::{Sprite3d, Sprite3dBuilder, Sprite3dBundle};
use std::time::Duration;

#[derive(Default, Debug, Clone, PartialEq)]
pub enum PlayerAnimation {
    #[default]
    Idle,
    Running,
    Walking,
    Jumping,
}

impl PlayerAnimation {
    pub fn to_animation(&self) -> (AnimationNodeIndex, Duration) {
        match self {
            PlayerAnimation::Idle => (12.into(), Duration::from_millis(250)),
            PlayerAnimation::Running => (8.into(), Duration::from_millis(250)),
            PlayerAnimation::Walking => (4.into(), Duration::from_millis(250)),
            PlayerAnimation::Jumping => (11.into(), Duration::from_millis(250)),
        }
    }
}

#[derive(Component, Default, Debug)]
pub struct Player {
    pub current_animation: PlayerAnimation,
    path: Option<(Vec3, Quat)>,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, player_system)
            .add_systems(Update, player_movement_system);
    }
}

fn distance_to_segment(p: Vec3, a: Vec3, b: Vec3) -> f32 {
    let ab = b - a;
    let ap = p - a;
    let t = (ap.dot(ab) / ab.length_squared()).clamp(0.0, 1.0);
    let closest_point = a + ab * t;
    p.distance(closest_point)
}

fn is_blocked(start: &Vec3, end: &Vec3, elements: Vec<&Vec3>) -> Vec3 {
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
            let distance = step_point.distance(*obstacle);

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

fn player_system(
    mut query: Query<(&mut Player, &Transform)>,
    mut bundle_query: Query<(&mut Sprite3d, &Transform)>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Projection>>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let (camera, camera_transform) = camera_query.single();
    let window = windows.single();

    let elements = bundle_query
        .iter()
        .map(|(_, transform)| &transform.translation)
        .collect::<Vec<_>>();

    if let Some(cursor_pos) = window.cursor_position() {
        if let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) {
            let plane_origin = Vec3::ZERO;
            let plane = InfinitePlane3d::new(Vec3::Y);

            if let Some(distance) = ray.intersect_plane(plane_origin, plane) {
                let mut intersection_point = ray.origin + ray.direction * distance;

                let grid_size = 1.0;

                intersection_point.x = (intersection_point.x / grid_size).round() * grid_size;
                intersection_point.z = (intersection_point.z / grid_size).round() * grid_size;

                if let Ok((mut player, transform)) = query.get_single_mut() {
                    // Direction from player to target (XZ only)
                    let mut direction = intersection_point - transform.translation;
                    direction.y = 0.0; // Ignore Y to constrain rotation to XZ plane

                    let rotation = if direction.length_squared() > 0.0 {
                        direction = direction.normalize();

                        // Calculate angle between forward (Z+) and direction
                        let angle = direction.x.atan2(direction.z);

                        // Rotate player around Y axis
                        Quat::from_rotation_y(angle)
                    } else {
                        transform.rotation
                    };

                    player.current_animation = PlayerAnimation::Running;
                    player.path = Some((
                        is_blocked(&transform.translation, &intersection_point, elements),
                        rotation,
                    ));

                    info!("target -> {:?}", intersection_point);
                }
            }
        }
    }
}

fn player_movement_system(
    mut query: Query<(&mut Transform, &mut Player)>,
    time: Res<Time>,
    collision_query: Query<&Collider>, // Add a query for colliders
) {
    if let Ok((mut transform, mut player)) = query.get_single_mut() {
        // println!("{:?}", get_grid_coordinates(&transform));

        if let Some((target_pos, rotation)) = player.path {
            let current = transform.translation;
            let current_rotation = transform.rotation;
            let target = Vec3::new(target_pos.x, current.y, target_pos.z); // maintain current Y

            let direction = target - current;
            let distance = direction.length();

            if distance > 0.01 {
                // Movement speed in units per second
                let speed = 5.0;
                let max_step = speed * time.delta_secs();

                let movement = if distance <= max_step {
                    // Snap if we're close enough
                    player.path = None;
                    player.current_animation = PlayerAnimation::Idle;
                    target - current // just go straight to the target
                } else {
                    direction.normalize() * max_step
                };

                transform.translation += movement;

                let t = (time.delta_secs() * 8.0).min(1.0);
                transform.rotation = current_rotation.slerp(rotation, t);
            } else {
                transform.translation = target;
                transform.rotation = rotation;
                player.path = None;
                player.current_animation = PlayerAnimation::Idle;
            }
        }
    }
}
