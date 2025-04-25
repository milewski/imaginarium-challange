use crate::robot::{Player, Robot};
use bevy::color::palettes::tailwind::GRAY_800;
use bevy::prelude::*;

pub struct UIPlugin;

#[derive(Component)]
struct CoordinateText;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, add_coordinate_to_screen_system);
        app.add_systems(Update, update_coordinate_system);
    }
}

fn update_coordinate_system(
    all_players: Query<&Robot>,
    main_player: Query<&Transform, With<Player>>,
    mut text_query: Query<&mut Text, With<CoordinateText>>,
) {
    for mut text in &mut text_query {
        let online_players = all_players.iter().count();
        if let Ok(transform) = main_player.get_single() {
            let x = transform.translation.x as i32;
            let z = transform.translation.z as i32;
            text.0 = format!("Online: {}\nAddress: {:}:{:}", online_players, x, z);
        }
    }
}

fn add_coordinate_to_screen_system(mut commands: Commands) {
    commands.spawn((
        CoordinateText,
        Text::new("0x0"),
        TextFont {
            // This font is loaded and will be used instead of the default font.
            font_size: 20.0,
            ..default()
        },
        TextColor(GRAY_800.into()),
        TextLayout::new_with_justify(JustifyText::Left),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
    ));
}