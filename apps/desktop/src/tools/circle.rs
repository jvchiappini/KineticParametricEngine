//! Circle tool — click‑click to create a `CylinderDef` primitive.
//!
//! Flow:
//!   1. First click sets the centre and infers a construction plane.
//!   2. Second click defines the radius.
//!   3. A `CylinderDef { radius, height: 0.01, segments: 32 }` node is created
//!      via `AddFeatureCommand`.

use bevy::prelude::*;
use bevy::render::primitives::Aabb;
use kpe_schema::geometry::{CylinderDef, GeometryNode, GeometryNodeType};
use kpe_parametric::commands::AddFeatureCommand;

use crate::{app::AppState, sync::MeshNodeId, tools::core::*};

/// Circle tool system.
///
/// Activated when `BuildTool::Circle` is the active tool (`[C]` key).
pub fn circle_tool_system(
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<crate::camera::OrbitCamera>>,
    mouse: Res<ButtonInput<MouseButton>>,
    meshes_query: Query<(&MeshNodeId, &GlobalTransform, &Aabb)>,
    mut state: ResMut<AppState>,
    mut tool_state: ResMut<BuildToolState>,
    editor: Res<crate::sketch_editor::SketchEditorState>,
) {
    if editor.active {
        return;
    }
    if tool_state.active_tool != BuildTool::Circle {
        return;
    }
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Some((ray_o, ray_d)) = get_ray(&windows, &cameras) else {
        return;
    };
    let window = match windows.get_single() {
        Ok(w) => w,
        _ => return,
    };
    let cursor = match window.cursor_position() {
        Some(c) => c,
        None => return,
    };
    if !in_viewport(cursor, window) {
        return;
    }

    match &tool_state.phase {
        ToolPhase::Idle => {
            let plane = infer_plane(ray_o, ray_d, &meshes_query, &state);
            let Some(center) = plane.intersect_ray(ray_o, ray_d) else {
                return;
            };
            tool_state.phase = ToolPhase::Placing {
                p1_world: center,
                plane,
            };
        }
        ToolPhase::Placing { p1_world, plane } => {
            let Some(cp) = plane.intersect_ray(ray_o, ray_d) else {
                return;
            };
            let cp_p = plane.project(cp);
            let radius = (cp_p - *p1_world).length() as f64;
            if radius < 0.01 {
                tool_state.phase = ToolPhase::Idle;
                return;
            }

            let scene = state.document.to_scene();
            let counter = kpe_parametric::next_counter(&scene.scene, "Circle_");
            let node_id = format!("Circle_{:03}", counter);
            let node = GeometryNode {
                id: node_id.clone(),
                node_type: GeometryNodeType::Cylinder(CylinderDef {
                    radius,
                    height: 0.01,
                    segments: 32,
                }),
                transform: Some(kpe_schema::geometry::TransformOp {
                    translation: Some([p1_world.x as f64, p1_world.y as f64, p1_world.z as f64]),
                    rotation: None,
                    scale: None,
                    pivots: Vec::new(),
                }),
                children: vec![],
                operations: vec![],
                color: None,
            };
            let cmd = AddFeatureCommand {
                parent_id: "Root".to_string(),
                node,
            };
            state.execute(Box::new(cmd));
            state.document.selection = Some(node_id);
            tool_state.phase = ToolPhase::Idle;
        }
        _ => {}
    }
}
