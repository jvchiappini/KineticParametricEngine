use crate::commands::CommandHistory;
use crate::document::Document;
use crate::ui;
use bevy::prelude::*;
use bevy_egui::EguiContexts;
use kpe_schema::geometry::{BoxDef, GeometryNode, GeometryNodeType};

#[derive(Resource)]
pub struct AppState {
    pub document: Document,
    pub history: CommandHistory,
    pub mesh_gen: u64,
}

impl AppState {
    pub fn new() -> Self {
        let mut doc = Document::new();
        doc.recipe.scene = make_default_scene();
        doc.evaluate_all();
        doc.selection = Some("Box_001".to_string());
        Self { document: doc, history: CommandHistory::new(), mesh_gen: 1 }
    }

    pub fn mark_dirty(&mut self) {
        self.mesh_gen += 1;
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

fn make_default_scene() -> GeometryNode {
    GeometryNode {
        id: "Root".to_string(),
        node_type: GeometryNodeType::Compound,
        transform: None,
        children: vec![
            GeometryNode {
                id: "Box_001".to_string(),
                node_type: GeometryNodeType::Box(BoxDef { width: 3.0, height: 3.0, depth: 3.0 }),
                transform: None,
                children: vec![],
                operations: vec![],
            },
        ],
        operations: vec![],
    }
}

pub fn ui_system(mut contexts: EguiContexts, mut state: ResMut<AppState>) {
    ui::toolbar::show(&mut contexts, &mut *state);
    ui::scene_tree::show(&mut contexts, &mut *state);
    ui::properties::show(&mut contexts, &mut *state);
    ui::status_bar::show(&mut contexts, &mut *state);
}

pub fn default_scene() -> GeometryNode {
    make_default_scene()
}
