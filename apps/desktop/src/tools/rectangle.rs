//! Rectangle tool — click‑click to create a `BoxDef` primitive.
//!
//! Flow:
//!   1. First click infers a construction plane (from face hit or ground).
//!   2. Second click defines the opposite corner.
//!   3. A `BoxDef { width, height: 0.01, depth }` node is created via
//!      `AddFeatureCommand`, centred on the two click points.

use bevy::prelude::*;
use bevy::render::primitives::Aabb;
use kpe_schema::geometry::{BoxDef, GeometryNode, GeometryNodeType};
use kpe_parametric::commands::AddFeatureCommand;

use crate::{app::AppState, sync::MeshNodeId, tools::core::*};

/// Rectangle tool system.
///
/// Activated when `BuildTool::Rectangle` is the active tool (`[R]` key).
pub fn rect_tool_system(
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
    if tool_state.active_tool != BuildTool::Rectangle {
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
            let Some(p1) = plane.intersect_ray(ray_o, ray_d) else {
                return;
            };
            tool_state.phase = ToolPhase::Placing {
                p1_world: p1,
                plane,
            };
        }
        ToolPhase::Placing { p1_world, plane } => {
            let Some(p2) = plane.intersect_ray(ray_o, ray_d) else {
                return;
            };
            let p2_p = plane.project(p2);
            let p1_2d = plane.to_2d(*p1_world);
            let p2_2d = plane.to_2d(p2_p);
            let min = p1_2d.min(p2_2d);
            let max = p1_2d.max(p2_2d);
            let width = (max.x - min.x) as f64;
            let depth = (max.y - min.y) as f64;
            if width < 0.01 || depth < 0.01 {
                tool_state.phase = ToolPhase::Idle;
                return;
            }
            let center_2d = (p1_2d + p2_2d) / 2.0;
            let center_w =
                plane.origin + plane.u_axis * center_2d.x + plane.v_axis * center_2d.y;

            let scene = state.document.to_scene();
            let counter = kpe_parametric::next_counter(&scene.scene, "Rect_");
            let node_id = format!("Rect_{:03}", counter);
            let node = GeometryNode {
                id: node_id.clone(),
                node_type: GeometryNodeType::Box(BoxDef {
                    width,
                    height: 0.01,
                    depth,
                }),
                transform: Some(kpe_schema::geometry::TransformOp {
                    translation: Some([center_w.x as f64, center_w.y as f64, center_w.z as f64]),
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
