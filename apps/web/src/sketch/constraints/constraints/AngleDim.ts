import type { SketchModel } from "../../model/SketchModel";
import type { AngleDimConstraint } from "../../model/Constraint";

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
 * Evaluates an angle dimension constraint between two lines.
 * Error = atan2(dy1, dx1) - atan2(dy2, dx2) - targetValue.
 * The error is normalized to [-pi, pi].
 */
export function evaluateConstraint(
  constraint: AngleDimConstraint,
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

  const len1_2 = dx1 * dx1 + dy1 * dy1 + 1e-10;
  const len2_2 = dx2 * dx2 + dy2 * dy2 + 1e-10;

  const theta1 = Math.atan2(dy1, dx1);
  const theta2 = Math.atan2(dy2, dx2);

  let error = theta1 - theta2 - constraint.value;
  error = Math.atan2(Math.sin(error), Math.cos(error));

  const dT1_dx1 = -dy1 / len1_2;
  const dT1_dy1 = dx1 / len1_2;
  const dT1_dx2 = dy1 / len1_2;
  const dT1_dy2 = -dx1 / len1_2;

  const dT2_dx3 = -dy2 / len2_2;
  const dT2_dy3 = dx2 / len2_2;
  const dT2_dx4 = dy2 / len2_2;
  const dT2_dy4 = -dx2 / len2_2;

  gradients.set(`${eA.start}.x`, (gradients.get(`${eA.start}.x`) ?? 0) + dT1_dx1);
  gradients.set(`${eA.start}.y`, (gradients.get(`${eA.start}.y`) ?? 0) + dT1_dy1);
  gradients.set(`${eA.end}.x`, (gradients.get(`${eA.end}.x`) ?? 0) + dT1_dx2);
  gradients.set(`${eA.end}.y`, (gradients.get(`${eA.end}.y`) ?? 0) + dT1_dy2);

  gradients.set(`${eB.start}.x`, (gradients.get(`${eB.start}.x`) ?? 0) - dT2_dx3);
  gradients.set(`${eB.start}.y`, (gradients.get(`${eB.start}.y`) ?? 0) - dT2_dy3);
  gradients.set(`${eB.end}.x`, (gradients.get(`${eB.end}.x`) ?? 0) - dT2_dx4);
  gradients.set(`${eB.end}.y`, (gradients.get(`${eB.end}.y`) ?? 0) - dT2_dy4);

  return error;
}
