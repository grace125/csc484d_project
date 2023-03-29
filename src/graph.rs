use bevy::{prelude::*, math::Vec3Swizzles, utils::HashMap,};
use bevy_prototype_lyon::{prelude::{ShapeBundle, GeometryBuilder}, shapes};

pub use bevy_prototype_lyon::prelude::Fill;

use crate::{AppSet, camera::PrimaryCamera, Mode, ui::egui_unfocused, helper::LastPrimaryCursorPos};

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
    input: Res<Input<MouseButton>>,
    last_cursor_move: Res<LastPrimaryCursorPos>,
    camera: Query<(&Camera, &GlobalTransform), With<PrimaryCamera>>,
    vertices: Query<(Entity, &GlobalTransform, &Vertex)>,
    // edges: Query<&GlobalTransform, With<Edge>>,
) {
    if input.just_pressed(MouseButton::Left) {
        let Some(last_cursor_pos) = last_cursor_move.0 else { 
            commands.remove_resource::<GraphSelection>();
            return; 
        };

        let (camera, camera_transform) = camera.single();

        let Some(click_pos) = camera.viewport_to_world_2d(camera_transform, last_cursor_pos) 
        else {
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

fn quantize(f: f32) -> f32 {
    (f/20.0).round() * 20.0
}

// TODO: make it so that clicking a node doesn't move it.
fn drag_node(
    mut transforms: Query<&mut Transform>,
    selection: Res<GraphSelection>,
    input: Res<Input<MouseButton>>,
    last_cursor_pos: Res<LastPrimaryCursorPos>,
    mut cursor_node_diff: Local<Option<Vec2>>,

    camera: Query<(&Camera, &GlobalTransform), With<PrimaryCamera>>,
) {
    if input.just_pressed(MouseButton::Left) {
        *cursor_node_diff = None;
    }
    else if input.pressed(MouseButton::Left) {
        let Ok(mut selection_transform) = transforms.get_mut(selection.entity) 
            else { return };
        let (camera, camera_transform) = camera.single();
        let Some(last_cursor_pos) = last_cursor_pos.0 else { return };
        let Some(world_cursor_pos) = camera.viewport_to_world_2d(camera_transform, last_cursor_pos) 
            else { return };

        if *cursor_node_diff == None {
            *cursor_node_diff = Some(world_cursor_pos - selection_transform.translation.xy());
        }

        let Some(cursor_node_diff) = *cursor_node_diff else { return };
        
        let new_transform = world_cursor_pos + cursor_node_diff;

        selection_transform.translation.x = quantize(new_transform.x);
        selection_transform.translation.y = quantize(new_transform.y);
    }
}


// fn on_added_vertex(
//     query: Query<(), Added<Vertex>>,
// ) {

// }