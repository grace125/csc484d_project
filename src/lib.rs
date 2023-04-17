pub mod ui;
pub mod graph;
pub mod camera;
pub mod helper;
mod audio;

pub use audio::*;
use helper::HelperPlugin;
pub use ui::Mode;

use bevy::{prelude::{PluginGroup, SystemSet}, app::PluginGroupBuilder};
use bevy_egui::EguiPlugin;
use bevy_prototype_lyon::prelude::ShapePlugin;

#[derive(SystemSet, Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppSet {
    AudioStartup,
    UiStartup,
    CameraStartup,
    GraphStartup,
    Ui,
    Camera,
    GraphInteraction,
    GraphManagement,
}

pub struct AppPlugins;

impl PluginGroup for AppPlugins {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(KnystAudioPlugin)
            .add(ShapePlugin)
            .add(HelperPlugin)
            .add(EguiPlugin)
            .add(ui::UiPlugin)
            .add(graph::GraphPlugin)
            .add(camera::CameraPlugin)
    }
}
