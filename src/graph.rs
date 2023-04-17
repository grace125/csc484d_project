use std::iter;

use bevy::{prelude::*, math::Vec3Swizzles, utils::{HashMap, HashSet}};
use bevy_prototype_lyon::{prelude::{ShapeBundle, GeometryBuilder, Stroke, StrokeOptions, Path, ShapePath}, shapes, plugin::BuildShapes};

pub use bevy_prototype_lyon::prelude::Fill;

use crate::{AppSet, camera::PrimaryCamera, Mode, ui::egui_unfocused, helper::LastPrimaryCursorPos};

pub struct GraphPlugin;

impl Plugin for GraphPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .configure_sets((
                AppSet::GraphStartup
                    .in_base_set(StartupSet::Startup),
                AppSet::GraphInteraction
                    .in_base_set(CoreSet::Update),
                AppSet::GraphManagement
                    .in_base_set(CoreSet::Update)
                    .after(AppSet::GraphInteraction)
                    .after(AppSet::Ui)
            ))
            .init_resource::<Graph>()
            .add_startup_system(setup.in_set(AppSet::GraphStartup))
            .add_systems((
                interaction::select,
                interaction::drag_node
                    .run_if(resource_exists::<GraphSelection>())
                    .run_if(not(state_exists_and_equals(Mode::Interact))),
                interaction::create_edge
                    .run_if(resource_exists::<GraphSelection>())
                    .run_if(state_exists_and_equals(Mode::Edit)),
            )
                .distributive_run_if(egui_unfocused)
                .chain()
                .in_set(AppSet::GraphInteraction)
            )
            .add_systems((
                graph_handle::on_vertex_change,
                graph_handle::on_edge_builder,
                graph_handle::on_vertex_position_change,
                graph_handle::on_edge_removal,
            )
                .chain()
                .in_set(AppSet::GraphManagement)
            );
    }
}

#[derive(Resource, Default, Debug)]
pub struct Graph {
    incident_edges: HashMap<Entity, HashSet<Entity>>,
    incident_vertices: HashMap<Entity, (Entity, Entity)>,
}

impl Graph {
    pub fn insert_vertex(&mut self, v: Entity) {
        assert!(!self.has_vertex(&v));
        self.incident_edges.insert(v, HashSet::new());
    }

    pub fn has_vertex(&self, v: &Entity) -> bool {
        self.incident_edges.contains_key(v)
    }

    pub fn remove_vertex(&mut self, v: &Entity) {
        for edge in self.incident_edges.remove(v).iter().flatten() {
            self.remove_edge(edge);
        }
    }

    pub fn insert_edge(&mut self, e: Entity, v1: Entity, v2: Entity) {
        assert!(self.has_vertex(&v1));
        assert!(self.has_vertex(&v2));
        assert!(!self.has_edge(&e));
        if let Some(edges) = self.incident_edges.get_mut(&v1) {
            edges.insert(e);
        }
        if let Some(edges) = self.incident_edges.get_mut(&v2) {
            edges.insert(e);
        }
        let (v1, v2) = (v1.min(v2), v1.max(v2));
        self.incident_vertices.insert(e, (v1, v2));
    }

    pub fn has_edge(&mut self, e: &Entity) -> bool {
        self.incident_vertices.contains_key(e)
    }

    pub fn remove_edge(&mut self, e: &Entity) {
        let Some((u, v)) = self.incident_vertices.remove(e) else {
            return;
        };
        if let Some(edges) = self.incident_edges.get_mut(&u) {
            edges.remove(&e);
        }
        if let Some(edges) = self.incident_edges.get_mut(&v) {
            edges.remove(&e);
        }
    }

    pub fn iter_edges(&self, v: &Entity) -> impl IntoIterator<Item = &Entity> {
        self.incident_edges.get(v).into_iter().flatten()
    }

    pub fn incident_vertices(&self, e: &Entity) -> Option<(Entity, Entity)> {
        self.incident_vertices.get(&e).copied()
    }

    pub fn get_edge_between(&self, v1: Entity, v2: Entity) -> Option<Entity> {
        let (v1, v2) = (v1.min(v2), v1.max(v2));
        if let Some(in_edges) = self.incident_edges.get(&v1) {
            for edge in self.iter_edges(&v2) {
                if in_edges.contains(edge) {
                    return Some(*edge);
                }
            }
        }
        None
    }
}

#[derive(Resource, Debug, Clone)]
pub enum GraphSelection {
    Vertex(Entity),
    Edge(Entity),
}

#[derive(Component, Default, Clone, Debug)]
pub struct Vertex;

#[derive(Component, Default, Clone, Debug)]
pub struct BlankVertex;

#[derive(Component, Default, Clone, Debug)]
pub struct VertexArea {
    half_extend: f32
}

impl VertexArea {
    pub fn intersects(&self, area_pos: Vec2, other_pos: Vec2) -> bool{
        let diff = (area_pos - other_pos).abs();
        diff.x < self.half_extend && diff.y < self.half_extend
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct VertexName(pub String);

#[derive(Bundle)]
pub struct VertexBundle {
    vertex: Vertex,
    vertex_area: VertexArea,
    vertex_name: VertexName,
    #[bundle]
    shape: ShapeBundle,
    colour: Fill,
}

impl VertexBundle {
    pub fn new(pos: Vec3, name: impl Into<String>, half_extend: f32) -> Self {
        let shape = shapes::RegularPolygon {
            sides: 4,
            feature: shapes::RegularPolygonFeature::SideLength(half_extend*2.0),
            ..default()
        };
        VertexBundle {
            vertex: Vertex,
            vertex_area: VertexArea { half_extend },
            vertex_name: VertexName(name.into()),
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

#[derive(Component)]
pub struct Edge;

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct EdgeBuilder {
    pub u: Entity,
    pub v: Entity,
}


#[derive(Component)]
struct DisplayCreationEdge;

fn setup(mut commands: Commands) {
    commands.spawn((
        DisplayCreationEdge,
        ShapeBundle {
            visibility: Visibility::Hidden,
            ..default()
        },
        Stroke {
            options: StrokeOptions::default().with_line_width(3.0),
            color: Color::BLACK,
        }
    ));
}

mod interaction {
    use super::*;

    pub(super) fn edge_collide(cursor_pos: Vec2, u_pos: Vec2, v_pos: Vec2) -> bool {
        let m = v_pos - u_pos;
        if m == Vec2::ZERO { return false; }
        let p = cursor_pos - u_pos;
        let proj_m_p = m.dot(p)/m.dot(m)*m;

        let cursor_between_vertices = {
            let len_squared = proj_m_p.length_squared();
            0.0 <= len_squared && len_squared <= m.length_squared()
        };
        let distance_from_edge_squared = (p - proj_m_p).length_squared();
        cursor_between_vertices && distance_from_edge_squared <= 25.0
    }

    pub(super) fn select(
        mut commands: Commands, 
        graph: Res<Graph>,
        input: Res<Input<MouseButton>>,
        last_cursor_move: Res<LastPrimaryCursorPos>,
        camera: Query<(&Camera, &GlobalTransform), With<PrimaryCamera>>,
        vertices: Query<(Entity, &GlobalTransform, &VertexArea), With<Vertex>>,
        edges: Query<Entity, With<Edge>>,
        transforms: Query<&GlobalTransform>,
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
            
            for (entity, transform, area) in vertices.iter() {
                let vertex_pos = transform.translation().xy();
                if area.intersects(vertex_pos, click_pos) {
                    commands.insert_resource(GraphSelection::Vertex(entity));
                    return;
                }
            }

            for entity in edges.iter() {
                if let Some((u, v)) = graph.incident_vertices(&entity) {
                    let Ok(u_transform) = transforms.get(u) else { continue };
                    let Ok(v_transform) = transforms.get(v) else { continue };
                    let u_pos = u_transform.translation().xy();
                    let v_pos = v_transform.translation().xy();

                    if edge_collide(click_pos, u_pos, v_pos) {
                        commands.insert_resource(GraphSelection::Edge(entity));
                        return;
                    }
                }
            }
    
            commands.remove_resource::<GraphSelection>();
        }
    }
    
    
    fn quantize(f: f32) -> f32 { (f/20.0).round() * 20.0 }
    fn quantize_vec2(v: Vec2) -> Vec2 { 
        let v_mod = v % 20.0;
        v - 2.0*v_mod 
    }
    
    // TODO: make it so that clicking a node doesn't move it.
    pub(super) fn drag_node(
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
            let GraphSelection::Vertex(selected_entity) = *selection 
                else { return };
            let Ok(mut selection_transform) = transforms.get_mut(selected_entity) 
                else { return };
            let (camera, camera_transform) = camera.single();
            let Some(last_cursor_pos) = last_cursor_pos.0 else { return };
            let Some(world_cursor_pos) = camera.viewport_to_world_2d(camera_transform, last_cursor_pos) 
                else { return };
    
            if *cursor_node_diff == None {
                *cursor_node_diff = Some(quantize_vec2(world_cursor_pos - selection_transform.translation.xy()));
            }
    
            let Some(cursor_node_diff) = *cursor_node_diff else { return };
            
            let new_transform = world_cursor_pos + cursor_node_diff;
    
            selection_transform.translation.x = quantize(new_transform.x);
            selection_transform.translation.y = quantize(new_transform.y);
        }
    }
    
    pub(super) fn create_edge(
        mut commands: Commands,
        input: Res<Input<KeyCode>>,
        selection: Res<GraphSelection>,
        last_cursor_pos: Res<LastPrimaryCursorPos>,
        mut display_edge: Query<(&mut Path, &mut Visibility), With<DisplayCreationEdge>>,
        transforms: Query<&Transform>,
        camera: Query<(&Camera, &GlobalTransform), With<PrimaryCamera>>,
        vertices: Query<(Entity, &GlobalTransform, &VertexArea), With<Vertex>>,
    ) {
        if input.pressed(KeyCode::E) {
            let GraphSelection::Vertex(selected_entity) = *selection else { return };
            let Some(last_cursor_pos) = last_cursor_pos.0 else { return };
            let Ok(selected_transform) = transforms.get(selected_entity) else { return };
            let world_cursor_pos = {
                let (camera, camera_transform) = camera.single();
                let Some(pos) = camera.viewport_to_world_2d(camera_transform, last_cursor_pos) else { return; };
                pos
            };
            let (mut path, mut visibility) = display_edge.single_mut();
            let start_pos = selected_transform.translation.xy();
    
            if input.just_pressed(KeyCode::E) {
                *visibility = Visibility::Visible;
            }
            *path = ShapePath::build_as(&shapes::Line(start_pos, world_cursor_pos));
        }
        else if input.just_released(KeyCode::E) {
            let (_, mut visibility) = display_edge.single_mut();
            *visibility = Visibility::Hidden;
    
            let Some(last_cursor_pos) = last_cursor_pos.0 else { return };
            let world_cursor_pos = {
                let (camera, camera_transform) = camera.single();
                let Some(pos) = camera.viewport_to_world_2d(camera_transform, last_cursor_pos) else { return; };
                pos
            };
    
            for (entity, trans, area) in vertices.iter() {
                let GraphSelection::Vertex(selected_entity) = *selection else { return };
                let vertex_pos = trans.translation().xy();
                if area.intersects(vertex_pos, world_cursor_pos) {
                    commands.spawn(EdgeBuilder {
                        u: selected_entity,
                        v: entity,
                    });
                    return;
                }
            }        
        }
    }

}

mod graph_handle {
    use super::*;

    pub(super) fn on_vertex_change(
        mut commands: Commands,
        mut graph: ResMut<Graph>,
        mut removed_vertices: RemovedComponents<Vertex>,
        added_vertices: Query<Entity, Added<Vertex>>,
    ) {
        for entity in added_vertices.iter() {
            graph.insert_vertex(entity);
        }
        for entity in removed_vertices.iter() {
            for edge in graph.iter_edges(&entity) {
                commands.entity(*edge).despawn();
            }
            graph.remove_vertex(&entity);

        }
    }

    pub(super) fn on_vertex_position_change(
        graph: Res<Graph>,
        vertices_with_changed_transforms: Query<Entity, (With<Vertex>, Changed<Transform>)>,
        transforms: Query<&Transform>,
        mut paths: Query<&mut Path>
    ) {
        for vertex in vertices_with_changed_transforms.iter() {
            for edge in graph.iter_edges(&vertex) {
                let Some((u, v)) = graph.incident_vertices(edge) else {continue };
                let Ok(mut path) = paths.get_mut(*edge) else { continue };
                
                let (u_pos, v_pos) = {
                    let Ok(u_transform) = transforms.get(u) else { continue };
                    let Ok(v_transform) = transforms.get(v) else {continue };

                    (u_transform.translation.xy(), v_transform.translation.xy())
                };

                *path = ShapePath::build_as(&shapes::Line(u_pos, v_pos));
            }
        }
    }

    pub(super) fn on_edge_builder(
        mut commands: Commands,
        mut graph: ResMut<Graph>,
        transforms: Query<&Transform>,
        added_edge_builders: Query<(Entity, &EdgeBuilder), Added<EdgeBuilder>>,
    ) {
        for (entity, edge_builder) in added_edge_builders.iter() {
            let mut entity_commands = commands.entity(entity);
            
            if edge_builder.u == edge_builder.v 
            || !graph.has_vertex(&edge_builder.u) 
            || !graph.has_vertex(&edge_builder.v)
            || graph.get_edge_between(edge_builder.u, edge_builder.v).is_some() {
                entity_commands.despawn(); 
                continue; 
            }

            entity_commands.remove::<EdgeBuilder>();
            
            graph.insert_edge(entity, edge_builder.u, edge_builder.v);

            let from_pos = transforms.get(edge_builder.u).unwrap().translation.xy();
            let to_pos = transforms.get(edge_builder.v).unwrap().translation.xy();
            
            entity_commands.insert((
                Edge,
                ShapeBundle {
                    path: ShapePath::build_as(&shapes::Line(from_pos, to_pos)),
                    ..default()
                },
                Stroke {
                    options: StrokeOptions::default().with_line_width(3.0),
                    color: Color::BLACK,
                }
            ));
        }
    }

    pub(super) fn on_edge_removal(
        mut removed_edges: RemovedComponents<Edge>,
        mut graph: ResMut<Graph>
    ) {
        for edge in removed_edges.iter() {
            graph.remove_edge(&edge);
        }
    }
}
