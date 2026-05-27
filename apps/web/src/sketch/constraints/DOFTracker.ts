import type { SketchModel } from "../model/SketchModel";
import type { EntityId } from "../model/Entity";
import type { Constraint } from "../model/Constraint";

function constraintDOF(constraint: Constraint): number {
  switch (constraint.kind) {
    case "coincident":
      return 2;
    case "horizontal":
      return 1;
    case "vertical":
      return 1;
    case "parallel":
      return 1;
    case "perpendicular":
      return 1;
    case "equal":
      return 1;
    case "symmetric":
      return 2;
    case "tangent":
      return 1;
    case "collinear":
      return Math.max(0, constraint.points.length - 1);
    case "fixed":
      return 0;
    case "lengthDim":
      return 1;
    case "angleDim":
      return 1;
    case "radiusDim":
      return 1;
  }
}

/**
 * Counts the total degrees of freedom in the sketch.
 * Free points contribute 2 DOF each. Circles contribute 1 (radius).
 * Arcs contribute 3 (radius, startAngle, endAngle).
 * Constraints subtract their respective DOF.
 */
export function countDOF(model: SketchModel): number {
  let dof = 0;

  for (const point of model.points.values()) {
    if (!point.fixed) {
      dof += 2;
    }
  }

  for (const entity of model.entities.values()) {
    if (entity.kind === "circle") {
      dof += 1;
    } else if (entity.kind === "arc") {
      dof += 3;
    }
  }

  for (const constraint of model.constraints.values()) {
    dof -= constraintDOF(constraint);
  }

  return Math.max(0, dof);
}

/**
 * Returns a per-entity breakdown of degrees of freedom.
 * Each entry maps entity ID to its individual DOF contribution.
 */
export function dofBreakdown(model: SketchModel): Map<EntityId, number> {
  const breakdown = new Map<EntityId, number>();

  for (const point of model.points.values()) {
    if (!point.fixed) {
      breakdown.set(point.id, 2);
    }
  }

  for (const entity of model.entities.values()) {
    if (entity.kind === "circle") {
      breakdown.set(entity.id, (breakdown.get(entity.id) ?? 0) + 1);
    } else if (entity.kind === "arc") {
      breakdown.set(entity.id, (breakdown.get(entity.id) ?? 0) + 3);
    }
  }

  for (const constraint of model.constraints.values()) {
    const dof = constraintDOF(constraint);
    for (const eId of constraint.entities) {
      breakdown.set(eId, (breakdown.get(eId) ?? 0) - dof / constraint.entities.length);
    }
  }

  return breakdown;
}
