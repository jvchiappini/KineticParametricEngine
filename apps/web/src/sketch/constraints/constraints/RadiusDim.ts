import type { SketchModel } from "../../model/SketchModel";
import type { RadiusDimConstraint } from "../../model/Constraint";

function readVar(variables: Map<string, number>, key: string, fallback: number): number {
  const v = variables.get(key);
  return v !== undefined ? v : fallback;
}

/**
 * Evaluates a radius dimension constraint on a circle or arc.
 * Error = current radius - target radius.
 */
export function evaluateConstraint(
  constraint: RadiusDimConstraint,
  model: SketchModel,
  variables: Map<string, number>,
  gradients: Map<string, number>
): number {
  const entity = model.entities.get(constraint.entity);
  if (!entity || (entity.kind !== "circle" && entity.kind !== "arc")) return 0;

  const rKey = `${constraint.entity}.radius`;
  const currentRadius = readVar(variables, rKey, "radius" in entity ? (entity as any).radius : 0);

  const error = currentRadius - constraint.value;

  gradients.set(rKey, (gradients.get(rKey) ?? 0) + 1);

  return error;
}
