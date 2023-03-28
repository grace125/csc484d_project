
use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui::{self, Id, RichText, Color32}};

const TOP_PANEL_ID: &str = &"top menu";
const TOP_PANEL_SELECT_COLOUR: Color32 = Color32::WHITE;

#[derive(States, Default, Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum Mode {
    #[default]
    Edit,
    Settings,
    SaveLoad,
    Interact,
}

#[derive(SystemSet, Debug, Clone, Eq, PartialEq, Hash)]
pub enum UiSet {
    Main
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_state::<Mode>()
            .configure_set(UiSet::Main.in_base_set(CoreSet::Update))
            .add_system(top_menu.in_set(UiSet::Main))
            .add_system(edit_menu.in_set(OnUpdate(Mode::Edit)))
            .add_system(settings_menu.in_set(OnUpdate(Mode::Settings)))
            .add_system(save_load_menu.in_set(OnUpdate(Mode::SaveLoad)));
    }
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
                    let _ = ui.button(RichText::new(text).color(TOP_PANEL_SELECT_COLOUR));
                }
            }
        });
    });
}

fn settings_menu(
    mut _contexts: EguiContexts,
) {

}

fn edit_menu(
    mut _contexts: EguiContexts,
) {

}

fn save_load_menu(
    mut _contexts: EguiContexts,
) {

}
