use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use std::collections::{HashMap, HashSet};
use crate::app::AppState;
use crate::commands;
use kpe_schema::geometry::TriangleMesh;

#[derive(Resource)]
pub struct MeshCache {
    pub handles: HashMap<String, Handle<Mesh>>,
    pub entities: HashMap<String, Entity>,
    pub materials: HashMap<String, Handle<StandardMaterial>>,
    pub last_gen: u64,
}

impl Default for MeshCache {
    fn default() -> Self {
        Self { handles: HashMap::new(), entities: HashMap::new(), materials: HashMap::new(), last_gen: 0 }
    }
}

#[derive(Component)]
pub struct SceneMeshRoot;

/// Maps a Bevy entity back to its scene node ID for selection
#[derive(Component)]
pub struct MeshNodeId(pub String);

pub fn setup_scene(mut commands: Commands) {
    commands.spawn((
        SceneMeshRoot,
        Visibility::default(),
        Transform::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
    ));
}

pub fn sync_meshes(
    state: Res<AppState>,
    mut cache: ResMut<MeshCache>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    root: Query<Entity, With<SceneMeshRoot>>,
    mut commands: Commands,
) {
    let root_entity = match root.get_single() {
        Ok(e) => e,
        _ => return,
    };

    if state.mesh_gen == cache.last_gen {
        return;
    }
    cache.last_gen = state.mesh_gen;

    let mut known: HashSet<String> = HashSet::new();
    let hidden = &state.document.hidden_nodes;

    for (id, tri_mesh) in &state.document.evaluated.meshes {
        if hidden.contains(id) { continue; }
        known.insert(id.clone());

        let node_color = commands::find_node(&state.document.recipe.scene, id)
            .and_then(|n| n.color.as_ref())
            .cloned();

        if let Some(handle) = cache.handles.get(id) {
            let bevy_mesh = kpe_mesh_to_bevy(tri_mesh);
            meshes.insert(handle.id(), bevy_mesh);
            // Update material color if changed
            if let Some(mat_handle) = cache.materials.get(id) {
                if let Some(mat) = materials.get_mut(mat_handle.id()) {
                    mat.base_color = hex_to_bevy_color(&node_color);
                }
            }
        } else {
            let bevy_mesh = kpe_mesh_to_bevy(tri_mesh);
            let handle = meshes.add(bevy_mesh);
            let mat_handle = match cache.materials.get(id) {
                Some(h) => h.clone(),
                None => {
                    let mat = materials.add(StandardMaterial {
                        base_color: hex_to_bevy_color(&node_color),
                        metallic: 0.1,
                        perceptual_roughness: 0.3,
                        ..default()
                    });
                    cache.materials.insert(id.clone(), mat.clone());
                    mat
                }
            };
            let child = commands.spawn((
                Mesh3d(handle.clone()),
                MeshMaterial3d(mat_handle.clone()),
                Visibility::default(),
                Transform::default(),
                MeshNodeId(id.clone()),
            )).id();
            commands.entity(root_entity).add_child(child);
            cache.handles.insert(id.clone(), handle);
            cache.entities.insert(id.clone(), child);
        }
    }

    // Despawn stale or hidden meshes
    let stale: Vec<String> = cache.handles.keys()
        .filter(|k| !known.contains(k.as_str()) || hidden.contains(*k))
        .cloned()
        .collect();
    for id in &stale {
        if let Some(entity) = cache.entities.remove(id) {
            commands.entity(entity).despawn_recursive();
        }
        cache.handles.remove(id);
    }
}

pub fn kpe_mesh_to_bevy(kpe_mesh: &TriangleMesh) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );

    let positions: Vec<[f32; 3]> = kpe_mesh
        .vertices
        .iter()
        .map(|v| [v[0] as f32, v[1] as f32, v[2] as f32])
        .collect();

    let normals: Vec<[f32; 3]> = if !kpe_mesh.normals.is_empty() {
        kpe_mesh.normals.iter().map(|n| [n[0] as f32, n[1] as f32, n[2] as f32]).collect()
    } else {
        vec![[0.0, 1.0, 0.0]; positions.len()]
    };

    let uvs: Vec<[f32; 2]> = if !kpe_mesh.uvs.is_empty() {
        kpe_mesh.uvs.iter().map(|u| [u[0] as f32, u[1] as f32]).collect()
    } else {
        vec![[0.0, 0.0]; positions.len()]
    };

    let indices: Vec<u32> = kpe_mesh.triangles.iter().flat_map(|t| [t[0], t[1], t[2]]).collect();

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));

    mesh.duplicate_vertices();
    mesh.compute_flat_normals();

    mesh
}

fn hex_to_bevy_color(hex: &Option<String>) -> Color {
    match hex {
        Some(h) if h.len() >= 6 => {
            let h = h.trim_start_matches('#');
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&h[0..2], 16),
                u8::from_str_radix(&h[2..4], 16),
                u8::from_str_radix(&h[4..6], 16),
            ) {
                return Color::srgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);
            }
            Color::srgb(0.25, 0.5, 0.9)
        }
        _ => Color::srgb(0.25, 0.5, 0.9),
    }
}
