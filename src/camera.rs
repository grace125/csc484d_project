use bevy::{prelude::*, input::mouse::MouseMotion};

use crate::{AppSet, graph::GraphSelection, ui::egui_unfocused};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .configure_sets((
                AppSet::CameraStartup.in_base_set(StartupSet::Startup),
                AppSet::Camera.in_base_set(CoreSet::Update)
            ))
            .add_startup_system(setup.in_set(AppSet::CameraStartup))
            .add_system((
                pan_camera
                    .run_if(not(resource_exists::<GraphSelection>()))
                    .run_if(egui_unfocused)
            ).in_set(AppSet::Camera));
    }
}

#[derive(Component)]
pub struct PrimaryCamera;

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle::default(),
        PrimaryCamera
    ));
}

fn pan_camera(
    mut motion_evr: EventReader<MouseMotion>,
    mut camera_pos: Query<&mut Transform, With<PrimaryCamera>>,
    input: Res<Input<MouseButton>>,
) {
    if input.pressed(MouseButton::Left) {
        let change: Vec2 = motion_evr.into_iter().map(|ev| ev.delta).sum();
        if change != Vec2::ZERO {
            camera_pos.single_mut().translation.x -= change.x;
            camera_pos.single_mut().translation.y += change.y;
    
        }
    }
}