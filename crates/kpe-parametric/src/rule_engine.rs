use kpe_schema::block::RuleAction;
use kpe_schema::recipe::KPERecipe;

pub struct RuleEngine;

impl RuleEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn apply_rules(
        &self,
        recipe: &KPERecipe,
        active_rule_indices: &[(String, usize)],
    ) -> KPERecipe {
        let mut modified = recipe.clone();

        for (block_id, rule_idx) in active_rule_indices {
            let block = match modified.blocks.get(block_id) {
                Some(b) => b,
                None => continue,
            };

            let rule = match block.rules.get(*rule_idx) {
                Some(r) => r,
                None => continue,
            };

            for action in &rule.then {
                match action {
                    RuleAction::AddChild(child) => {
                        let child_node = kpe_schema::geometry::GeometryNode {
                            id: child.id.clone(),
                            node_type: kpe_schema::geometry::GeometryNodeType::Box(
                                kpe_schema::geometry::BoxDef {
                                    width: child.params.get("width")
                                        .and_then(|v| v.as_f64()).unwrap_or(0.0),
                                    height: child.params.get("height")
                                        .and_then(|v| v.as_f64()).unwrap_or(0.0),
                                    depth: child.params.get("depth")
                                        .and_then(|v| v.as_f64()).unwrap_or(0.0),
                                }
                            ),
                            transform: None,
                            children: vec![],
                            operations: vec![],
                            color: None,
                        };
                        modified.scene.children.push(child_node);
                    }
                    RuleAction::AddOperation(op) => {
                        let csg_op = kpe_schema::geometry::CsgOperation {
                            op_type: match op.op_type.as_str() {
                                "subtract" => kpe_schema::geometry::CsgOpType::Subtract,
                                "union" => kpe_schema::geometry::CsgOpType::Union,
                                "intersect" => kpe_schema::geometry::CsgOpType::Intersect,
                                _ => kpe_schema::geometry::CsgOpType::Subtract,
                            },
                            tool_id: op.tool.id.clone(),
                            tool_transform: op.tool.transform.clone(),
                        };
                        modified.scene.operations.push(csg_op);
                    }
                    RuleAction::SetParam(_set_param) => {
                        continue;
                    }
                }
            }
        }

        modified
    }
}

impl Default for RuleEngine {
    fn default() -> Self {
        Self::new()
    }
}
