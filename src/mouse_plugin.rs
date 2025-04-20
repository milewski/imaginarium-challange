use bevy::app::{App, Plugin, Update};
use bevy::input::ButtonInput;
use bevy::math::{Vec2, Vec3};
use bevy::prelude::{MouseButton, Query, Res, ResMut, Resource, Transform, Window};

pub struct MousePlugin;

#[derive(Resource, Default)]
pub struct Draggable {
    pub last_position: Option<(Vec2, Vec2)>,
}

impl Draggable {
    pub fn apply(&self, transform: &mut Transform) {
        if let Some((_, delta)) = self.last_position {
            let right = transform.rotation * Vec3::X;
            let up = transform.rotation * Vec3::Y;

            transform.translation -= right * delta.x * 0.01;
            transform.translation += up * delta.y * 0.01;
        }
    }
}

impl Plugin for MousePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, on_update)
            .insert_resource(Draggable::default());
    }
}

fn on_update(
    mouse: Res<ButtonInput<MouseButton>>,
    mut windows: Query<&mut Window>,
    mut drag: ResMut<Draggable>,
) {
    let window = windows.single();

    if let Some(current_position) = window.cursor_position() {
        if mouse.pressed(MouseButton::Left) {
            if let Some((last_position, _)) = drag.last_position {
                drag.last_position = Some((current_position, current_position - last_position));
            } else {
                drag.last_position = Some((current_position, Vec2::ZERO));
            }
        } else {
            drag.last_position = None;
        }
    }
}
