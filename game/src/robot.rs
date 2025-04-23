use std::time::Duration;

use bevy::animation::{AnimationClip, AnimationPlayer};
use bevy::app::{App, Startup, Update};
use bevy::asset::{Assets, AssetServer, Handle};
use bevy::gltf::GltfAssetLabel;
use bevy::log::info;
use bevy::math::Vec3;
use bevy::pbr::StandardMaterial;
use bevy::prelude::{Added, AnimationGraph, AnimationGraphHandle, AnimationNodeIndex, AnimationNodeType, AnimationTransitions, Changed, Commands, default, Entity, Mesh, Plugin, Query, Res, ResMut, Resource, SceneRoot, Transform};

use crate::fox_plugin::FOX_PATH;
use crate::player::{Player, PlayerAnimation};

pub struct RobotPlugin;

impl Plugin for RobotPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, initialize_animations);
        app.add_systems(Update, start_robot_idle_animation);
        app.add_systems(Update, handle_player_movement_animation);
    }
}

fn handle_player_movement_animation(
    mut animation_players: Query<(Changed<Player>, &mut AnimationPlayer, &mut AnimationTransitions)>,
)
{
    for (player, animator, transition) in &mut animation_players {
        info!("{:?}", player);
    }
}

#[derive(Resource)]
struct Animations {
    animations: Vec<AnimationNodeIndex>,
    graph: Handle<AnimationGraph>,
}

fn initialize_animations(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    let (graph, node_indices) = AnimationGraph::from_clips(
        PlayerAnimation::clips().map(|clip| asset_server.load(clip))
    );

    commands.insert_resource(Animations {
        animations: node_indices,
        graph: graphs.add(graph)
    });

    // commands.spawn((
    //     SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(FOX_PATH))),
    //     Transform {
    //         scale: Vec3::splat(0.5),
    //         ..default()
    //     },
    //     Player::default(),
    // ));
}

fn start_robot_idle_animation(
    mut commands: Commands,
    animations: Res<Animations>,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) {
    for (entity, mut player) in &mut players {
        let mut transitions = AnimationTransitions::new();

        transitions
            .play(&mut player, animations.animations[0], Duration::ZERO)
            .repeat();

        commands
            .entity(entity)
            .insert(AnimationGraphHandle(animations.graph.clone()))
            .insert(transitions);
    }
}