//! Plays animations from a skinned glTF.

use std::{f32::consts::PI, time::Duration};

use crate::player::{Player, PlayerAnimation};
use bevy::{
    animation::{AnimationTargetId, RepeatAnimation},
    color::palettes::css::WHITE,
    pbr::CascadeShadowConfigBuilder,
    prelude::*,
};

pub const FOX_PATH: &str = "RobotExpressive.glb";

pub struct FoxPlugin;

impl Plugin for FoxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, setup_scene_once_loaded)
            .add_systems(Update, switch_player_animation);
        // .add_observer(observe_on_step);
    }
}

#[derive(Resource)]
struct Animations {
    animations: Vec<AnimationNodeIndex>,
    graph: Handle<AnimationGraph>,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    // Build the animation graph
    let (graph, node_indices) =
        AnimationGraph::from_clips(PlayerAnimation::clips().map(|clip| asset_server.load(clip)));

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
            _ => {
                info!("gone gone");
                unreachable!()
            }
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

#[derive(Default, PartialEq)]
enum Queue {
    #[default]
    None,
    Next(PlayerAnimation),
}

fn switch_player_animation(
    time: Res<Time>,
    mut animation_players: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
    mut query: Query<(&mut Player)>,
    mut previous_animation: Local<PlayerAnimation>,
    mut current_animation: Local<PlayerAnimation>,
    mut queue: Local<Queue>,
    mut delay_timer: Local<Option<Timer>>,
) {
    // let player = query.single();

    'parent: for player in query.iter() {
        if player.current_animation == *previous_animation && *queue == Queue::None {
            continue;
        }

        for (mut animation_player, mut transitions) in &mut animation_players {
            match *queue {
                Queue::Next(animation) => {
                    if let Some(timer) = delay_timer.as_mut() {
                        timer.tick(time.delta());

                        if !timer.finished() {
                            continue 'parent;
                        }

                        *delay_timer = None;
                    }

                    transitions
                        .play(
                            &mut animation_player,
                            animation.to_animation(),
                            Duration::from_millis(250),
                        )
                        .repeat();

                    *previous_animation = current_animation.clone();
                    *queue = Queue::None;
                }
                Queue::None => {
                    *current_animation = player.current_animation.clone();

                    if player.current_animation == PlayerAnimation::Idle {
                        transitions.play(
                            &mut animation_player,
                            PlayerAnimation::Jumping.to_animation(),
                            Duration::from_millis(250),
                        );

                        *delay_timer = Some(Timer::from_seconds(0.708, TimerMode::Once));
                    }

                    *queue = Queue::Next(player.current_animation);
                }
            }
        }
    }
}

// fn keyboard_animation_control(
//     keyboard_input: Res<ButtonInput<KeyCode>>,
//     mut animation_players: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
//     animations: Res<Animations>,
//     mut current_animation: Local<usize>,
// ) {
//     for (mut player, mut transitions) in &mut animation_players {
//         let Some((&playing_animation_index, _)) = player.playing_animations().next() else {
//             continue;
//         };
//
//         if keyboard_input.just_pressed(KeyCode::Space) {
//             let playing_animation = player.animation_mut(playing_animation_index).unwrap();
//             if playing_animation.is_paused() {
//                 playing_animation.resume();
//             } else {
//                 playing_animation.pause();
//             }
//         }
//
//         if keyboard_input.just_pressed(KeyCode::ArrowUp) {
//             let playing_animation = player.animation_mut(playing_animation_index).unwrap();
//             let speed = playing_animation.speed();
//             playing_animation.set_speed(speed * 1.2);
//         }
//
//         if keyboard_input.just_pressed(KeyCode::ArrowDown) {
//             let playing_animation = player.animation_mut(playing_animation_index).unwrap();
//             let speed = playing_animation.speed();
//             playing_animation.set_speed(speed * 0.8);
//         }
//
//         if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
//             let playing_animation = player.animation_mut(playing_animation_index).unwrap();
//             let elapsed = playing_animation.seek_time();
//             playing_animation.seek_to(elapsed - 0.1);
//         }
//
//         if keyboard_input.just_pressed(KeyCode::ArrowRight) {
//             let playing_animation = player.animation_mut(playing_animation_index).unwrap();
//             let elapsed = playing_animation.seek_time();
//             playing_animation.seek_to(elapsed + 0.1);
//         }
//
//         if keyboard_input.just_pressed(KeyCode::Enter) {
//             *current_animation = (*current_animation + 1) % animations.animations.len();
//
//             transitions
//                 .play(
//                     &mut player,
//                     3.into(),
//                     Duration::from_millis(0),
//                 );
//
//             info!("Animation INDEX: {:?}", animations.animations[*current_animation]);
//
//             // transitions
//             //     .play(
//             //         &mut player,
//             //         animations.animations[*current_animation],
//             //         Duration::from_millis(250),
//             //     )
//             //     .repeat();
//         }
//
//         if keyboard_input.just_pressed(KeyCode::Digit1) {
//             let playing_animation = player.animation_mut(playing_animation_index).unwrap();
//             playing_animation
//                 .set_repeat(RepeatAnimation::Count(1))
//                 .replay();
//         }
//
//         if keyboard_input.just_pressed(KeyCode::Digit3) {
//             let playing_animation = player.animation_mut(playing_animation_index).unwrap();
//             playing_animation
//                 .set_repeat(RepeatAnimation::Count(3))
//                 .replay();
//         }
//
//         if keyboard_input.just_pressed(KeyCode::Digit5) {
//             let playing_animation = player.animation_mut(playing_animation_index).unwrap();
//             playing_animation
//                 .set_repeat(RepeatAnimation::Count(5))
//                 .replay();
//         }
//
//         if keyboard_input.just_pressed(KeyCode::KeyL) {
//             let playing_animation = player.animation_mut(playing_animation_index).unwrap();
//             playing_animation.set_repeat(RepeatAnimation::Forever);
//         }
//     }
// }
