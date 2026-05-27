import type { SketchModel } from "../../model/SketchModel";
import type { ParallelConstraint } from "../../model/Constraint";

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
 * Evaluates a parallel constraint between two lines.
 * Error = cross product of direction vectors: dx1*dy2 - dy1*dx2 = 0.
 */
export function evaluateConstraint(
  constraint: ParallelConstraint,
  model: SketchModel,
  variables: Map<string, number>,
  gradients: Map<string, number>
): number {
  const eA = model.entities.get(constraint.lineA);
  const eB = model.entities.get(constraint.lineB);
  if (!eA || !eB || eA.kind !== "line" || eB.kind !== "line") return 0;

  const a1 = getCoords(model, variables, eA.start);
  const a2 = getCoords(model, variables, eA.end);
  const b1 = getCoords(model, variables, eB.start);
  const b2 = getCoords(model, variables, eB.end);

  const dx1 = a2.x - a1.x;
  const dy1 = a2.y - a1.y;
  const dx2 = b2.x - b1.x;
  const dy2 = b2.y - b1.y;

  const error = dx1 * dy2 - dy1 * dx2;

  gradients.set(`${eA.start}.x`, (gradients.get(`${eA.start}.x`) ?? 0) - dy2);
  gradients.set(`${eA.start}.y`, (gradients.get(`${eA.start}.y`) ?? 0) + dx2);
  gradients.set(`${eA.end}.x`, (gradients.get(`${eA.end}.x`) ?? 0) + dy2);
  gradients.set(`${eA.end}.y`, (gradients.get(`${eA.end}.y`) ?? 0) - dx2);

  gradients.set(`${eB.start}.x`, (gradients.get(`${eB.start}.x`) ?? 0) + dy1);
  gradients.set(`${eB.start}.y`, (gradients.get(`${eB.start}.y`) ?? 0) - dx1);
  gradients.set(`${eB.end}.x`, (gradients.get(`${eB.end}.x`) ?? 0) - dy1);
  gradients.set(`${eB.end}.y`, (gradients.get(`${eB.end}.y`) ?? 0) + dx1);

  return error;
}
