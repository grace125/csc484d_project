use bevy::{prelude::*, input::InputSystem, window::PrimaryWindow};

pub struct HelperPlugin;

impl Plugin for HelperPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LastPrimaryCursorPos>()
            .add_system(update_last_cursor_move.in_base_set(CoreSet::PreUpdate).after(InputSystem));
    }
}

#[derive(Resource, Default)]
pub struct LastPrimaryCursorPos(pub Option<Vec2>);

fn update_last_cursor_move(
    mut last_cursor_move: ResMut<LastPrimaryCursorPos>, 
    mut ev: EventReader<CursorMoved>,
    prime_window: Query<(), With<PrimaryWindow>>,
) {
    if let Some(m) = ev.into_iter().filter(|ev| prime_window.contains(ev.window)).last() { 
        last_cursor_move.0 = Some(m.position);
    };
}