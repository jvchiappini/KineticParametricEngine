import type { SketchModel } from "../../model/SketchModel";
import type { FixedConstraint } from "../../model/Constraint";

/**
 * Evaluates a fixed constraint.
 * Fixed constraints are handled by excluding the constrained entity's
 * dependent variables from the solver's variable list. This evaluator
 * is a no-op that always returns 0 error.
 */
export function evaluateConstraint(
  _constraint: FixedConstraint,
  _model: SketchModel,
  _variables: Map<string, number>,
  _gradients: Map<string, number>
): number {
  return 0;
}
