use std::collections::HashMap;

pub struct ExpressionEvaluator;

impl ExpressionEvaluator {
    pub fn new() -> Self {
        Self
    }

    pub fn evaluate(
        &self,
        expr: &str,
        params: &HashMap<String, f64>,
        variables: &HashMap<String, f64>,
    ) -> Result<f64, String> {
        let expr = expr.trim();

        if let Ok(num) = expr.parse::<f64>() {
            return Ok(num);
        }

        if let Some(val) = params.get(expr) {
            return Ok(*val);
        }

        if let Some(val) = variables.get(expr) {
            return Ok(*val);
        }

        if expr.contains("params.") {
            let key = expr.strip_prefix("params.").unwrap_or(expr);
            if let Some(val) = params.get(key) {
                return Ok(*val);
            }
        }

        let simplified = expr
            .replace("Math.floor(", "f64_floor(")
            .replace("Math.ceil(", "f64_ceil(")
            .replace("Math.round(", "f64_round(")
            .replace("Math.sqrt(", "f64_sqrt(")
            .replace("Math.abs(", "f64_abs(");

        if simplified.contains('*') || simplified.contains('/')
            || simplified.contains('+') || simplified.contains('-')
            || simplified.contains('(') || simplified.contains(')')
        {
            let substituted = self.substitute(simplified, params, variables);
            return self.eval_arithmetic(&substituted);
        }

        Err(format!("Cannot evaluate expression: {expr}"))
    }

    fn substitute(
        &self,
        expr: String,
        params: &HashMap<String, f64>,
        variables: &HashMap<String, f64>,
    ) -> String {
        let mut result = expr;
        for (key, val) in params {
            let pattern = format!("params.{key}");
            result = result.replace(&pattern, &val.to_string());
        }
        for (key, val) in variables {
            result = result.replace(key, &val.to_string());
        }
        result
    }

    fn eval_arithmetic(&self, expr: &str) -> Result<f64, String> {
        let expr = expr
            .replace("f64_floor(", "floor(")
            .replace("f64_ceil(", "ceil(")
            .replace("f64_round(", "round(")
            .replace("f64_sqrt(", "sqrt(")
            .replace("f64_abs(", "abs(");

        let trimmed = expr.trim();
        if !trimmed.contains(' ') && !trimmed.contains('+') && !trimmed.contains('-')
            && !trimmed.contains('*') && !trimmed.contains('/') && !trimmed.contains('(')
        {
            return trimmed.parse::<f64>()
                .map_err(|_| format!("Cannot parse number: {trimmed}"));
        }

        meval::eval_str(expr)
            .map_err(|e| format!("Expression error: {e}"))
    }
}

impl Default for ExpressionEvaluator {
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
        m.insert("thickness".to_string(), 18.0);
        m.insert("height".to_string(), 2100.0);
        m
    }

    #[test]
    fn test_literal_number() {
        let eval = ExpressionEvaluator::new();
        assert_eq!(eval.evaluate("42.5", &HashMap::new(), &HashMap::new()).unwrap(), 42.5);
    }

    #[test]
    fn test_param_reference() {
        let eval = ExpressionEvaluator::new();
        let p = params();
        assert_eq!(eval.evaluate("params.width", &p, &HashMap::new()).unwrap(), 600.0);
    }

    #[test]
    fn test_simple_arithmetic() {
        let eval = ExpressionEvaluator::new();
        let p = params();
        let result = eval.evaluate("params.width - 2 * params.thickness", &p, &HashMap::new()).unwrap();
        assert!((result - 564.0).abs() < 1e-9);
    }

    #[test]
    fn test_variable_reference() {
        let eval = ExpressionEvaluator::new();
        let mut vars = HashMap::new();
        vars.insert("inner_width".to_string(), 564.0);
        assert_eq!(eval.evaluate("inner_width", &HashMap::new(), &vars).unwrap(), 564.0);
    }

    #[test]
    fn test_math_floor() {
        let eval = ExpressionEvaluator::new();
        let p = params();
        let result = eval.evaluate("Math.floor(params.height / 32)", &p, &HashMap::new()).unwrap();
        assert_eq!(result, 65.0);
    }

    #[test]
    fn test_division() {
        let eval = ExpressionEvaluator::new();
        let p = params();
        let result = eval.evaluate("params.width / 2.0", &p, &HashMap::new()).unwrap();
        assert_eq!(result, 300.0);
    }

    #[test]
    fn test_invalid_expr() {
        let eval = ExpressionEvaluator::new();
        assert!(eval.evaluate("undefined_var", &HashMap::new(), &HashMap::new()).is_err());
    }
}
