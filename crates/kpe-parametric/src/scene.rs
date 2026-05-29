use kpe_schema::geometry::GeometryNode;
use kpe_schema::joint::Joint;

/// The mutable geometric state that parametric commands operate on.
///
/// This represents the abstract parametric model — a scene tree plus
/// joints — with no coupling to any rendering or UI framework.
#[derive(Debug, Clone)]
pub struct GeometryScene {
    pub scene: GeometryNode,
    pub joints: Vec<Joint>,
}

impl GeometryScene {
    /// Create an empty scene with a root Compound node.
    pub fn new() -> Self {
        Self {
            scene: GeometryNode {
                id: "Root".to_string(),
                node_type: kpe_schema::geometry::GeometryNodeType::Compound,
                transform: None,
                children: vec![],
                operations: vec![],
                color: None,
            },
            joints: vec![],
        }
    }
}

impl Default for GeometryScene {
    fn default() -> Self {
        Self::new()
    }
}
