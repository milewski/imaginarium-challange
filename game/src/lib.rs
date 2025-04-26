#![allow(warnings)]
use crate::builder::BuilderPlugin;
use crate::js_bridge_plugin::JsBridgePlugin;
use crate::camera::CameraController;
use crate::network::NetworkPlugin;
use crate::robot::RobotPlugin;
use crate::tokens::TokensPlugin;
use crate::ui::UIPlugin;
use bevy::app::{App, Plugin};
use bevy::asset::AssetMetaCheck;
use bevy::color::Color;
use bevy::prelude::*;
use bevy::DefaultPlugins;
use bevy_infinite_grid::InfiniteGridPlugin;
use bevy_sprite3d::Sprite3dPlugin;
use num_traits::Float;
use wasm_bindgen::prelude::wasm_bindgen;

mod js_bridge_plugin;
mod mouse_plugin;
mod network;
mod robot;
mod tokens;
mod builder;
mod ui;
mod camera;

#[wasm_bindgen]
pub fn start_application(canvas: Option<String>) {
    App::new()
        .add_plugins(UIPlugin)
        .add_plugins(JsBridgePlugin)
        .add_plugins(RobotPlugin)
        .add_plugins(BuilderPlugin)
        .add_plugins(NetworkPlugin)
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        canvas,
                        fit_canvas_to_parent: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                }),
            InfiniteGridPlugin,
        ))
        .add_plugins(CameraController)
        .add_plugins(TokensPlugin)
        .add_plugins(Sprite3dPlugin)
        // .add_plugins(FoxPlugin)
        // .add_plugins(PanOrbitCameraPlugin)
        .insert_resource(ClearColor(Color::WHITE))
        // .add_systems(Startup, joint_animation)
        // .add_plugins((
        //     SkinnedAabbPlugin,
        //     SkinnedAabbDebugPlugin::enable_by_default(),
        // ))
        // .add_systems(Update, init_animations)
        .run();
}

fn get_random_between<T: Float>(min: T, max: T) -> T {
    let t: f32 = fastrand::f32(); // Still f32
    min + (max - min) * T::from(t).unwrap()
}

