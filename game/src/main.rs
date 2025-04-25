#![allow(warnings)]
mod button_plugin;
mod mouse_plugin;
mod network;
mod robot;
mod tokens;
mod builder;

use crate::builder::BuilderPlugin;
use crate::button_plugin::ButtonPlugin;
use crate::mouse_plugin::{Draggable, MousePlugin};
use crate::network::NetworkPlugin;
use crate::robot::RobotPlugin;
use crate::tokens::TokensPlugin;
use bevy::color::palettes::tailwind::{GRAY_100, GRAY_200};
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy::render::mesh::skinning::SkinnedMesh;
use bevy_infinite_grid::{InfiniteGridBundle, InfiniteGridPlugin, InfiniteGridSettings};
use bevy_mod_skinned_aabb::debug::SkinnedAabbDebugPlugin;
use bevy_mod_skinned_aabb::SkinnedAabbPlugin;
use bevy_sprite3d::{Sprite3dBuilder, Sprite3dParams, Sprite3dPlugin};
use num_traits::{Float, FloatConst};
use std::any::Any;
use std::f32::consts::PI;

#[derive(Resource, Default)]
struct AssetsCache(Vec<Handle<Image>>);

#[derive(States, Hash, Clone, PartialEq, Eq, Debug, Default)]
enum GameState {
    #[default]
    Loading,
    Ready,
}

fn main() {
    App::new()
        .add_plugins(RobotPlugin)
        .add_plugins(BuilderPlugin)
        .add_plugins(NetworkPlugin)
        .add_plugins((DefaultPlugins, InfiniteGridPlugin))
        .add_plugins(CameraController)
        .add_plugins((ButtonPlugin, TokensPlugin))
        .add_plugins(Sprite3dPlugin)
        // .add_plugins(FoxPlugin)
        // .add_plugins(PanOrbitCameraPlugin)
        .insert_resource(ClearColor(Color::WHITE))
        .init_state::<GameState>()
        .add_systems(Startup, setup_system)
        .add_systems(Startup, sprite_system)
        .insert_resource(AssetsCache::default())
        // .add_systems(Startup, joint_animation)
        // .add_plugins((
        //     SkinnedAabbPlugin,
        //     SkinnedAabbDebugPlugin::enable_by_default(),
        // ))
        // .add_systems(Update, init_animations)
        .add_systems(
            Startup,
            |asset_server: Res<AssetServer>, mut assets: ResMut<AssetsCache>| {
                assets.0.push(asset_server.load("funny-guy.png"));
                assets.0.push(asset_server.load("giraffe.png"));
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

fn get_random_between<T: Float>(min: T, max: T) -> T {
    let t: f32 = fastrand::f32(); // Still f32
    min + (max - min) * T::from(t).unwrap()
}

fn sprite_system(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut sprite_params: Sprite3dParams,
    assets: Res<AssetsCache>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for asset in assets.0.iter() {
        if !asset_server
            .get_load_state(asset.id())
            .is_some_and(|s| s.is_loaded())
        {
            return;
        }
    }

    next_state.set(GameState::Ready);

    for asset in assets.0.iter() {
        commands.spawn((
            Sprite3dBuilder {
                image: asset.clone(),
                pixels_per_metre: 100.,
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                pivot: Some(Vec2::new(0.5, 0.0)),
                double_sided: true,
                ..default()
            }
                .bundle(&mut sprite_params),
            Transform {
                translation: Vec3::new(
                    get_random_between(-5.0, 5.0),
                    0.0,
                    get_random_between(-5.0, 5.0),
                ),
                rotation: Quat::from_rotation_y(45f32.to_radians()),
                ..default()
            },
        ));
    }
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
        // Mesh3d(meshes.add(Cuboid::default())),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
        Transform {
            translation: Vec3::new(0.0, 0.5, 0.0),
            ..default()
        },
        // Player::default(),
        // SceneRoot(
        //     asset_server.load(GltfAssetLabel::Scene(0).from_asset("robot.glb")),
        // ),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            ..default()
        },
        Transform::from_xyz(3.0, 8.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // commands.spawn((
    //     // Transform {
    //     //     translation: Vec3::new(0.0, 0.5, 0.0),
    //     //     ..default()
    //     // },
    //     SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("RobotExpressive.glb"))),
    // ));
}

fn joint_animation(
    mut commands: Commands,
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
    asset_server: Res<AssetServer>,
) {
    let mut animation_graph = AnimationGraph::new();
    let blend_node = animation_graph.add_blend(0.5, animation_graph.root);

    animation_graph.add_clip(
        asset_server.load(GltfAssetLabel::Animation(0).from_asset("RobotExpressive.glb")),
        1.0,
        animation_graph.root,
    );
    animation_graph.add_clip(
        asset_server.load(GltfAssetLabel::Animation(3).from_asset("RobotExpressive.glb")),
        1.0,
        blend_node,
    );

    animation_graph.add_clip(
        asset_server.load(GltfAssetLabel::Animation(4).from_asset("RobotExpressive.glb")),
        1.0,
        blend_node,
    );

    animation_graph.add_clip(
        asset_server.load(GltfAssetLabel::Animation(5).from_asset("RobotExpressive.glb")),
        1.0,
        blend_node,
    );

    let handle = animation_graphs.add(animation_graph);
    commands.insert_resource(ExampleAnimationGraph(handle));
}

fn init_animations(
    mut commands: Commands,
    mut query: Query<(Entity, &mut AnimationPlayer)>,
    animation_graph: Res<ExampleAnimationGraph>,
    mut done: Local<bool>,
) {
    if *done {
        return;
    }

    info!("initializing animations");

    for (entity, mut player) in query.iter_mut() {
        commands
            .entity(entity)
            .insert((AnimationGraphHandle(animation_graph.0.clone()),));

        player.play(3.into()).repeat();

        *done = true;
    }
}

/// The [`AnimationGraph`] asset, which specifies how the animations are to
/// be blended together.
#[derive(Clone, Resource)]
struct ExampleAnimationGraph(Handle<AnimationGraph>);
