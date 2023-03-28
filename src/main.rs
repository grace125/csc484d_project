use bevy::prelude::*;

use project::AppPlugins;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(AppPlugins)
        .run();
}