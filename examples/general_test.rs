use project::{*, graph::*};
use bevy::{prelude::*, ecs::schedule::{ScheduleBuildSettings, LogLevel}};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(AppPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .edit_schedule(CoreSchedule::Main, |schedule| {
            schedule.set_build_settings(ScheduleBuildSettings {
                ambiguity_detection: LogLevel::Warn,
                hierarchy_detection: LogLevel::Ignore,
                use_shortnames: true,
                report_sets: true,
            });
        })
        .add_startup_system(setup)
        .add_system(on_n_press)
        .add_system(on_graph_change)
        .run();
}

fn setup(mut _commands: Commands) {

}

fn on_n_press(mut commands: Commands, input: Res<Input<KeyCode>>) {
    if input.just_pressed(KeyCode::N) {
        commands.spawn((
            VertexBundle::new((0.0, 0.0, 1.0).into(), "Alice", 20.0),
            BlankVertex
        ));
    }
}

fn on_graph_change(graph: Res<Graph>) {
    if graph.is_changed() {
        println!("{:?}", *graph);
    }
}