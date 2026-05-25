use std::collections::HashMap;

pub struct ConditionEvaluator;

impl ConditionEvaluator {
    pub fn new() -> Self {
        Self
    }

    pub fn evaluate(
        &self,
        condition: &str,
        params: &HashMap<String, f64>,
        variables: &HashMap<String, f64>,
    ) -> bool {
        let substituted = self.substitute(condition, params, variables);
        self.eval_boolean(&substituted)
    }

    fn substitute(
        &self,
        expr: &str,
        params: &HashMap<String, f64>,
        variables: &HashMap<String, f64>,
    ) -> String {
        let mut result = expr.to_string();
        for (key, val) in params {
            let pattern = format!("params.{key}");
            result = result.replace(&pattern, &val.to_string());
        }
        for (key, val) in variables {
            result = result.replace(key, &val.to_string());
        }
        result
    }

    fn eval_boolean(&self, expr: &str) -> bool {
        let expr = expr.trim();

        if expr == "true" {
            return true;
        }
        if expr == "false" {
            return false;
        }

        let parts: Vec<&str> = expr.splitn(3, |c| c == '>' || c == '<'
            || c == '=' || c == '!').collect();

        if parts.len() < 3 {
            let op_pos = expr.find(|c: char| c == '>' || c == '<'
                || (c == '=' && expr.contains("=="))
                || (c == '!' && expr.contains("!=")));
            return op_pos.is_some();
        }

        let left = parts[0].trim().parse::<f64>().unwrap_or(0.0);
        let right = parts[2].trim().parse::<f64>().unwrap_or(0.0);

        if expr.contains(">=") { left >= right }
        else if expr.contains("<=") { left <= right }
        else if expr.contains("==") { (left - right).abs() < 1e-9 }
        else if expr.contains("!=") { (left - right).abs() > 1e-9 }
        else if expr.contains('>') { left > right }
        else if expr.contains('<') { left < right }
        else { false }
    }
}

impl Default for ConditionEvaluator {
    fn default() -> Self {
        Self::new()
    }
}
