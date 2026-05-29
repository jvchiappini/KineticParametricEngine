use crate::commands::Command;
use crate::scene::GeometryScene;

/// A bounded undo/redo stack for parametric commands.
pub struct CommandHistory {
    pub undo_stack: Vec<Box<dyn Command>>,
    pub redo_stack: Vec<Box<dyn Command>>,
    pub max_undo: usize,
}

impl CommandHistory {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_undo: 50,
        }
    }

    /// Execute a command, pushing it onto the undo stack and clearing redo.
    pub fn execute(&mut self, mut cmd: Box<dyn Command>, scene: &mut GeometryScene) {
        cmd.execute(scene);
        self.undo_stack.push(cmd);
        self.redo_stack.clear();
        if self.undo_stack.len() > self.max_undo {
            self.undo_stack.remove(0);
        }
    }

    /// Undo the most recent command.
    pub fn undo(&mut self, scene: &mut GeometryScene) {
        if let Some(mut cmd) = self.undo_stack.pop() {
            cmd.undo(scene);
            self.redo_stack.push(cmd);
        }
    }

    /// Redo the most recently undone command.
    pub fn redo(&mut self, scene: &mut GeometryScene) {
        if let Some(mut cmd) = self.redo_stack.pop() {
            cmd.execute(scene);
            self.undo_stack.push(cmd);
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Undo description for the top of the undo stack.
    pub fn undo_description(&self) -> Option<&str> {
        self.undo_stack.last().map(|c| c.description())
    }

    /// Redo description for the top of the redo stack.
    pub fn redo_description(&self) -> Option<&str> {
        self.redo_stack.last().map(|c| c.description())
    }
}

impl Default for CommandHistory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kpe_schema::geometry::{BoxDef, GeometryNode, GeometryNodeType};
    use crate::commands::AddFeatureCommand;

    #[test]
    fn test_history_execute_undo_redo() {
        let mut scene = GeometryScene::new();
        let mut history = CommandHistory::new();
        let node = GeometryNode {
            id: "Box_001".into(),
            node_type: GeometryNodeType::Box(BoxDef {
                width: 1.0,
                height: 1.0,
                depth: 1.0,
            }),
            transform: None,
            children: vec![],
            operations: vec![],
            color: None,
        };
        let cmd = AddFeatureCommand {
            parent_id: "Root".into(),
            node,
        };

        assert!(!history.can_undo());
        history.execute(Box::new(cmd), &mut scene);
        assert!(history.can_undo());
        assert!(!history.can_redo());

        history.undo(&mut scene);
        assert!(history.can_redo());
        assert!(!history.can_undo());

        history.redo(&mut scene);
        assert!(history.can_undo());
        assert!(!history.can_redo());
    }
}
