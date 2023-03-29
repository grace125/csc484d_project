use bevy::{prelude::*, math::Vec3Swizzles, window::PrimaryWindow, utils::HashMap, input::mouse::MouseMotion};
use bevy_prototype_lyon::{prelude::{ShapeBundle, GeometryBuilder}, shapes};

pub use bevy_prototype_lyon::prelude::Fill;

use crate::{AppSet, camera::PrimaryCamera, Mode, ui::egui_unfocused};

pub struct GraphPlugin;

impl Plugin for GraphPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .configure_sets((
                AppSet::GraphStartup.in_base_set(StartupSet::Startup),
                AppSet::Graph.in_base_set(CoreSet::Update)
            ))
            .init_resource::<AppGraph>()
            .add_startup_system(setup.in_set(AppSet::GraphStartup))
            .add_systems((
                select
                    .run_if(egui_unfocused),
                drag_node
                    .run_if(resource_exists::<GraphSelection>())
                    .run_if(egui_unfocused)
                    .run_if(not(state_exists_and_equals(Mode::Interact)))
                )
                .chain()
                .in_set(AppSet::Graph));
    }
}

fn setup() {
    
}

#[derive(Resource, Default)]
pub struct AppGraph {
    in_neighbours: HashMap<Entity, Vec<Entity>>,
    out_neighbours: HashMap<Entity, Vec<Entity>>,
    arc_incidence: HashMap<Entity, (Entity, Entity)>
}

#[derive(Component, Default, Clone, Debug)]
pub struct Vertex {
    half_extends: f32,
}

impl Vertex {
    pub fn intersects(&self, area_pos: Vec2, other_pos: Vec2) -> bool{
        let diff = (area_pos - other_pos).abs();
        diff.x < self.half_extends && diff.y < self.half_extends
    }
}

#[derive(Component, Default, Clone, Debug)]
pub struct Edge;

#[derive(Component, Clone, Debug)]
#[component(storage = "SparseSet")]
pub struct EdgeFromTo(pub Entity, pub Entity);

#[derive(Bundle)]
pub struct VertexBundle {
    vertex: Vertex,
    #[bundle]
    shape: ShapeBundle,
    colour: Fill,
}

impl VertexBundle {
    pub fn new(pos: Vec3, name: impl Into<String>, half_extends: f32) -> Self {
        let shape = shapes::RegularPolygon {
            sides: 4,
            feature: shapes::RegularPolygonFeature::SideLength(half_extends*2.0),
            ..default()
        };
        VertexBundle {
            vertex: Vertex {
                half_extends
            },
            shape: ShapeBundle {
                path: GeometryBuilder::build_as(&shape),
                transform: Transform {
                    translation: pos,
                    ..default()
                },
                ..default()
            },
            colour: Fill::color(Color::WHITE),
        }
    }
}

#[derive(Bundle)]
pub struct EdgeBundle {
    edge: Edge,
    #[bundle]
    shape: ShapeBundle,

}

// When it exists, there is a selected vertex/edge of the graph.
#[derive(Resource)]
pub(crate) struct GraphSelection {
    pub entity: Entity,
    pub is_vertex: bool
}

// TODO: this is stupid: 
// Shouldn't have to iterate over all vertices and edges, should use bevy spatial
// Change this when bevy spatial updates to 0.10.
fn select(
    mut commands: Commands, 
    mut cursor_moved_ev: EventReader<CursorMoved>,
    input: Res<Input<MouseButton>>,
    mut mouse_pos: Local<Option<CursorMoved>>,
    primary_window: Query<(), With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<PrimaryCamera>>,
    vertices: Query<(Entity, &GlobalTransform, &Vertex)>,
    // edges: Query<&GlobalTransform, With<Edge>>,
) {
    if let Some(moved) = cursor_moved_ev.into_iter().last() { 
        *mouse_pos = Some(moved.clone());
    };
    
    if input.just_pressed(MouseButton::Left) {

        let (camera, camera_transform) = camera.single();

        let Some(CursorMoved { window, position: mouse_position }) = *mouse_pos else { 
            commands.remove_resource::<GraphSelection>();
            return; 
        };

        if !primary_window.contains(window) {
            commands.remove_resource::<GraphSelection>();
            return; 
        }

        let Some(click_pos) = camera.viewport_to_world_2d(camera_transform, mouse_position) else {
            commands.remove_resource::<GraphSelection>();
            return; 
        };
        
        for (entity, trans, area) in vertices.iter() {
            let vertex_pos = trans.translation().xy();
            if area.intersects(vertex_pos, click_pos) {
                commands.insert_resource(GraphSelection {
                    entity,
                    is_vertex: true
                });
                return
            }
        }

        commands.remove_resource::<GraphSelection>();
    }
}


fn drag_node(
    mut motion_evr: EventReader<MouseMotion>,
    mut transforms: Query<&mut Transform>,
    selection: Res<GraphSelection>,
    input: Res<Input<MouseButton>>,
    mut drag_pos: Local<Option<Vec2>>,
) {
    if selection.is_changed() {
        *drag_pos = None;
    }
    if input.pressed(MouseButton::Left) {
        let Ok(mut transform) = transforms.get_mut(selection.entity) else {
            return
        };

        if *drag_pos == None {
            *drag_pos = Some(transform.translation.xy());
        }

        let Some(drag_pos) = &mut *drag_pos else { return };
        
        let delta: Vec2 = motion_evr.into_iter().map(|ev| ev.delta).sum();

        drag_pos.x += delta.x;
        drag_pos.y -= delta.y;
        transform.translation.x = (drag_pos.x/20.0).round() * 20.0;
        transform.translation.y = (drag_pos.y/20.0).round() * 20.0;
    }
}


// fn on_added_vertex(
//     query: Query<(), Added<Vertex>>,
// ) {

// }