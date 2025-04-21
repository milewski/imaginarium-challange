//! Plays animations from a skinned glTF.

use std::{f32::consts::PI, time::Duration};

use crate::player::{Player, PlayerAnimation};
use bevy::{
    animation::{AnimationTargetId, RepeatAnimation},
    color::palettes::css::WHITE,
    pbr::CascadeShadowConfigBuilder,
    prelude::*,
};

const FOX_PATH: &str = "RobotExpressive.glb";

pub struct FoxPlugin;

impl Plugin for FoxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, setup_scene_once_loaded)
            .add_systems(
                Update,
                (keyboard_animation_control, switch_player_animation),
            );
        // .add_observer(observe_on_step);
    }
}

#[derive(Resource)]
struct Animations {
    animations: Vec<AnimationNodeIndex>,
    graph: Handle<AnimationGraph>,
}

#[derive(Event, Reflect, Clone)]
struct OnStep;

// fn observe_on_step(
//     trigger: Trigger<OnStep>,
//     particle: Res<ParticleAssets>,
//     mut commands: Commands,
//     transforms: Query<&GlobalTransform>,
// ) {
//     let translation = transforms.get(trigger.entity()).unwrap().translation();
//     let mut rng = thread_rng();
//     // Spawn a bunch of particles.
//     for _ in 0..14 {
//         let horizontal = rng.gen::<Dir2>() * rng.gen_range(8.0..12.0);
//         let vertical = rng.gen_range(0.0..4.0);
//         let size = rng.gen_range(0.2..1.0);
//         commands.queue(spawn_particle(
//             particle.mesh.clone(),
//             particle.material.clone(),
//             translation.reject_from_normalized(Vec3::Y),
//             rng.gen_range(0.2..0.6),
//             size,
//             Vec3::new(horizontal.x, vertical, horizontal.y) * 10.0,
//         ));
//     }
// }

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    // Build the animation graph
    let (graph, node_indices) = AnimationGraph::from_clips([
        asset_server.load(GltfAssetLabel::Animation(13).from_asset(FOX_PATH)), // yes
        asset_server.load(GltfAssetLabel::Animation(12).from_asset(FOX_PATH)), // hi
        asset_server.load(GltfAssetLabel::Animation(11).from_asset(FOX_PATH)), // hop
        asset_server.load(GltfAssetLabel::Animation(10).from_asset(FOX_PATH)), // walk
        asset_server.load(GltfAssetLabel::Animation(9).from_asset(FOX_PATH)),  // like
        asset_server.load(GltfAssetLabel::Animation(8).from_asset(FOX_PATH)),
        asset_server.load(GltfAssetLabel::Animation(7).from_asset(FOX_PATH)),
        asset_server.load(GltfAssetLabel::Animation(6).from_asset(FOX_PATH)), // running
        asset_server.load(GltfAssetLabel::Animation(5).from_asset(FOX_PATH)),
        asset_server.load(GltfAssetLabel::Animation(4).from_asset(FOX_PATH)),
        asset_server.load(GltfAssetLabel::Animation(3).from_asset(FOX_PATH)),
        asset_server.load(GltfAssetLabel::Animation(2).from_asset(FOX_PATH)),
        asset_server.load(GltfAssetLabel::Animation(1).from_asset(FOX_PATH)),
        asset_server.load(GltfAssetLabel::Animation(0).from_asset(FOX_PATH)),
    ]);

    // Insert a resource with the current scene information
    let graph_handle = graphs.add(graph);
    commands.insert_resource(Animations {
        animations: node_indices,
        graph: graph_handle,
    });

    // Fox
    commands.spawn((
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(FOX_PATH))),
        Transform {
            scale: Vec3::splat(0.5),
            ..default()
        },
        Player::default(),
    ));

    println!("Animation controls:");
    println!("  - spacebar: play / pause");
    println!("  - arrow up / down: speed up / slow down animation playback");
    println!("  - arrow left / right: seek backward / forward");
    println!("  - digit 1 / 3 / 5: play the animation <digit> times");
    println!("  - L: loop the animation forever");
    println!("  - return: change animation");
}

// An `AnimationPlayer` is automatically added to the scene when it's ready.
// When the player is added, start the animation.
fn setup_scene_once_loaded(
    mut commands: Commands,
    animations: Res<Animations>,
    graphs: Res<Assets<AnimationGraph>>,
    mut clips: ResMut<Assets<AnimationClip>>,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) {
    fn get_clip<'a>(
        node: AnimationNodeIndex,
        graph: &AnimationGraph,
        clips: &'a mut Assets<AnimationClip>,
    ) -> &'a mut AnimationClip {
        let node = graph.get(node).unwrap();
        let clip = match &node.node_type {
            AnimationNodeType::Clip(handle) => clips.get_mut(handle),
            _ => unreachable!(),
        };
        clip.unwrap()
    }

    for (entity, mut player) in &mut players {
        let mut transitions = AnimationTransitions::new();

        // Make sure to start the animation via the `AnimationTransitions`
        // component. The `AnimationTransitions` component wants to manage all
        // the animations and will get confused if the animations are started
        // directly via the `AnimationPlayer`.
        transitions
            .play(&mut player, animations.animations[0], Duration::ZERO)
            .repeat();

        commands
            .entity(entity)
            .insert(AnimationGraphHandle(animations.graph.clone()))
            .insert(transitions);
    }
}

fn switch_player_animation(
    mut animation_players: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
    mut query: Query<(&mut Player)>,
    mut previous_animation: Local<PlayerAnimation>,
) {
    let player = query.single();

    if player.current_animation == *previous_animation {
        return;
    }

    for (mut animation_player, mut transitions) in &mut animation_players {
        let (index, duration) = player.current_animation.to_animation();



        transitions
            .play(&mut animation_player, index, duration)
            .repeat();

        *previous_animation = player.current_animation.clone()
    }
}

fn keyboard_animation_control(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut animation_players: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
    animations: Res<Animations>,
    mut current_animation: Local<usize>,
) {
    for (mut player, mut transitions) in &mut animation_players {
        let Some((&playing_animation_index, _)) = player.playing_animations().next() else {
            continue;
        };

        if keyboard_input.just_pressed(KeyCode::Space) {
            let playing_animation = player.animation_mut(playing_animation_index).unwrap();
            if playing_animation.is_paused() {
                playing_animation.resume();
            } else {
                playing_animation.pause();
            }
        }

        if keyboard_input.just_pressed(KeyCode::ArrowUp) {
            let playing_animation = player.animation_mut(playing_animation_index).unwrap();
            let speed = playing_animation.speed();
            playing_animation.set_speed(speed * 1.2);
        }

        if keyboard_input.just_pressed(KeyCode::ArrowDown) {
            let playing_animation = player.animation_mut(playing_animation_index).unwrap();
            let speed = playing_animation.speed();
            playing_animation.set_speed(speed * 0.8);
        }

        if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
            let playing_animation = player.animation_mut(playing_animation_index).unwrap();
            let elapsed = playing_animation.seek_time();
            playing_animation.seek_to(elapsed - 0.1);
        }

        if keyboard_input.just_pressed(KeyCode::ArrowRight) {
            let playing_animation = player.animation_mut(playing_animation_index).unwrap();
            let elapsed = playing_animation.seek_time();
            playing_animation.seek_to(elapsed + 0.1);
        }

        if keyboard_input.just_pressed(KeyCode::Enter) {
            *current_animation = (*current_animation + 1) % animations.animations.len();

            transitions
                .play(
                    &mut player,
                    3.into(),
                    Duration::from_millis(0),
                );

            info!("Animation INDEX: {:?}", animations.animations[*current_animation]);

            // transitions
            //     .play(
            //         &mut player,
            //         animations.animations[*current_animation],
            //         Duration::from_millis(250),
            //     )
            //     .repeat();
        }

        if keyboard_input.just_pressed(KeyCode::Digit1) {
            let playing_animation = player.animation_mut(playing_animation_index).unwrap();
            playing_animation
                .set_repeat(RepeatAnimation::Count(1))
                .replay();
        }

        if keyboard_input.just_pressed(KeyCode::Digit3) {
            let playing_animation = player.animation_mut(playing_animation_index).unwrap();
            playing_animation
                .set_repeat(RepeatAnimation::Count(3))
                .replay();
        }

        if keyboard_input.just_pressed(KeyCode::Digit5) {
            let playing_animation = player.animation_mut(playing_animation_index).unwrap();
            playing_animation
                .set_repeat(RepeatAnimation::Count(5))
                .replay();
        }

        if keyboard_input.just_pressed(KeyCode::KeyL) {
            let playing_animation = player.animation_mut(playing_animation_index).unwrap();
            playing_animation.set_repeat(RepeatAnimation::Forever);
        }
    }
}
