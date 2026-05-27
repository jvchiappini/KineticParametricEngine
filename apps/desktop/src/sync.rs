use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use std::collections::{HashMap, HashSet};
use crate::app::AppState;
use kpe_schema::geometry::TriangleMesh;

#[derive(Resource)]
pub struct MeshCache {
    pub handles: HashMap<String, Handle<Mesh>>,
    pub material: Handle<StandardMaterial>,
    pub last_gen: u64,
}

impl Default for MeshCache {
    fn default() -> Self {
        Self { handles: HashMap::new(), material: Handle::default(), last_gen: 0 }
    }
}

#[derive(Component)]
pub struct SceneMeshRoot;

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

    for (id, tri_mesh) in &state.document.evaluated.meshes {
        known.insert(id.clone());

        if let Some(handle) = cache.handles.get(id) {
            let bevy_mesh = kpe_mesh_to_bevy(tri_mesh);
            meshes.insert(handle.id(), bevy_mesh);
        } else {
            let bevy_mesh = kpe_mesh_to_bevy(tri_mesh);
            let handle = meshes.add(bevy_mesh);
            let mat_handle = if cache.material.id() == Handle::default().id() {
                let mat = materials.add(StandardMaterial {
                    base_color: Color::srgb(0.25, 0.5, 0.9),
                    metallic: 0.1,
                    perceptual_roughness: 0.3,
                    ..default()
                });
                cache.material = mat.clone();
                mat
            } else {
                cache.material.clone()
            };
            commands.entity(root_entity).with_children(|parent| {
                parent.spawn((
                    Mesh3d(handle.clone()),
                    MeshMaterial3d(mat_handle.clone()),
                    Visibility::default(),
                    Transform::default(),
                ));
            });
            cache.handles.insert(id.clone(), handle);
        }
    }

    // Remove stale meshes
    let stale: Vec<String> = cache.handles.keys()
        .filter(|k| !known.contains(k.as_str()))
        .cloned()
        .collect();
    for id in &stale {
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
