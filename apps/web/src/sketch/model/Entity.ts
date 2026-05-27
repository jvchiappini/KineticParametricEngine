export type EntityId = string;

export interface PointEntity {
  id: EntityId;
  kind: "point";
  x: number;
  y: number;
  fixed: boolean;
}

export interface LineEntity {
  id: EntityId;
  kind: "line";
  start: EntityId;
  end: EntityId;
  construction: boolean;
}

export interface CircleEntity {
  id: EntityId;
  kind: "circle";
  center: EntityId;
  radius: number;
  construction: boolean;
}

export interface ArcEntity {
  id: EntityId;
  kind: "arc";
  center: EntityId;
  radius: number;
  startAngle: number;
  endAngle: number;
  startPoint: EntityId;
  endPoint: EntityId;
  construction: boolean;
}

export interface PolylineEntity {
  id: EntityId;
  kind: "polyline";
  points: EntityId[];
  closed: boolean;
  construction: boolean;
}

export type Entity =
  | PointEntity
  | LineEntity
  | CircleEntity
  | ArcEntity
  | PolylineEntity;

let _nextId = 1;

export function resetEntityIdCounter(): void {
  _nextId = 1;
}

export function generateEntityId(): EntityId {
  return `e${_nextId++}`;
}

export function makePoint(x: number, y: number, fixed = false): PointEntity {
  return { id: generateEntityId(), kind: "point", x, y, fixed };
}

export function makeLine(
  start: EntityId,
  end: EntityId,
  construction = false
): LineEntity {
  return { id: generateEntityId(), kind: "line", start, end, construction };
}

export function makeCircle(
  center: EntityId,
  radius: number,
  construction = false
): CircleEntity {
  return { id: generateEntityId(), kind: "circle", center, radius, construction };
}

export function makeArc(
  center: EntityId,
  radius: number,
  startAngle: number,
  endAngle: number,
  startPoint: EntityId,
  endPoint: EntityId,
  construction = false
): ArcEntity {
  return {
    id: generateEntityId(),
    kind: "arc",
    center,
    radius,
    startAngle,
    endAngle,
    startPoint,
    endPoint,
    construction,
  };
}

export function makePolyline(
  points: EntityId[],
  closed = false,
  construction = false
): PolylineEntity {
  return {
    id: generateEntityId(),
    kind: "polyline",
    points,
    closed,
    construction,
  };
}
