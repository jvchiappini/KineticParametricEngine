import type { SketchModel } from "../../model/SketchModel";
import type { VerticalConstraint } from "../../model/Constraint";

function readVar(variables: Map<string, number>, key: string, fallback: number): number {
  const v = variables.get(key);
  return v !== undefined ? v : fallback;
}

/**
 * Evaluates a vertical constraint on a line.
 * Error = x2 - x1 (both endpoints at same x).
 * Gradient: d(error)/dx1 = -1, d(error)/dx2 = 1.
 */
export function evaluateConstraint(
  constraint: VerticalConstraint,
  model: SketchModel,
  variables: Map<string, number>,
  gradients: Map<string, number>
): number {
  const entity = model.entities.get(constraint.line);
  if (!entity || entity.kind !== "line") return 0;

  const start = model.points.get(entity.start);
  const end = model.points.get(entity.end);
  if (!start || !end) return 0;

  const x1 = readVar(variables, `${entity.start}.x`, start.x);
  const x2 = readVar(variables, `${entity.end}.x`, end.x);

  const error = x2 - x1;

  gradients.set(`${entity.start}.x`, (gradients.get(`${entity.start}.x`) ?? 0) - 1);
  gradients.set(`${entity.end}.x`, (gradients.get(`${entity.end}.x`) ?? 0) + 1);

  return error;
}
