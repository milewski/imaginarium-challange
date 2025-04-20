use bevy::app::{App, Plugin, Update};
use bevy::input::ButtonInput;
use bevy::math::Vec3;
use bevy::prelude::{
    Camera, Component, GlobalTransform, InfinitePlane3d, MouseButton, Projection, Query, Res, Time,
    Transform, Window, With,
};

#[derive(Component, Default, Debug)]
pub struct Player {
    path: Option<Vec3>,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, player_system)
            .add_systems(Update, player_movement_system);
    }
}

fn player_system(
    mut query: Query<(&mut Player)>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Projection>>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let (camera, camera_transform) = camera_query.single();
    let window = windows.single();

    if let Some(cursor_pos) = window.cursor_position() {
        if let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) {
            let plane_origin = Vec3::ZERO;
            let plane = InfinitePlane3d::new(Vec3::Y);

            if let Some(distance) = ray.intersect_plane(plane_origin, plane) {
                let mut intersection_point = ray.origin + ray.direction * distance;

                let grid_size = 1.0;

                intersection_point.x = (intersection_point.x / grid_size).round() * grid_size;
                intersection_point.z = (intersection_point.z / grid_size).round() * grid_size;

                if let Ok(mut player) = query.get_single_mut() {
                    player.path = Some(intersection_point);
                }
            }
        }
    }
}

fn player_movement_system(mut query: Query<(&mut Transform, &mut Player)>, time: Res<Time>) {
    if let Ok((mut transform, mut player)) = query.get_single_mut() {
        if let Some(target_pos) = player.path {
            let current = transform.translation;
            let target = Vec3::new(target_pos.x, current.y, target_pos.z); // keep Y

            let distance_to_target = current.distance(target);

            // Easing factor: how fast to move per second relative to distance
            let ease_speed = 8.0;

            // Interpolation factor, clamped to not overshoot
            let t = (ease_speed * time.delta_secs()).min(1.0);

            // Interpolate toward the target
            transform.translation = current.lerp(target, t);

            // Snap if we're close enough
            if transform.translation.distance(target) < 0.05 {
                transform.translation = target;
                player.path = None;
            }
        }
    }
}
