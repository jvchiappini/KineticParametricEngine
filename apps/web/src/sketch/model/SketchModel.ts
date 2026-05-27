import type { Entity, EntityId, PointEntity } from "./Entity";
import type { Constraint, ConstraintId } from "./Constraint";

export interface SketchModel {
  points: Map<EntityId, PointEntity>;
  entities: Map<EntityId, Entity>;
  constraints: Map<ConstraintId, Constraint>;
  activeTool: string;
  plane: "XY" | "XZ" | "YZ";
  planeOffset: number;
}

export function createSketchModel(): SketchModel {
  return {
    points: new Map(),
    entities: new Map(),
    constraints: new Map(),
    activeTool: "select",
    plane: "XZ",
    planeOffset: 0,
  };
}

export function addEntity(
  model: SketchModel,
  entity: Entity
): SketchModel {
  const next = cloneModel(model);
  if (entity.kind === "point") {
    next.points.set(entity.id, entity);
  }
  next.entities.set(entity.id, entity);
  return next;
}

export function removeEntity(
  model: SketchModel,
  entityId: EntityId
): SketchModel {
  const next = cloneModel(model);
  const entity = next.entities.get(entityId);
  if (!entity) return next;

  if (entity.kind === "point") {
    next.points.delete(entityId);
  }

  const refs = findReferencingEntities(next, entityId);
  for (const ref of refs) {
    next.entities.delete(ref);
    if (ref === entityId) continue;
    const refEnt = next.entities.get(ref);
    if (refEnt?.kind === "point") next.points.delete(ref);
  }

  next.entities.delete(entityId);

  const deps = findDependentConstraints(next, entityId);
  for (const cId of deps) {
    next.constraints.delete(cId);
  }

  return next;
}

export function removeConstraint(
  model: SketchModel,
  constraintId: ConstraintId
): SketchModel {
  const next = cloneModel(model);
  next.constraints.delete(constraintId);
  return next;
}

export function addConstraint(
  model: SketchModel,
  constraint: Constraint
): SketchModel {
  const next = cloneModel(model);
  next.constraints.set(constraint.id, constraint);
  return next;
}

export function findPoint(
  model: SketchModel,
  pointId: EntityId
): PointEntity | undefined {
  return model.points.get(pointId);
}

export function findEntity(
  model: SketchModel,
  entityId: EntityId
): Entity | undefined {
  return model.entities.get(entityId);
}

export function getPointCoords(
  model: SketchModel,
  pointId: EntityId
): { x: number; y: number } | undefined {
  const p = model.points.get(pointId);
  if (!p) return undefined;
  return { x: p.x, y: p.y };
}

export function updatePoint(
  model: SketchModel,
  pointId: EntityId,
  x: number,
  y: number
): SketchModel {
  const next = cloneModel(model);
  const p = next.points.get(pointId);
  if (p) {
    next.points.set(pointId, { ...p, x, y });
  }
  return next;
}

export function updateEntity<T extends Entity>(
  model: SketchModel,
  entityId: EntityId,
  updater: (e: T) => T
): SketchModel {
  const next = cloneModel(model);
  const ent = next.entities.get(entityId) as T | undefined;
  if (ent) {
    const updated = updater(ent);
    next.entities.set(entityId, updated);
    if (updated.kind === "point") {
      next.points.set(entityId, updated as unknown as PointEntity);
    }
  }
  return next;
}

export function allPoints(model: SketchModel): PointEntity[] {
  return Array.from(model.points.values());
}

export function allEntities(model: SketchModel): Entity[] {
  return Array.from(model.entities.values());
}

export function allConstraints(model: SketchModel): Constraint[] {
  return Array.from(model.constraints.values());
}

function findReferencingEntities(
  model: SketchModel,
  pointId: EntityId
): EntityId[] {
  const refs: EntityId[] = [];
  for (const [id, ent] of model.entities) {
    if (id === pointId) continue;
    switch (ent.kind) {
      case "line":
        if (ent.start === pointId || ent.end === pointId) refs.push(id);
        break;
      case "circle":
        if (ent.center === pointId) refs.push(id);
        break;
      case "arc":
        if (
          ent.center === pointId ||
          ent.startPoint === pointId ||
          ent.endPoint === pointId
        )
          refs.push(id);
        break;
      case "polyline":
        if (ent.points.includes(pointId)) refs.push(id);
        break;
    }
  }
  return refs;
}

function findDependentConstraints(
  model: SketchModel,
  entityId: EntityId
): ConstraintId[] {
  const deps: ConstraintId[] = [];
  for (const [id, c] of model.constraints) {
    if (c.entities.includes(entityId)) deps.push(id);
  }
  return deps;
}

export function cloneModel(model: SketchModel): SketchModel {
  return {
    points: new Map(model.points),
    entities: new Map(model.entities),
    constraints: new Map(model.constraints),
    activeTool: model.activeTool,
    plane: model.plane,
    planeOffset: model.planeOffset,
  };
}
