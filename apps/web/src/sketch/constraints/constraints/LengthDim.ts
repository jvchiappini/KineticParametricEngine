import type { SketchModel } from "../../model/SketchModel";
import type { LengthDimConstraint } from "../../model/Constraint";

function readVar(variables: Map<string, number>, key: string, fallback: number): number {
  const v = variables.get(key);
  return v !== undefined ? v : fallback;
}

function getCoords(
  model: SketchModel,
  variables: Map<string, number>,
  pointId: string
): { x: number; y: number } {
  const pt = model.points.get(pointId);
  if (!pt) return { x: 0, y: 0 };
  return {
    x: readVar(variables, `${pointId}.x`, pt.x),
    y: readVar(variables, `${pointId}.y`, pt.y)
  };
}

/**
 * Evaluates a length dimension constraint on a line.
 * Error = current length - target length.
 */
export function evaluateConstraint(
  constraint: LengthDimConstraint,
  model: SketchModel,
  variables: Map<string, number>,
  gradients: Map<string, number>
): number {
  const ent = model.entities.get(constraint.line);
  if (!ent || ent.kind !== "line") return 0;

  const s = getCoords(model, variables, ent.start);
  const e = getCoords(model, variables, ent.end);

  const dx = e.x - s.x;
  const dy = e.y - s.y;
  const len = Math.sqrt(dx * dx + dy * dy + 1e-10);

  const error = len - constraint.value;

  const gx = dx / len;
  const gy = dy / len;

  gradients.set(`${ent.start}.x`, (gradients.get(`${ent.start}.x`) ?? 0) - gx);
  gradients.set(`${ent.start}.y`, (gradients.get(`${ent.start}.y`) ?? 0) - gy);
  gradients.set(`${ent.end}.x`, (gradients.get(`${ent.end}.x`) ?? 0) + gx);
  gradients.set(`${ent.end}.y`, (gradients.get(`${ent.end}.y`) ?? 0) + gy);

  return error;
}
