import type { SketchModel } from "../model/SketchModel";
import type { EntityId } from "../model/Entity";
import type { Constraint, ConstraintId } from "../model/Constraint";

/**
 * Dependency graph between entities and constraints.
 * Tracks which constraints affect which entities and vice versa.
 */
export class ConstraintGraph {
  private entityToConstraints: Map<EntityId, ConstraintId[]>;
  private constraintToEntities: Map<ConstraintId, EntityId[]>;

  constructor(model: SketchModel) {
    this.entityToConstraints = new Map();
    this.constraintToEntities = new Map();
    this.rebuild(model);
  }

  /**
   * Returns all constraint IDs that reference the given entity.
   */
  getConstraintsForEntity(entityId: EntityId): ConstraintId[] {
    return this.entityToConstraints.get(entityId) ?? [];
  }

  /**
   * Returns all entity IDs referenced by the given constraint.
   */
  getEntitiesForConstraint(constraintId: ConstraintId): EntityId[] {
    return this.constraintToEntities.get(constraintId) ?? [];
  }

  /**
   * Rebuilds the graph from the current model state.
   */
  rebuild(model: SketchModel): void {
    this.entityToConstraints.clear();
    this.constraintToEntities.clear();

    for (const [cId, constraint] of model.constraints) {
      const entities = this.extractEntities(constraint);
      this.constraintToEntities.set(cId, entities);

      for (const entityId of entities) {
        const list = this.entityToConstraints.get(entityId);
        if (list) {
          list.push(cId);
        } else {
          this.entityToConstraints.set(entityId, [cId]);
        }
      }
    }
  }

  private extractEntities(constraint: Constraint): EntityId[] {
    const result: EntityId[] = [];

    switch (constraint.kind) {
      case "coincident":
        result.push(constraint.pointA, constraint.pointB);
        break;
      case "horizontal":
        result.push(constraint.line);
        break;
      case "vertical":
        result.push(constraint.line);
        break;
      case "parallel":
        result.push(constraint.lineA, constraint.lineB);
        break;
      case "perpendicular":
        result.push(constraint.lineA, constraint.lineB);
        break;
      case "equal":
        result.push(constraint.entityA, constraint.entityB);
        break;
      case "symmetric":
        result.push(constraint.entityA, constraint.entityB, constraint.symmetryLine);
        break;
      case "tangent":
        result.push(constraint.entityA, constraint.entityB);
        break;
      case "collinear":
        result.push(constraint.line, ...constraint.points);
        break;
      case "fixed":
        result.push(constraint.entity);
        break;
      case "lengthDim":
        result.push(constraint.line);
        break;
      case "angleDim":
        result.push(constraint.lineA, constraint.lineB);
        break;
      case "radiusDim":
        result.push(constraint.entity);
        break;
    }

    return result;
  }
}
