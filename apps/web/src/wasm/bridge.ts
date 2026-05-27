import type { SketchModel } from "../sketch/model/SketchModel";
import type { Constraint } from "../sketch/model/Constraint";

type WasmModule = typeof import("../../kpe-wasm/kpe_wasm.js");

let wasm: WasmModule | null = null;
let initPromise: Promise<void> | null = null;

export async function initWasm(): Promise<void> {
  if (wasm) return;
  if (initPromise) return initPromise;
  initPromise = (async () => {
    const mod = await import("../../kpe-wasm/kpe_wasm.js");
    await mod.default();
    wasm = mod;
  })();
  return initPromise;
}

function ready(): WasmModule {
  if (!wasm) throw new Error("WASM not initialized. Call initWasm() first.");
  return wasm;
}

export function hello(): string { return ready().hello(); }

// ── Convert TS SketchModel ↔ Rust JSON ──────────────────────────

export function tsModelToRustJson(model: SketchModel): string {
  const points: { id: number; x: number; y: number }[] = [];
  const lines: { id: number; start: number; end: number }[] = [];
  const arcs: { id: number; center: number; start: number; end: number; radius: number; sweep_angle: number }[] = [];
  const circles: { id: number; center: number; radius: number }[] = [];
  const constraints: Record<string, unknown>[] = [];
  let nextId = 1;
  const idMap = new Map<string, number>();

  for (const [eid, e] of model.entities) {
    if (e.kind === "point") { idMap.set(eid, nextId); points.push({ id: nextId++, x: e.x, y: e.y }); }
  }
  for (const [eid, e] of model.entities) {
    if (e.kind === "line") {
      const lid = nextId++; idMap.set(eid, lid);
      lines.push({ id: lid, start: idMap.get(e.start) ?? 0, end: idMap.get(e.end) ?? 0 });
    } else if (e.kind === "circle") {
      const cid = nextId++; idMap.set(eid, cid);
      circles.push({ id: cid, center: idMap.get(e.center) ?? 0, radius: e.radius });
    } else if (e.kind === "arc") {
      const aid = nextId++; idMap.set(eid, aid);
      arcs.push({ id: aid, center: idMap.get(e.center) ?? 0, start: idMap.get(e.startPoint ?? "") ?? 0, end: idMap.get(e.endPoint ?? "") ?? 0, radius: e.radius, sweep_angle: e.endAngle - e.startAngle });
    }
  }
  for (const [, c] of model.constraints) constraints.push(tsConstraintToRust(c, idMap));
  return JSON.stringify({ points, lines, arcs, circles, constraints, next_id: nextId });
}

function tsConstraintToRust(c: Constraint, idMap: Map<string, number>): Record<string, unknown> {
  const e = c.entities.map((id) => idMap.get(id) ?? 0);
  const rc = c as unknown as Record<string, unknown>;
  switch (c.kind) {
    case "horizontal": return { Horizontal: { line: e[0] } };
    case "vertical": return { Vertical: { line: e[0] } };
    case "coincident": return { Coincident: { point_a: e[0], point_b: e[1] } };
    case "fixed": return { Fix: { point: e[0], x: (rc.x as number) ?? 0, y: (rc.y as number) ?? 0 } };
    case "lengthDim": return { Distance: { point_a: e[0], point_b: e[1], distance: (rc.distance as number) ?? 0 } };
    case "equal": return { EqualLength: { line_a: e[0], line_b: e[1] } };
    case "parallel": return { Parallel: { line_a: e[0], line_b: e[1] } };
    case "perpendicular": return { Perpendicular: { line_a: e[0], line_b: e[1] } };
    case "tangent": return { Tangent: { line: e[0], arc: e[1] } };
    case "radiusDim": return { Radius: { arc_or_circle: e[0], radius: (rc.radius as number) ?? 0 } };
    case "angleDim": return { Angle: { line_a: e[0], line_b: e[1], angle: (rc.angle as number) ?? 0 } };
    case "collinear": return { Collinear: { line_a: e[0], line_b: e[1] } };
    case "symmetric": return { Midpoint: { point: e[0], line: e[1] } };
    default: return {};
  }
}

interface RustDoc {
  points: { id: number; x: number; y: number }[];
  lines: { id: number; start: number; end: number }[];
  arcs: { id: number; center: number; radius: number; sweep_angle: number }[];
  circles: { id: number; center: number; radius: number }[];
}

export function applyRustSolution(model: SketchModel, rustJson: string): void {
  const doc: RustDoc = JSON.parse(rustJson);
  const ridToTs = new Map<number, string>();
  for (const p of doc.points) {
    for (const [tsId, e] of model.entities) {
      if (e.kind === "point" && Math.abs(e.x - p.x) < 0.001 && Math.abs(e.y - p.y) < 0.001) ridToTs.set(p.id, tsId);
    }
  }
  for (const p of doc.points) {
    const tsId = ridToTs.get(p.id);
    if (tsId) { const pt = model.points.get(tsId); if (pt) { pt.x = p.x; pt.y = p.y; } }
  }
}

// ── WASM operations ─────────────────────────────────────────────

export async function solveWithWasm(model: SketchModel): Promise<boolean> {
  if (!wasm) await initWasm();
  const json = tsModelToRustJson(model);
  const result = ready().sketch_solve(json);
  applyRustSolution(model, result);
  return true;
}

/** Count DOF via Rust (async — inits WASM if needed) */
export async function wasmCountDOF(model: SketchModel): Promise<number> {
  if (!wasm) await initWasm();
  return ready().sketch_count_dof(tsModelToRustJson(model));
}

/** Count DOF via Rust (synchronous — WASM must be ready) */
export function wasmCountDOFSync(model: SketchModel): number {
  return ready().sketch_count_dof(tsModelToRustJson(model));
}

/** Get ordered contour chains via Rust (async) */
export async function wasmGetContours(model: SketchModel): Promise<[number, number][][]> {
  if (!wasm) await initWasm();
  return JSON.parse(ready().sketch_get_contours(tsModelToRustJson(model)));
}

/** Get ordered contour chains via Rust (synchronous) */
export function wasmGetContoursSync(model: SketchModel): [number, number][][] {
  return JSON.parse(ready().sketch_get_contours(tsModelToRustJson(model)));
}

/** Extrude via Rust (async) */
export async function wasmExtrude(model: SketchModel, distance: number): Promise<{ vertices: [number, number, number][]; triangles: [number, number, number][] }> {
  if (!wasm) await initWasm();
  return JSON.parse(ready().sketch_extrude(tsModelToRustJson(model), distance));
}

/** Extrude via Rust (synchronous) */
export function wasmExtrudeSync(model: SketchModel, distance: number): { vertices: [number, number, number][]; triangles: [number, number, number][] } {
  return JSON.parse(ready().sketch_extrude(tsModelToRustJson(model), distance));
}
