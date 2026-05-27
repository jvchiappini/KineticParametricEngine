import type { SketchModel } from "../../model/SketchModel";
import type { TangentConstraint } from "../../model/Constraint";

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
 * Evaluates a tangent constraint between a line and a circle.
 * Error = (cross^2 / lineLen^2) - radius^2.
 * Gradient is computed via quotient rule on the squared formulation.
 */
export function evaluateConstraint(
  constraint: TangentConstraint,
  model: SketchModel,
  variables: Map<string, number>,
  gradients: Map<string, number>
): number {
  const eA = model.entities.get(constraint.entityA);
  const eB = model.entities.get(constraint.entityB);
  if (!eA || !eB) return 0;

  let lineEnt: typeof eA;
  let circleEnt: typeof eA;

  if (eA.kind === "line" && (eB.kind === "circle" || eB.kind === "arc")) {
    lineEnt = eA;
    circleEnt = eB;
  } else if (eB.kind === "line" && (eA.kind === "circle" || eA.kind === "arc")) {
    lineEnt = eB;
    circleEnt = eA;
  } else {
    return 0;
  }

  const sPt = model.points.get(lineEnt.start);
  const ePt = model.points.get(lineEnt.end);
  if (!sPt || !ePt) return 0;

  const sx = readVar(variables, `${lineEnt.start}.x`, sPt.x);
  const sy = readVar(variables, `${lineEnt.start}.y`, sPt.y);
  const ex = readVar(variables, `${lineEnt.end}.x`, ePt.x);
  const ey = readVar(variables, `${lineEnt.end}.y`, ePt.y);

  const dx = ex - sx;
  const dy = ey - sy;
  const lineLen2 = dx * dx + dy * dy;
  const lineLen = Math.sqrt(lineLen2 + 1e-10);

  const cPt = model.points.get(circleEnt.center);
  if (!cPt) return 0;
  const cx = readVar(variables, `${circleEnt.center}.x`, cPt.x);
  const cy = readVar(variables, `${circleEnt.center}.y`, cPt.y);

  const rKey = `${circleEnt.id}.radius`;
  const radius = readVar(variables, rKey, "radius" in circleEnt ? (circleEnt as any).radius : 0);

  const cross = (cx - sx) * dy - (cy - sy) * dx;
  const error = (cross * cross) / lineLen2 - radius * radius;

  const dC_dcx = 2 * cross * dy / lineLen2;
  const dC_dcy = -2 * cross * dx / lineLen2;
  const dC_dr = -2 * radius;

  const dC_dsx = (-2 * cross * dy * lineLen2 + 2 * cross * cross * dx) / (lineLen2 * lineLen2);
  const dC_dsy = (2 * cross * dx * lineLen2 + 2 * cross * cross * dy) / (lineLen2 * lineLen2);
  const dC_dex = (2 * cross * (-(cy - sy)) * lineLen2 - 2 * cross * cross * dx) / (lineLen2 * lineLen2);
  const dC_dey = (2 * cross * (cx - sx) * lineLen2 - 2 * cross * cross * dy) / (lineLen2 * lineLen2);

  gradients.set(`${circleEnt.center}.x`, (gradients.get(`${circleEnt.center}.x`) ?? 0) + dC_dcx);
  gradients.set(`${circleEnt.center}.y`, (gradients.get(`${circleEnt.center}.y`) ?? 0) + dC_dcy);
  gradients.set(rKey, (gradients.get(rKey) ?? 0) + dC_dr);

  gradients.set(`${lineEnt.start}.x`, (gradients.get(`${lineEnt.start}.x`) ?? 0) + dC_dsx);
  gradients.set(`${lineEnt.start}.y`, (gradients.get(`${lineEnt.start}.y`) ?? 0) + dC_dsy);
  gradients.set(`${lineEnt.end}.x`, (gradients.get(`${lineEnt.end}.x`) ?? 0) + dC_dex);
  gradients.set(`${lineEnt.end}.y`, (gradients.get(`${lineEnt.end}.y`) ?? 0) + dC_dey);

  return error;
}
