use project::{*, graph::*};
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(AppPlugins)
        .add_startup_system(setup)
        .add_system(on_n_press)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(VertexBundle::new(
        (20.0, 10.0, 0.0).into(), 
        "Alice", 
        20.0
    ));
}

fn on_n_press(mut commands: Commands, input: Res<Input<KeyCode>>) {
    if input.just_pressed(KeyCode::N) {
        commands.spawn(VertexBundle::new(
            (0.0, 0.0, 0.0).into(), 
            "Alice", 
            20.0
        ));
    }
}