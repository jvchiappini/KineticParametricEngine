import type { SketchModel } from "./SketchModel";
import { allEntities, allConstraints } from "./SketchModel";
import { countDOF } from "../constraints/DOFTracker";

export interface SerializedSketch {
  id: string;
  name: string;
  plane: "XY" | "XZ" | "YZ";
  planeOffset: number;
  depth: number;
  bevel: boolean;
  bevelSize: number;
  dof: number;
  fullyConstrained: boolean;
  entities: {
    id: string;
    kind: string;
    [key: string]: unknown;
  }[];
  constraints: {
    id: string;
    kind: string;
    entities: string[];
    [key: string]: unknown;
  }[];
}

export interface SketchCollection {
  version: number;
  generatedAt: string;
  sketches: SerializedSketch[];
}

function serializeModel(model: SketchModel): Omit<SerializedSketch, "id" | "name" | "depth" | "bevel" | "bevelSize"> {
  const dof = countDOF(model);
  const entities = allEntities(model).map((e) => {
    const base: Record<string, unknown> = { id: e.id, kind: e.kind };
    switch (e.kind) {
      case "point": base.x = e.x; base.y = e.y; base.fixed = e.fixed; break;
      case "line": base.start = e.start; base.end = e.end; base.construction = e.construction; break;
      case "circle": base.center = e.center; base.radius = e.radius; base.construction = e.construction; break;
      case "arc": base.center = e.center; base.radius = e.radius; base.startAngle = e.startAngle; base.endAngle = e.endAngle; base.startPoint = e.startPoint; base.endPoint = e.endPoint; base.construction = e.construction; break;
      case "polyline": base.points = e.points; base.closed = e.closed; base.construction = e.construction; break;
    }
    return base;
  });

  return {
    plane: model.plane,
    planeOffset: model.planeOffset,
    dof,
    fullyConstrained: dof === 0,
    entities: entities as SerializedSketch["entities"],
    constraints: allConstraints(model).map((c) => ({
      id: c.id,
      kind: c.kind,
      entities: c.entities,
      satisfied: c.satisfied,
      ...Object.fromEntries(Object.entries(c).filter(([k]) => !["id", "kind", "entities", "satisfied"].includes(k))),
    })),
  };
}

export function serializeSketches(
  sketchData: { id: string; name: string; model: SketchModel; depth: number; bevel: boolean; bevelSize: number }[]
): SketchCollection {
  return {
    version: 1,
    generatedAt: new Date().toISOString(),
    sketches: sketchData.map((s) => ({ id: s.id, name: s.name, ...serializeModel(s.model), depth: s.depth, bevel: s.bevel, bevelSize: s.bevelSize })),
  };
}

export interface DeserializedSketch {
  id: string; name: string; model: SketchModel; depth: number; bevel: boolean; bevelSize: number;
}

export function deserializeSketches(json: string): DeserializedSketch[] {
  const collection: SketchCollection = JSON.parse(json);
  return collection.sketches.map((s) => {
    const model: SketchModel = { points: new Map(), entities: new Map(), constraints: new Map(), activeTool: "select", plane: s.plane, planeOffset: s.planeOffset };
    for (const e of s.entities) {
      if (e.kind === "point") model.points.set(e.id, e as unknown as import("./Entity").PointEntity);
      model.entities.set(e.id, e as unknown as import("./Entity").Entity);
    }
    for (const c of s.constraints) {
      model.constraints.set(c.id, c as unknown as import("./Constraint").Constraint);
    }
    return { id: s.id, name: s.name, model, depth: s.depth, bevel: s.bevel, bevelSize: s.bevelSize };
  });
}
