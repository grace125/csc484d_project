
use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui::{self, Id}};

use crate::{AppSet, graph::GraphSelection};

const TOP_PANEL_ID: usize = 0;
const SETTING_PANEL_ID: usize = 1;
const EDIT_PANEL_ID: usize = 2;
const SAVE_LOAD_PANEL_ID: usize = 3;

#[derive(States, Default, Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum Mode {
    #[default]
    Edit,
    Settings,
    SaveLoad,
    Interact,
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_state::<Mode>()
            .init_resource::<EguiHover>()
            .configure_sets((
                AppSet::UiStartup.in_base_set(StartupSet::Startup),
                AppSet::Ui.in_base_set(CoreSet::Update),
            ))
            .add_startup_system(setup.in_set(AppSet::UiStartup))
            .add_systems((
                top_menu,
                edit_menu
                    .run_if(resource_exists::<GraphSelection>())
                    .run_if(state_exists_and_equals(Mode::Edit)),
                settings_menu
                    .run_if(state_exists_and_equals(Mode::Settings)),
                save_load_menu
                    .run_if(state_exists_and_equals(Mode::SaveLoad))
            )
                .chain()
                .in_set(AppSet::Ui)
            )
            .add_system(update_egui_hover.in_base_set(CoreSet::Last));
    }
}

fn setup() {

}

fn top_menu(
    mut contexts: EguiContexts, 
    mut next_mode: ResMut<NextState<Mode>>,
    mode: Res<State<Mode>>,
) {
    egui::TopBottomPanel::top(Id::new(TOP_PANEL_ID)).show(contexts.ctx_mut(), |ui| {
        egui::menu::bar(ui, |ui| {
            for (text, new_mode) in [
                ("Settings", Mode::Settings),
                ("Edit", Mode::Edit),
                ("Save/Load", Mode::SaveLoad),
                ("Interact", Mode::Interact),
            ] {
                if mode.0 != new_mode {
                    if ui.button(text).clicked() {
                        next_mode.0 = Some(new_mode);
                        break;
                    }
                }
                else {
                    let _ = ui.add_enabled(false, egui::Button::new(text));
                }
            }
        });
    });
}

fn settings_menu(
    mut contexts: EguiContexts,
) {
    egui::SidePanel::left(Id::new(SETTING_PANEL_ID)).show(contexts.ctx_mut(), |ui| {
        ui.label("Settings: TODO");
    });
}

fn edit_menu(
    mut commands: Commands,
    mut contexts: EguiContexts,
    selection: Res<GraphSelection>,
) {
    egui::SidePanel::left(Id::new(EDIT_PANEL_ID)).show(contexts.ctx_mut(), |ui| {
        ui.label(format!("{:?}", selection));
        let entity = match *selection { GraphSelection::Edge(e) | GraphSelection::Vertex(e) => e };
        if ui.button("Delete").clicked() {
            commands.entity(entity).despawn();
            commands.remove_resource::<GraphSelection>()
        }
    });
}

fn save_load_menu(
    mut contexts: EguiContexts,
) {
    egui::SidePanel::left(Id::new(SAVE_LOAD_PANEL_ID)).show(contexts.ctx_mut(), |ui| {
        ui.label("Save/Load: TODO");
    });

}

#[derive(Resource, Default)]
pub struct EguiHover(bool);

fn update_egui_hover(mut selected: ResMut<EguiHover>, mut contexts: EguiContexts) {
    selected.0 = contexts.ctx_mut().is_pointer_over_area()
}

// Run condition for whether a mouse's position is over an egui window
pub fn egui_unfocused(egui_hover: Res<EguiHover>) -> bool {
    !egui_hover.0
}