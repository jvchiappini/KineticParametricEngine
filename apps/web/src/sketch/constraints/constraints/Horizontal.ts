import type { SketchModel } from "../../model/SketchModel";
import type { HorizontalConstraint } from "../../model/Constraint";

function readVar(variables: Map<string, number>, key: string, fallback: number): number {
  const v = variables.get(key);
  return v !== undefined ? v : fallback;
}

/**
 * Evaluates a horizontal constraint on a line.
 * Error = y2 - y1 (both endpoints at same y).
 * Gradient: d(error)/dy1 = -1, d(error)/dy2 = 1.
 */
export function evaluateConstraint(
  constraint: HorizontalConstraint,
  model: SketchModel,
  variables: Map<string, number>,
  gradients: Map<string, number>
): number {
  const entity = model.entities.get(constraint.line);
  if (!entity || entity.kind !== "line") return 0;

  const start = model.points.get(entity.start);
  const end = model.points.get(entity.end);
  if (!start || !end) return 0;

  const y1 = readVar(variables, `${entity.start}.y`, start.y);
  const y2 = readVar(variables, `${entity.end}.y`, end.y);

  const error = y2 - y1;

  gradients.set(`${entity.start}.y`, (gradients.get(`${entity.start}.y`) ?? 0) - 1);
  gradients.set(`${entity.end}.y`, (gradients.get(`${entity.end}.y`) ?? 0) + 1);

  return error;
}
