use crate::robot::{Player, PlayerKind, Robot};
use bevy::color::palettes::tailwind::GRAY_800;
use bevy::prelude::*;
use bevy::utils::tracing::metadata::Kind;
use shared::PlayerData;
use crate::js_bridge_plugin::{JSBridgeMessages, SendJsBridgeMessage};

#[derive(Resource, Default)]
pub struct UiInputBlocker(pub bool);

pub struct UIPlugin;

#[derive(Component)]
struct CoordinateText;

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(UiInputBlocker::default());

        app.add_systems(Startup, add_coordinate_to_screen_system);
        app.add_systems(Startup, add_build_monument_button_system);
        app.add_systems(Update, update_coordinate_system);
        app.add_systems(Update, update_balance_system);
        app.add_systems(Update, handle_build_monument_button_state_system);
        app.add_systems(Update, reset_ui_blocker.before(handle_build_monument_button_state_system));
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

fn update_balance_system(
    mut button_query: Query<&Children, With<Button>>,
    mut text_query: Query<&mut Text>,
    mut player_query: Query<&PlayerKind, (With<Player>, Changed<PlayerKind>)>,
) {
    if let Ok(metadata) = player_query.get_single() {
        let balance = match metadata {
            PlayerKind::MainPlayer(PlayerData { balance, .. }) => balance,
            PlayerKind::Enemy(..) => unreachable!(),
        };

        let children = button_query.single();

        if let Ok(mut text) = text_query.get_mut(children[1]) {
            **text = format!("Tokens ({})", balance);
        }
    }
}

pub fn handle_build_monument_button_state_system(
    mut interaction_query: Query<(&Interaction, &mut BackgroundColor, &Children), (Changed<Interaction>, With<Button>)>,
    mut event: EventWriter<SendJsBridgeMessage>,
    mut blocker: ResMut<UiInputBlocker>,
) {
    for (interaction, mut background_color, children) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                blocker.0 = true;
                event.send(SendJsBridgeMessage(JSBridgeMessages::CallOpenModal));
            }
            Interaction::Hovered => {
                background_color.0 = HOVERED_BUTTON;
            }
            Interaction::None => {
                background_color.0 = NORMAL_BUTTON;
            }
        }
    }
}

fn add_build_monument_button_system(mut commands: Commands) {
    let mut button = commands.spawn(Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        align_items: AlignItems::End,
        justify_content: JustifyContent::End,
        right: Val::Px(10.0),
        bottom: Val::Px(10.0),
        ..default()
    });

    button.with_children(|parent| {
        let mut parent = parent.spawn((
            Button,
            Node {
                width: Val::Px(150.0),
                height: Val::Px(65.0),
                border: UiRect::all(Val::Px(5.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BorderRadius::all(Val::Px(16.0)),
            BackgroundColor(NORMAL_BUTTON),
        ));

        parent.with_child((
            Text::new("Build"),
            TextFont::default().with_font_size(24.0),
            TextColor(Color::srgb(0.9, 0.9, 0.9))
        ));

        parent.with_child((
            Text::new("Tokens (0)"),
            TextFont::default().with_font_size(10.0),
            TextColor(Color::srgb(0.9, 0.9, 0.9))
        ));
    });
}

fn reset_ui_blocker(mut blocker: ResMut<UiInputBlocker>) {
    blocker.0 = false;
}