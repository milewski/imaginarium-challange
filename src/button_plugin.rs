use bevy::app::{App, Plugin, Update};
use bevy::math::Vec2;
use bevy::prelude::*;
use bevy_sprite3d::{Sprite3dBuilder, Sprite3dParams};
use crate::{get_random_between, AssetsCache};
use crate::player::Player;

pub struct ButtonPlugin;

#[derive(Resource, Default)]
pub struct Draggable {
    pub last_position: Option<(Vec2, Vec2)>,
}

impl Plugin for ButtonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, button_system);
    }
}

fn button_system(
    mut commands: Commands,
    assets: Res<AssetsCache>,
    mut sprite_params: Sprite3dParams,

    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut Text>,
    mut player_query: Query<&Player>,
) {
    let player = player_query.single();

    for (interaction, mut color, mut border_color, children) in &mut interaction_query {
        let mut text = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Pressed => {
                **text = "Press".to_string();


                commands.spawn((
                    Sprite3dBuilder {
                        image: assets.0[0].clone(),
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
                            player.current_position.0,
                            0.0,
                            player.current_position.1,
                        ),
                        rotation: Quat::from_rotation_y(45f32.to_radians()),
                        ..default()
                    },
                ));


            }
            Interaction::Hovered => {
                **text = "Hover".to_string();
            }
            Interaction::None => {
                **text = "Button".to_string();
            }
        }
    }
}

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::End,
            justify_content: JustifyContent::End,
            right: Val::Px(10.0),
            bottom: Val::Px(10.0),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(150.0),
                        height: Val::Px(65.0),
                        border: UiRect::all(Val::Px(5.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderColor(Color::BLACK),
                    BorderRadius::MAX,
                    BackgroundColor(NORMAL_BUTTON),
                ))
                .with_child((Text::new("Button"), TextColor(Color::srgb(0.9, 0.9, 0.9))));
        });
}
