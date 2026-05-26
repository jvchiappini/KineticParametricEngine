import {
  sketch_new,
  sketch_add_line,
  sketch_add_rect,
  sketch_solve,
  sketch_snap,
  sketch_extrude,
} from "../kpe-wasm/kpe_wasm.js";

export type ToolType = "select" | "line" | "rect" | "circle" | "polyline" | "extrude";

interface SnapResult {
  x: number;
  y: number;
  kind: string;
  target_id: number | null;
}

export interface SketchState {
  docJson: string;
  tool: ToolType;
  hoverSnap: SnapResult | null;
  selectedIds: number[];
  constraints: any[];
  mode: "idle" | "drawing";
  drawStart: { x: number; y: number } | null;
}

export class SketchEngine {
  state: SketchState;
  gridSize = 0.5;
  snapEnabled = true;

  constructor() {
    const docJson = sketch_new();
    this.state = {
      docJson,
      tool: "line",
      hoverSnap: null,
      selectedIds: [],
      constraints: [],
      mode: "idle",
      drawStart: null,
    };
  }

  setTool(tool: ToolType) {
    this.state.tool = tool;
    this.state.mode = "idle";
    this.state.drawStart = null;
    this.state.selectedIds = [];
  }

  snap(x: number, y: number): SnapResult {
    if (!this.snapEnabled) return { x, y, kind: "none", target_id: null };
    try {
      const raw = sketch_snap(this.state.docJson, x, y, this.gridSize);
      return JSON.parse(raw);
    } catch {
      return { x, y, kind: "none", target_id: null };
    }
  }

  addLine(x1: number, y1: number, x2: number, y2: number) {
    try {
      this.state.docJson = sketch_add_line(this.state.docJson, x1, y1, x2, y2);
      this.state.docJson = sketch_solve(this.state.docJson);
    } catch (e) {
      console.error("Add line failed:", e);
    }
  }

  addRect(x: number, y: number, w: number, h: number) {
    try {
      this.state.docJson = sketch_add_rect(this.state.docJson, x, y, w, h);
      this.state.docJson = sketch_solve(this.state.docJson);
    } catch (e) {
      console.error("Add rect failed:", e);
    }
  }

  extrude(distance: number): string | null {
    try {
      return sketch_extrude(this.state.docJson, distance);
    } catch (e) {
      console.error("Extrude failed:", e);
      return null;
    }
  }

  getDoc(): any {
    try {
      return JSON.parse(this.state.docJson);
    } catch {
      return null;
    }
  }

  getContours(): [number, number][][] {
    const doc = this.getDoc();
    if (!doc) return [];
    const contours: [number, number][][] = [];
    const ptMap = new Map<number, [number, number]>();
    for (const p of doc.points || []) {
      ptMap.set(p.id, [p.x, p.y]);
    }
    for (const line of doc.lines || []) {
      const s = ptMap.get(line.start);
      const e = ptMap.get(line.end);
      if (s && e) contours.push([s, e]);
    }
    for (const c of doc.circles || []) {
      const center = ptMap.get(c.center);
      if (!center) continue;
      const pts: [number, number][] = [];
      for (let i = 0; i < 32; i++) {
        const a = (i / 32) * Math.PI * 2;
        pts.push([center[0] + c.radius * Math.cos(a), center[1] + c.radius * Math.sin(a)]);
      }
      contours.push(pts);
    }
    return contours;
  }

  getPoints(): { id: number; x: number; y: number }[] {
    const doc = this.getDoc();
    return doc?.points || [];
  }

  getConstraints(): any[] {
    const doc = this.getDoc();
    return doc?.constraints || [];
  }
}
