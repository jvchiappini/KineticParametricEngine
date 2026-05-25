# Parametric Solver — Architecture

## Overview

The parametric solver resolves a `KPERecipe` from symbolic/parametric form
to concrete values. It is the first stage of the geometry pipeline.

## Input / Output

```
Input:  KPERecipe (symbolic)
        {
          params: { width: { type: "number", default: 600 } },
          variables: { inner_width: "params.width - 2 * params.thickness" },
          rules: [{ when: "params.width > 800", then: [...] }]
        }

Output: ResolvedRecipe (concrete)
        {
          resolved_params: { side_panel: { width: 600, thickness: 18, ... } },
          resolved_variables: { side_panel: { inner_width: 564.0 } },
          active_rules: ["side_panel: params.width > 800"],
          recipe: KPERecipe (with rules applied to scene tree)
        }
```

## Pipeline

```
KPERecipe
  │
  ▼
ExpressionEvaluator::evaluate(expr, params, vars)
  │
  ├── 1. Literal number? → return f64
  ├── 2. params.key? → return f64
  ├── 3. variables.key? → return f64
  ├── 4. Contains arithmetic? → substitute params/vars, eval via meval
  └── 5. Fallback → error
  │
  ▼
ConditionEvaluator::evaluate(condition, params, vars)
  │
  ├── 1. Substitute params/vars into expression
  ├── 2. Parse comparison operator (>, <, >=, <=, ==, !=)
  ├── 3. Evaluate both sides
  └── 4. Return boolean
  │
  ▼
RuleEngine::apply_rules(recipe, active_rule_indices)
  │
  ├── For each active rule:
  │     ├── add_child → push GeometryNode to scene.children
  │     ├── add_operation → push CsgOperation to scene.operations
  │     └── set_param → override param value
  │
  ▼
ResolvedRecipe
```

## Resolver Algorithm

```
For each block in recipe.blocks:
  1. For each param in block.params:
       Read default value → store as f64
       (future: clamp to min/max, validate enum)

  2. For each variable in block.variables:
       Substitute known param/variable references
       Evaluate arithmetic expression
       Handle Math.* functions:
         Math.floor, Math.ceil, Math.round, Math.sqrt, Math.abs

  3. For each rule in block.rules:
       Evaluate condition (substitute params + vars)
       If true → record as active rule

  4. Apply active rules to produce modified KPERecipe
```

## Dependency Graph

Variables may reference other variables. The resolver assumes the user
has defined them in topological order (no circular dependencies). If a
circular dependency is detected, `SolverError::CircularDependency` is
returned.

```
Valid:                              Invalid (cycle):
  width = 600                         a = b + 1
  inner_width = width - 36            b = a + 2
  panel_count = Math.floor(h / 300)
```

## Expression Grammar (current)

```
value       := number
             | "params." ident
             | variable_name
             | value operator value
             | Math.floor "(" value ")"
             | Math.ceil  "(" value ")"
             | Math.sqrt  "(" value ")"
             | "(" value ")"

operator    := "+" | "-" | "*" | "/"

condition   := value ">"  value
             | value "<"  value
             | value ">=" value
             | value "<=" value
             | value "==" value
             | value "!=" value
             | "true" | "false"
```

## Modules

| File | Responsibility |
|------|---------------|
| `solver.rs` | Orchestrates the pipeline, produces `ResolvedRecipe` |
| `expression.rs` | Evaluates math expressions, substitutes params/vars |
| `condition.rs` | Evaluates boolean conditions (comparisons, logical) |
| `rule_engine.rs` | Applies rule actions (add_child, add_operation, set_param) |
| `catalog.rs` | Registry of custom + industry standard blocks |

## Future Improvements

- Support logical operators in conditions (`&&`, `||`)
- Allow variables to reference variables from other blocks
- Add unit conversion (mm ↔ cm ↔ inches)
- Parameter UI hints (sliders, dropdowns) derived from `ParamSchema`
- Incremental re-resolution (only re-evaluate changed params)
