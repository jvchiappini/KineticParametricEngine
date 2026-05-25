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
