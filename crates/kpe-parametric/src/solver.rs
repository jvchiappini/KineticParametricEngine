use std::collections::HashMap;
use kpe_schema::recipe::{KPERecipe, ResolvedRecipe};
use crate::expression::ExpressionEvaluator;
use crate::condition::ConditionEvaluator;
use crate::rule_engine::RuleEngine;

#[derive(Debug)]
pub enum SolverError {
    CircularDependency(String),
    EvaluationError(String),
    UnknownParameter(String),
    UnknownVariable(String),
}

pub struct Solver {
    expression_eval: ExpressionEvaluator,
    condition_eval: ConditionEvaluator,
    _rule_engine: RuleEngine,
}

impl Solver {
    pub fn new() -> Self {
        Self {
            expression_eval: ExpressionEvaluator::new(),
            condition_eval: ConditionEvaluator::new(),
            _rule_engine: RuleEngine::new(),
        }
    }

    pub fn resolve(&self, recipe: &KPERecipe) -> Result<ResolvedRecipe, SolverError> {
        let mut resolved_params = HashMap::new();
        let mut resolved_variables = HashMap::new();

        for (block_id, block) in &recipe.blocks {
            let mut block_params = HashMap::new();
            for (param_name, schema) in &block.params {
                let val = schema.default.as_f64()
                    .ok_or_else(|| SolverError::EvaluationError(
                        format!("Non-numeric default for {block_id}.{param_name}")
                    ))?;
                block_params.insert(param_name.clone(), val);
            }

            let mut block_vars = HashMap::new();
            for (var_name, expr) in &block.variables {
                let resolved = self.expression_eval.evaluate(
                    expr, &block_params, &block_vars
                ).map_err(|e| SolverError::EvaluationError(e))?;
                block_vars.insert(var_name.clone(), resolved);
            }

            resolved_params.insert(block_id.clone(), block_params);
            resolved_variables.insert(block_id.clone(), block_vars);
        }

        let mut active_rules = Vec::new();
        for (block_id, block) in &recipe.blocks {
            let params = resolved_params.get(block_id)
                .ok_or_else(|| SolverError::UnknownParameter(block_id.clone()))?;
            let vars = resolved_variables.get(block_id)
                .ok_or_else(|| SolverError::UnknownVariable(block_id.clone()))?;

            for rule in &block.rules {
                if self.condition_eval.evaluate(&rule.when, params, vars) {
                    active_rules.push(format!("{block_id}: {}", rule.when));
                }
            }
        }

        Ok(ResolvedRecipe {
            recipe: recipe.clone(),
            resolved_params,
            resolved_variables,
            active_rules,
        })
    }
}

impl Default for Solver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kpe_schema::block::*;

    fn make_test_recipe() -> KPERecipe {
        let mut recipe = KPERecipe::default();
        let mut params = HashMap::new();
        params.insert("width".to_string(), ParamSchema {
            param_type: ParamType::Number,
            default: serde_json::json!(600.0),
            min: Some(200.0),
            max: Some(1200.0),
            unit: Some("mm".to_string()),
            options: None,
        });
        params.insert("thickness".to_string(), ParamSchema {
            param_type: ParamType::Number,
            default: serde_json::json!(18.0),
            min: Some(9.0),
            max: Some(36.0),
            unit: Some("mm".to_string()),
            options: None,
        });
        let mut variables = HashMap::new();
        variables.insert("inner_width".to_string(), "params.width - 2 * params.thickness".to_string());
        let mut rules = Vec::new();
        rules.push(Rule {
            when: "params.width > 800".to_string(),
            then: vec![],
        });
        recipe.blocks.insert("panel".to_string(), BlockDefinition {
            id: "panel".to_string(),
            label: "Panel".to_string(),
            params,
            variables,
            rules,
            joints: HashMap::new(),
            geometry: None,
            material: None,
        });
        recipe
    }

    #[test]
    fn test_solver_resolves_params() {
        let solver = Solver::new();
        let recipe = make_test_recipe();
        let resolved = solver.resolve(&recipe).unwrap();
        let panel_params = resolved.resolved_params.get("panel").unwrap();
        assert!((panel_params["width"] - 600.0).abs() < 1e-9);
        assert!((panel_params["thickness"] - 18.0).abs() < 1e-9);
    }

    #[test]
    fn test_solver_resolves_variables() {
        let solver = Solver::new();
        let recipe = make_test_recipe();
        let resolved = solver.resolve(&recipe).unwrap();
        let panel_vars = resolved.resolved_variables.get("panel").unwrap();
        assert!((panel_vars["inner_width"] - 564.0).abs() < 1e-9);
    }

    #[test]
    fn test_solver_rule_inactive() {
        let solver = Solver::new();
        let recipe = make_test_recipe();
        let resolved = solver.resolve(&recipe).unwrap();
        assert!(resolved.active_rules.is_empty());
    }

    #[test]
    fn test_solver_rule_active_when_condition_met() {
        let solver = Solver::new();
        let mut recipe = make_test_recipe();
        if let Some(block) = recipe.blocks.get_mut("panel") {
            block.params.get_mut("width").unwrap().default = serde_json::json!(1000.0);
        }
        let resolved = solver.resolve(&recipe).unwrap();
        assert_eq!(resolved.active_rules.len(), 1);
    }
}
