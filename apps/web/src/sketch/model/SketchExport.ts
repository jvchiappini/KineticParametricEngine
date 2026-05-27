import type { SketchModel } from "./SketchModel";
import { allEntities, findPoint } from "./SketchModel";
import type { PointEntity, LineEntity } from "./Entity";

export interface ProfileNode {
  type: "profile";
  closed: boolean;
  points: [number, number][];
  plane: "XY" | "XZ" | "YZ";
  planeOffset: number;
  origin: [number, number, number];
  fullyConstrained: boolean;
  dof: number;
}

export function exportSketchProfile(
  model: SketchModel,
  dof: number
): ProfileNode {
  const entities = allEntities(model);
  const profilePoints = extractClosedProfile(entities, model);
  const fullyConstrained = dof === 0;

  return {
    type: "profile",
    closed: true,
    points: profilePoints,
    plane: model.plane,
    planeOffset: model.planeOffset,
    origin: [0, 0, 0],
    fullyConstrained,
    dof,
  };
}

function extractClosedProfile(
  entities: import("./Entity").Entity[],
  model: SketchModel
): [number, number][] {
  const lines = entities.filter(
    (e) => e.kind === "line" && !e.construction
  ) as LineEntity[];

  if (lines.length === 0) return [];

  const adjacency = new Map<string, string[]>();
  const pointCoords = new Map<string, { x: number; y: number }>();

  for (const line of lines) {
    if (!adjacency.has(line.start)) adjacency.set(line.start, []);
    if (!adjacency.has(line.end)) adjacency.set(line.end, []);
    adjacency.get(line.start)!.push(line.end);
    adjacency.get(line.end)!.push(line.start);

    const p1 = findPoint(model, line.start);
    const p2 = findPoint(model, line.end);
    if (p1) pointCoords.set(line.start, { x: p1.x, y: p1.y });
    if (p2) pointCoords.set(line.end, { x: p2.x, y: p2.y });
  }

  if (pointCoords.size === 0) return [];

  const start = pointCoords.keys().next().value as string;
  const visited = new Set<string>();
  const result: [number, number][] = [];
  let current = start;

  for (let i = 0; i < lines.length + 1; i++) {
    if (visited.has(current)) break;
    visited.add(current);
    const pt = pointCoords.get(current);
    if (pt) result.push([pt.x, pt.y]);
    const neighbors = adjacency.get(current) || [];
    const next = neighbors.find((n) => !visited.has(n));
    if (!next) break;
    current = next;
  }

  if (
    result.length > 2 &&
    (result[0][0] !== result[result.length - 1][0] ||
      result[0][1] !== result[result.length - 1][1])
  ) {
    const firstPt = pointCoords.get(start);
    if (firstPt) result.push([firstPt.x, firstPt.y]);
  }

  return result;
}

export interface RevolveProfile {
  type: "revolve_profile";
  profile: [number, number][];
  axis: [number, number][];
  plane: "XY" | "XZ" | "YZ";
  planeOffset: number;
}

export function exportRevolveProfile(
  model: SketchModel
): RevolveProfile | null {
  const entities = allEntities(model);
  const axisLine = entities.find(
    (e) => e.kind === "line" && e.construction
  ) as LineEntity | undefined;

  if (!axisLine) return null;

  const p1 = findPoint(model, axisLine.start);
  const p2 = findPoint(model, axisLine.end);
  if (!p1 || !p2) return null;

  const nonConstruction = entities.filter(
    (e) => e.kind === "line" && !e.construction
  ) as LineEntity[];
  if (nonConstruction.length === 0) return null;

  const profile: [number, number][] = [];
  for (const line of nonConstruction) {
    const sp = findPoint(model, line.start);
    const ep = findPoint(model, line.end);
    if (sp && profile.length === 0) profile.push([sp.x, sp.y]);
    if (ep) profile.push([ep.x, ep.y]);
  }

  return {
    type: "revolve_profile",
    profile,
    axis: [
      [p1.x, p1.y],
      [p2.x, p2.y],
    ],
    plane: model.plane,
    planeOffset: model.planeOffset,
  };
}
