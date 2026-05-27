import type { EntityId } from "./Entity";

export type ConstraintId = string;

export interface BaseConstraint {
  id: ConstraintId;
  entities: EntityId[];
  satisfied: boolean;
}

export interface CoincidentConstraint extends BaseConstraint {
  kind: "coincident";
  pointA: EntityId;
  pointB: EntityId;
}

export interface HorizontalConstraint extends BaseConstraint {
  kind: "horizontal";
  line: EntityId;
}

export interface VerticalConstraint extends BaseConstraint {
  kind: "vertical";
  line: EntityId;
}

export interface ParallelConstraint extends BaseConstraint {
  kind: "parallel";
  lineA: EntityId;
  lineB: EntityId;
}

export interface PerpendicularConstraint extends BaseConstraint {
  kind: "perpendicular";
  lineA: EntityId;
  lineB: EntityId;
}

export interface EqualConstraint extends BaseConstraint {
  kind: "equal";
  entityA: EntityId;
  entityB: EntityId;
}

export interface SymmetricConstraint extends BaseConstraint {
  kind: "symmetric";
  entityA: EntityId;
  entityB: EntityId;
  symmetryLine: EntityId;
}

export interface TangentConstraint extends BaseConstraint {
  kind: "tangent";
  entityA: EntityId;
  entityB: EntityId;
}

export interface CollinearConstraint extends BaseConstraint {
  kind: "collinear";
  points: EntityId[];
  line: EntityId;
}

export interface FixedConstraint extends BaseConstraint {
  kind: "fixed";
  entity: EntityId;
}

export interface LengthDimConstraint extends BaseConstraint {
  kind: "lengthDim";
  line: EntityId;
  value: number;
}

export interface AngleDimConstraint extends BaseConstraint {
  kind: "angleDim";
  lineA: EntityId;
  lineB: EntityId;
  value: number;
}

export interface RadiusDimConstraint extends BaseConstraint {
  kind: "radiusDim";
  entity: EntityId;
  value: number;
}

export type Constraint =
  | CoincidentConstraint
  | HorizontalConstraint
  | VerticalConstraint
  | ParallelConstraint
  | PerpendicularConstraint
  | EqualConstraint
  | SymmetricConstraint
  | TangentConstraint
  | CollinearConstraint
  | FixedConstraint
  | LengthDimConstraint
  | AngleDimConstraint
  | RadiusDimConstraint;

let _nextId = 1;

export function resetConstraintIdCounter(): void {
  _nextId = 1;
}

export function generateConstraintId(): ConstraintId {
  return `c${_nextId++}`;
}
