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
use bevy_web_asset::WebAssetPlugin;
use bevy_kira_audio::Audio;
use bevy_kira_audio::AudioPlugin;
use bevy_kira_audio::AudioControl;
use crate::sound_effects::SoundEffectsPlugin;

mod js_bridge_plugin;
mod mouse_plugin;
mod network;
mod robot;
mod tokens;
mod builder;
mod ui;
mod camera;
mod sound_effects;

#[wasm_bindgen]
pub fn start_application(canvas: Option<String>) {
    App::new()
        .add_plugins(JsBridgePlugin)
        .add_plugins((
            NetworkPlugin,
            RobotPlugin,
            TokensPlugin,
            UIPlugin,
            BuilderPlugin,
        ))
        .add_plugins((
            WebAssetPlugin::default(),
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
            AudioPlugin,
            InfiniteGridPlugin,
        ))
        .add_plugins(SoundEffectsPlugin)
        .add_plugins(CameraController)
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

