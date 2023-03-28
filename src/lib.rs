mod audio;
pub use audio::*;

pub mod ui;
pub use ui::Mode;

use bevy::{prelude::PluginGroup, app::PluginGroupBuilder};
use bevy_egui::EguiPlugin;

pub struct AppPlugins;

impl PluginGroup for AppPlugins {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(KnystAudioPlugin)
            .add(EguiPlugin)
            .add(ui::UiPlugin)
    }
}