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
