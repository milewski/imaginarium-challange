use bevy::app::{App, Plugin, Startup, Update};
use bevy::asset::{AssetServer, Assets};
use bevy::color::Color;
use bevy::color::palettes::tailwind::{GRAY_100, GRAY_200};
use bevy::input::ButtonInput;
use bevy::math::Vec3;
use bevy::pbr::{DirectionalLight, MeshMaterial3d, StandardMaterial};
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy_infinite_grid::{InfiniteGridBundle, InfiniteGridSettings};
use crate::mouse_plugin::{Draggable, MousePlugin};

#[derive(Component, Debug, Default)]
pub struct CameraController;

impl Plugin for CameraController {
    fn build(&self, app: &mut App) {
        app.add_plugins(MousePlugin);
        app.add_systems(Update, camera_controller);
        app.add_systems(Startup, create_infinite_grid_system);
    }
}

fn camera_controller(
    draggable: Res<Draggable>,
    mut query: Query<(&mut Transform, &mut CameraController), With<Projection>>,
) {
    if let Ok((mut transform, mut state)) = query.get_single_mut() {
        draggable.apply(&mut transform);
    }
}

fn create_infinite_grid_system(
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
            scale: 1.0,
            ..default()
        },
        ..default()
    });

    commands.spawn((
        Camera3d::default(),
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 16.0,
            },
            near: -100.0,
            far: 100.0,
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_xyz(5.0, 4.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        CameraController::default(),
    ));

    // plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(6.0, 6.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            ..default()
        },
        Transform::from_xyz(3.0, 8.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}