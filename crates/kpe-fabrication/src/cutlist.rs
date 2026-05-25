use kpe_schema::fabrication::{CutPiece, CutList, GrainDirection};
use kpe_schema::geometry::GeometryNode;

pub struct CutListGenerator;

impl CutListGenerator {
    pub fn new() -> Self {
        Self
    }

    pub fn generate(&self, scene: &GeometryNode) -> CutList {
        let mut pieces = Vec::new();
        let mut id_counter = 0u32;

        self.collect_pieces(scene, &mut pieces, &mut id_counter);

        let total_pieces: u32 = pieces.iter().map(|p| p.quantity).sum();

        CutList {
            pieces,
            total_pieces,
        }
    }

    fn collect_pieces(
        &self,
        node: &GeometryNode,
        pieces: &mut Vec<CutPiece>,
        counter: &mut u32,
    ) {
        *counter += 1;
        let id = format!("piece_{}", counter);

        use kpe_schema::geometry::GeometryNodeType;
        if let GeometryNodeType::Box(box_def) = &node.node_type {
            pieces.push(CutPiece {
                id,
                label: node.id.clone(),
                width: box_def.width.max(box_def.depth),
                height: box_def.height,
                thickness: box_def.width.min(box_def.depth),
                quantity: 1,
                material_id: "default".to_string(),
                grain_direction: GrainDirection::Lengthwise,
                surface_area: 2.0 * (
                    box_def.width * box_def.height
                    + box_def.width * box_def.depth
                    + box_def.height * box_def.depth
                ),
            });
        }

        for child in &node.children {
            self.collect_pieces(child, pieces, counter);
        }
    }
}

impl Default for CutListGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kpe_schema::geometry::{GeometryNode, GeometryNodeType, BoxDef};

    #[test]
    fn test_generates_cutlist_from_box() {
        let generator = CutListGenerator::new();
        let scene = GeometryNode {
            id: "test".to_string(),
            node_type: GeometryNodeType::Box(BoxDef { width: 100.0, height: 200.0, depth: 50.0 }),
            transform: None,
            children: vec![],
            operations: vec![],
        };
        let cutlist = generator.generate(&scene);
        assert_eq!(cutlist.pieces.len(), 1);
        assert_eq!(cutlist.pieces[0].width, 100.0);
        assert_eq!(cutlist.pieces[0].height, 200.0);
    }

    #[test]
    fn test_generates_cutlist_from_compound() {
        let generator = CutListGenerator::new();
        let scene = GeometryNode {
            id: "root".to_string(),
            node_type: GeometryNodeType::Compound,
            transform: None,
            children: vec![
                GeometryNode {
                    id: "child1".to_string(),
                    node_type: GeometryNodeType::Box(BoxDef { width: 50.0, height: 30.0, depth: 10.0 }),
                    transform: None,
                    children: vec![],
                    operations: vec![],
                },
                GeometryNode {
                    id: "child2".to_string(),
                    node_type: GeometryNodeType::Box(BoxDef { width: 20.0, height: 40.0, depth: 10.0 }),
                    transform: None,
                    children: vec![],
                    operations: vec![],
                },
            ],
            operations: vec![],
        };
        let cutlist = generator.generate(&scene);
        assert_eq!(cutlist.pieces.len(), 2);
    }
}
