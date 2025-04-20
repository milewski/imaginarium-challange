mod mouse_plugin;
mod player;

use crate::mouse_plugin::{Draggable, MousePlugin};
use crate::player::{Player, PlayerPlugin};
use bevy::color::palettes::tailwind::{GRAY_100, GRAY_200};
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy_infinite_grid::{InfiniteGridBundle, InfiniteGridPlugin, InfiniteGridSettings};
use bevy_sprite3d::{Sprite3dBuilder, Sprite3dParams, Sprite3dPlugin};

#[derive(Resource, Default)]
struct AssetsCache(Handle<Image>);

#[derive(States, Hash, Clone, PartialEq, Eq, Debug, Default)]
enum GameState {
    #[default]
    Loading,
    Ready,
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, InfiniteGridPlugin))
        .add_plugins(CameraController)
        .add_plugins(PlayerPlugin)
        .add_plugins(Sprite3dPlugin)
        // .add_plugins(PanOrbitCameraPlugin)
        .insert_resource(ClearColor(Color::WHITE))
        .init_state::<GameState>()
        .add_systems(Startup, setup_system)
        .add_systems(Startup, sprite_system)
        .insert_resource(AssetsCache::default())
        .add_systems(
            Startup,
            |asset_server: Res<AssetServer>, mut assets: ResMut<AssetsCache>| {
                assets.0 = asset_server.load("funny-guy.png");
            },
        )
        .add_systems(Update, sprite_system.run_if(in_state(GameState::Loading)))
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

fn camera_controller(
    keyboard: Res<ButtonInput<KeyCode>>,
    draggable: Res<Draggable>,
    mut query: Query<(&mut Transform, &mut CameraController), With<Projection>>,
) {
    if let Ok((mut transform, mut state)) = query.get_single_mut() {
        draggable.apply(&mut transform);
    }
}

fn sprite_system(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut sprite_params: Sprite3dParams,
    assets: Res<AssetsCache>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if !asset_server
        .get_load_state(assets.0.id())
        .is_some_and(|s| s.is_loaded())
    {
        return;
    }

    next_state.set(GameState::Ready);

    commands.spawn((
        Sprite3dBuilder {
            image: assets.0.clone(),
            pixels_per_metre: 100.,
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            pivot: Some(Vec2::new(0.5, 0.0)),
            ..default()
        }
        .bundle(&mut sprite_params),
        Transform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            rotation: Quat::from_rotation_y(45f32.to_radians()),
            ..default()
        },
    ));
}

fn setup_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut windows: Query<&mut Window>,
    asset_server: Res<AssetServer>,
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

    // commands.spawn((
    //     Transform::from_translation(Vec3::new(0.0, 1.5, 5.0)),
    //     PanOrbitCamera::default(),
    // ));

    commands.spawn((
        Camera3d::default(),
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 12.0,
            },
            near: -10.0,
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
        Player::default(),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            ..default()
        },
        Transform::from_xyz(3.0, 8.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
