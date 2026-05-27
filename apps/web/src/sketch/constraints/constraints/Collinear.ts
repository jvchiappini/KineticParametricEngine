import type { SketchModel } from "../../model/SketchModel";
import type { CollinearConstraint } from "../../model/Constraint";

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
 * Evaluates a collinear constraint: multiple points lie on the same line.
 * Error = sum of squared cross products of (point - lineStart) with (lineEnd - lineStart).
 */
export function evaluateConstraint(
  constraint: CollinearConstraint,
  model: SketchModel,
  variables: Map<string, number>,
  gradients: Map<string, number>
): number {
  const lineEnt = model.entities.get(constraint.line);
  if (!lineEnt || lineEnt.kind !== "line" || constraint.points.length === 0) return 0;

  const sPt = model.points.get(lineEnt.start);
  const ePt = model.points.get(lineEnt.end);
  if (!sPt || !ePt) return 0;

  const sx = readVar(variables, `${lineEnt.start}.x`, sPt.x);
  const sy = readVar(variables, `${lineEnt.start}.y`, sPt.y);
  const ex = readVar(variables, `${lineEnt.end}.x`, ePt.x);
  const ey = readVar(variables, `${lineEnt.end}.y`, ePt.y);

  const dxL = ex - sx;
  const dyL = ey - sy;
  const lineLen2 = dxL * dxL + dyL * dyL + 1e-10;

  let totalError = 0;

  for (const ptId of constraint.points) {
    const pt = model.points.get(ptId);
    if (!pt) continue;

    const px = readVar(variables, `${ptId}.x`, pt.x);
    const py = readVar(variables, `${ptId}.y`, pt.y);

    const cross = (px - sx) * dyL - (py - sy) * dxL;
    const crossError = (cross * cross) / lineLen2;
    totalError += crossError;

    const dE_dpx = 2 * cross * dyL / lineLen2;
    const dE_dpy = -2 * cross * dxL / lineLen2;

    const dE_dsx = (-2 * cross * dyL * lineLen2 - 2 * cross * cross * (-dxL)) / (lineLen2 * lineLen2);
    const dE_dsy = (2 * cross * dxL * lineLen2 - 2 * cross * cross * (-dyL)) / (lineLen2 * lineLen2);
    const dE_dex = (2 * cross * (-(py - sy)) * lineLen2 - 2 * cross * cross * dxL) / (lineLen2 * lineLen2);
    const dE_dey = (2 * cross * (px - sx) * lineLen2 - 2 * cross * cross * dyL) / (lineLen2 * lineLen2);

    gradients.set(`${ptId}.x`, (gradients.get(`${ptId}.x`) ?? 0) + dE_dpx);
    gradients.set(`${ptId}.y`, (gradients.get(`${ptId}.y`) ?? 0) + dE_dpy);
    gradients.set(`${lineEnt.start}.x`, (gradients.get(`${lineEnt.start}.x`) ?? 0) + dE_dsx);
    gradients.set(`${lineEnt.start}.y`, (gradients.get(`${lineEnt.start}.y`) ?? 0) + dE_dsy);
    gradients.set(`${lineEnt.end}.x`, (gradients.get(`${lineEnt.end}.x`) ?? 0) + dE_dex);
    gradients.set(`${lineEnt.end}.y`, (gradients.get(`${lineEnt.end}.y`) ?? 0) + dE_dey);
  }

  return totalError;
}
