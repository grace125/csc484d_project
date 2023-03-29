use bevy::prelude::*;
use bevy::math::Vec3Swizzles;

use crate::helper::LastPrimaryCursorPos;
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
    mut camera: Query<(&Camera, &mut Transform), With<PrimaryCamera>>,
    camera_transform: Query<&mut GlobalTransform, With<PrimaryCamera>>,
    input: Res<Input<MouseButton>>,
    last_cursor_pos: Res<LastPrimaryCursorPos>,
    mut camera_diff: Local<Vec2>,  
    mut camera_click_global_transform: Local<GlobalTransform>,  

) {
    if input.just_pressed(MouseButton::Left) {
        *camera_diff = Vec2::ZERO;
        *camera_click_global_transform = camera_transform.single().clone();
    }
    else if input.pressed(MouseButton::Left) {
        let Some(last_cursor_pos) = last_cursor_pos.0 else { return };
        let (camera, mut camera_transform) = camera.single_mut();

        let Some(world_cursor_pos) = camera.viewport_to_world_2d(&*camera_click_global_transform, last_cursor_pos)
        else { return; };

        if *camera_diff == Vec2::ZERO {
            *camera_diff = camera_transform.translation.xy() + world_cursor_pos;
        }

        let new_transform = *camera_diff - world_cursor_pos;
        camera_transform.translation.x = new_transform.x;
        camera_transform.translation.y = new_transform.y;
    }
}