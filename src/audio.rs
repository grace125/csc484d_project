use std::time::Duration;

use bevy::{prelude::{Plugin, World, Resource, Deref, DerefMut, IntoSystemSetConfig, StartupSet, IntoSystemConfig}, tasks::IoTaskPool};
use knyst::{
    audio_backend::{CpalBackend, CpalBackendOptions}, 
    prelude::{AudioBackend, Graph, GraphSettings, RunGraphSettings}, 
    controller::KnystCommands
};

use crate::AppSet;


pub struct KnystAudioPlugin;

impl Plugin for KnystAudioPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app .configure_set(AppSet::AudioStartup.in_base_set(StartupSet::Startup))
            .add_startup_system(setup_knyst_graph.in_set(AppSet::AudioStartup));
    }
}

fn setup_knyst_graph(world: &mut World) {
    let mut backend = CpalBackend::new(CpalBackendOptions::default()).expect("Error in cpal audio backend");
    let num_outputs = backend.num_outputs();
    let sample_rate = backend.sample_rate() as f32;
    let block_size = backend.block_size().unwrap_or(64);
    let resources = knyst::Resources::new(knyst::ResourcesSettings::default());
    let graph = Graph::new(GraphSettings { block_size, sample_rate, num_outputs, ..Default::default()});
    let mut controller = backend
        .start_processing_return_controller(
            graph, 
            resources, 
            RunGraphSettings {
                ..Default::default()
            }, 
            knyst::controller::print_error_handler
        ).expect("Error in audio backend");
    let commands = controller.get_knyst_commands();
    
    IoTaskPool::get().spawn(async move {
        loop {
            while !controller.run(300) {}
            std::thread::sleep(Duration::from_micros(1));
        }
    }).detach();

    let commands = AudioCommands(commands);

    world.insert_resource(commands);
    world.insert_non_send_resource(backend);
}

#[derive(Resource, Deref, DerefMut)]
pub struct AudioCommands(pub KnystCommands);
