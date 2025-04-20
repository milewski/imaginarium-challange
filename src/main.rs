mod mouse_plugin;

use crate::mouse_plugin::{Draggable, MousePlugin};
use bevy::color::palettes::tailwind::{GRAY_50, GRAY_100, GRAY_200};
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy_infinite_grid::{InfiniteGridBundle, InfiniteGridPlugin, InfiniteGridSettings};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, InfiniteGridPlugin))
        .add_plugins(CameraController)
        .insert_resource(ClearColor(Color::WHITE))
        .add_systems(Startup, setup_system)
        .add_systems(Update, player_system)
        .run();
}

#[derive(Component, Debug, Default)]
struct CameraController;

impl Plugin for CameraController {
    fn build(&self, app: &mut App) {
        app.add_plugins(MousePlugin)
            .add_systems(Update, camera_controller);
    }
}

fn player_system(
    mut query: Query<(&mut Transform, &mut Player)>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Projection>>,
)
{
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
                let intersection_point = ray.origin + ray.direction * distance;

                if let Ok((mut transform, _player)) = query.get_single_mut() {
                    transform.translation.x = intersection_point.x;
                    transform.translation.z = intersection_point.z;
                }
            }
        }
    }
}

fn camera_controller(
    keyboard: Res<ButtonInput<KeyCode>>,
    draggable: Res<Draggable>,
    mut query: Query<(&mut Transform, &mut CameraController), With<Projection>>,
) {
    if let Ok((mut transform, mut state)) = query.get_single_mut() {
        draggable.apply(&mut transform);

        if keyboard.pressed(KeyCode::ArrowUp) {
            transform.translation.y += 0.1;
        }

        if keyboard.pressed(KeyCode::ArrowDown) {
            transform.translation.y -= 0.1;
        }

        if keyboard.pressed(KeyCode::ArrowLeft) {
            transform.translation.z += 0.1;
            transform.translation.x -= 0.1;
        }

        if keyboard.pressed(KeyCode::ArrowRight) {
            transform.translation.z -= 0.1;
            transform.translation.x += 0.1;
        }
    }
}

#[derive(Component, Default, Debug)]
struct Player;

fn setup_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(InfiniteGridBundle {
        settings: InfiniteGridSettings {
            x_axis_color: GRAY_200.into(),
            z_axis_color: GRAY_200.into(),
            major_line_color: GRAY_100.into(),
            minor_line_color: GRAY_100.into(),
            ..default()
        },
        ..default()
    });

    commands.spawn((
        Camera3d::default(),
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 12.0,
            },
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        CameraController::default(),
    ));

    // plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(6.0, 6.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::default())),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
        Transform::from_xyz(1.5, 0.5, -1.5),
        Player::default()
    ));

    commands.spawn((PointLight::default(), Transform::from_xyz(3.0, 8.0, 5.0)));
}
