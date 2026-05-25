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

        let ops = [">=", "<=", "==", "!=", ">", "<"];
        let mut found_op = None;
        let mut op_pos = None;

        for op in &ops {
            if let Some(pos) = expr.find(op) {
                found_op = Some(*op);
                op_pos = Some(pos);
                break;
            }
        }

        let (op, pos) = match (found_op, op_pos) {
            (Some(op), Some(pos)) => (op, pos),
            _ => return false,
        };

        let left_str = expr[..pos].trim();
        let right_str = expr[pos + op.len()..].trim();

        let left = left_str.parse::<f64>().unwrap_or(0.0);
        let right = right_str.parse::<f64>().unwrap_or(0.0);

        match op {
            ">=" => left >= right,
            "<=" => left <= right,
            "==" => (left - right).abs() < 1e-9,
            "!=" => (left - right).abs() > 1e-9,
            ">"  => left > right,
            "<"  => left < right,
            _ => false,
        }
    }
}

impl Default for ConditionEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params() -> HashMap<String, f64> {
        let mut m = HashMap::new();
        m.insert("width".to_string(), 600.0);
        m.insert("height".to_string(), 2100.0);
        m
    }

    #[test]
    fn test_greater_than_true() {
        let eval = ConditionEvaluator::new();
        assert!(eval.evaluate("params.width > 500", &params(), &HashMap::new()));
    }

    #[test]
    fn test_greater_than_false() {
        let eval = ConditionEvaluator::new();
        assert!(!eval.evaluate("params.width > 700", &params(), &HashMap::new()));
    }

    #[test]
    fn test_less_than() {
        let eval = ConditionEvaluator::new();
        assert!(eval.evaluate("params.width < 700", &params(), &HashMap::new()));
    }

    #[test]
    fn test_equals() {
        let eval = ConditionEvaluator::new();
        assert!(eval.evaluate("params.width == 600", &params(), &HashMap::new()));
    }

    #[test]
    fn test_not_equals() {
        let eval = ConditionEvaluator::new();
        assert!(eval.evaluate("params.width != 500", &params(), &HashMap::new()));
    }

    #[test]
    fn test_greater_equal() {
        let eval = ConditionEvaluator::new();
        assert!(eval.evaluate("params.width >= 600", &params(), &HashMap::new()));
        assert!(eval.evaluate("params.width >= 500", &params(), &HashMap::new()));
        assert!(!eval.evaluate("params.width >= 700", &params(), &HashMap::new()));
    }

    #[test]
    fn test_boolean_literal() {
        let eval = ConditionEvaluator::new();
        assert!(eval.evaluate("true", &HashMap::new(), &HashMap::new()));
        assert!(!eval.evaluate("false", &HashMap::new(), &HashMap::new()));
    }
}
