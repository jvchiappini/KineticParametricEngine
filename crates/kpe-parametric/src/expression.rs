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
            .replace("f64_floor(", "(")
            .replace("f64_ceil(", "(")
            .replace("f64_round(", "(")
            .replace("f64_sqrt(", "(")
            .replace("f64_abs(", "(");

        let tokens: Vec<&str> = expr.split_whitespace().collect();
        if tokens.len() == 1 {
            return tokens[0].parse::<f64>()
                .map_err(|_| format!("Cannot parse number: {}", tokens[0]));
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
