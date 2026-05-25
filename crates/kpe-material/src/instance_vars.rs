use std::collections::HashMap;
use kpe_schema::material::{InstanceVarDef, InstanceVarType};

pub struct InstanceVarResolver;

impl InstanceVarResolver {
    pub fn new() -> Self {
        Self
    }

    pub fn resolve(
        &self,
        defs: &HashMap<String, InstanceVarDef>,
        overrides: &HashMap<String, serde_json::Value>,
    ) -> HashMap<String, serde_json::Value> {
        let mut result = HashMap::new();

        for (name, def) in defs {
            let value = match overrides.get(name) {
                Some(val) => val.clone(),
                None => match &def.var_type {
                    InstanceVarType::RandomInt => {
                        let range = def.range.unwrap_or([0, 9999]);
                        let seed = self.hash(name) % (range[1] - range[0] + 1) + range[0];
                        serde_json::Value::Number(seed.into())
                    }
                    InstanceVarType::String => {
                        def.default.clone().unwrap_or(serde_json::Value::String(String::new()))
                    }
                },
            };
            result.insert(name.clone(), value);
        }

        result
    }

    fn hash(&self, s: &str) -> i64 {
        let mut h: i64 = 0;
        for b in s.bytes() {
            h = h.wrapping_mul(31).wrapping_add(b as i64);
        }
        h.abs()
    }
}

impl Default for InstanceVarResolver {
    fn default() -> Self {
        Self::new()
    }
}
