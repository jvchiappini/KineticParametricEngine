import type { SketchTool, ToolContext } from "./types";
import type { EntityId } from "../model/Entity";
import type { Command } from "../model/SketchHistory";
import type { SketchModel } from "../model/SketchModel";
import { addEntity, removeEntity, addConstraint, removeConstraint } from "../model/SketchModel";
import { makePoint, makeLine } from "../model/Entity";
import { generateConstraintId } from "../model/Constraint";
import { compoundCommand } from "../model/SketchHistory";

type DimMode = "linear" | "radial" | "angular";

export class DimensionTool implements SketchTool {
  readonly name = "dimension";
  readonly cursor = "crosshair";

  private mode: DimMode = "linear";
  private state: "idle" | "linearFirstPt" | "angularFirstLine" = "idle";
  private firstEntityId: EntityId | null = null;
  private firstPoint: { x: number; y: number } | null = null;
  private isDiameter = false;

  onActivate(_c: ToolContext): void { this.reset(); }
  onDeactivate(c: ToolContext): void { c.setPreview(null); this.reset(); }

  onPointerDown(_e: PointerEvent, ctx: ToolContext): void {
    const c = ctx.getSnapAwareCursor();
    if (this.mode === "linear") this.handleLinear(c, ctx);
    else if (this.mode === "radial") this.handleRadial(c, ctx);
    else this.handleAngular(c, ctx);
  }

  onPointerMove(_e: PointerEvent, ctx: ToolContext): void {
    if (this.state !== "idle") ctx.onRequestRender();
  }

  onPointerUp(_e: PointerEvent, _ctx: ToolContext): void {}

  onKeyDown(e: KeyboardEvent, ctx: ToolContext): void {
    if ((e.key === "d" || e.key === "D") && this.mode === "radial") {
      this.isDiameter = !this.isDiameter;
      ctx.onRequestRender();
    }
    if (e.key === "l" || e.key === "L") { this.mode = "linear"; this.reset(); ctx.onRequestRender(); }
    if (e.key === "r" || e.key === "R") { this.mode = "radial"; this.reset(); ctx.onRequestRender(); }
    if (e.key === "a" || e.key === "A") { this.mode = "angular"; this.reset(); ctx.onRequestRender(); }
    if (e.key === "Escape") { this.reset(); ctx.onRequestRender(); }
    if (e.key === "Enter" && this.state === "linearFirstPt") {
      this.completeTwoPt(ctx, ctx.getSnapAwareCursor());
    }
  }

  getHint(): string {
    const m = { linear: "Linear (L)", radial: `Radial${this.isDiameter ? " (D:diam)" : " (D:rad)"}`, angular: "Angular (A)" };
    const st = this.state === "linearFirstPt" ? "Click second point" : this.state === "angularFirstLine" ? "Click second line" : `Click ${this.mode === "linear" ? "line or first point" : this.mode === "radial" ? "circle/arc" : "first line"}`;
    return `${m[this.mode]} - ${st}`;
  }

  private handleLinear(c: { x: number; y: number }, ctx: ToolContext): void {
    if (this.state === "idle") {
      const hit = hitLine(c, ctx.model);
      if (hit) this.createLinear(hit, ctx);
      else { this.firstPoint = c; this.state = "linearFirstPt"; }
    } else this.completeTwoPt(ctx, c);
  }

  private handleRadial(c: { x: number; y: number }, ctx: ToolContext): void {
    const hit = hitCircleArc(c, ctx.model);
    if (hit) this.createRadial(hit, ctx);
  }

  private handleAngular(c: { x: number; y: number }, ctx: ToolContext): void {
    if (this.state === "idle") {
      const hit = hitLine(c, ctx.model);
      if (hit) { this.firstEntityId = hit; this.state = "angularFirstLine"; }
    } else {
      const hit = hitLine(c, ctx.model);
      if (hit && hit !== this.firstEntityId) this.createAngular(this.firstEntityId!, hit, ctx);
    }
  }

  private createLinear(lineId: EntityId, ctx: ToolContext): void {
    const line = ctx.model.entities.get(lineId);
    if (!line || line.kind !== "line") return;
    const s = ctx.model.points.get(line.start), e = ctx.model.points.get(line.end);
    if (!s || !e) return;
    const val = Math.hypot(e.x - s.x, e.y - s.y);
    const cId = generateConstraintId();
    const cmd: Command = {
      description: "Create linear dimension",
      execute: (m) => addConstraint(m, { id: cId, kind: "lengthDim", entities: [lineId], line: lineId, value: val, satisfied: true }),
      undo: (m) => removeConstraint(m, cId),
    };
    ctx.history.push(cmd);
    ctx.setModel(cmd.execute(ctx.model));
    this.reset();
  }

  private completeTwoPt(ctx: ToolContext, p2: { x: number; y: number }): void {
    const pt1 = makePoint(this.firstPoint!.x, this.firstPoint!.y);
    const pt2 = makePoint(p2.x, p2.y);
    const cl = makeLine(pt1.id, pt2.id, true);
    const val = Math.hypot(p2.x - this.firstPoint!.x, p2.y - this.firstPoint!.y);
    const cId = generateConstraintId();
    const cmd = compoundCommand("Create linear dim (2pt)", [
      { description: "Add pt1", execute: (m) => addEntity(m, pt1), undo: (m) => removeEntity(m, pt1.id) },
      { description: "Add pt2", execute: (m) => addEntity(m, pt2), undo: (m) => removeEntity(m, pt2.id) },
      { description: "Add constr line", execute: (m) => addEntity(m, cl), undo: (m) => removeEntity(m, cl.id) },
      { description: "Add dim", execute: (m) => addConstraint(m, { id: cId, kind: "lengthDim", entities: [cl.id], line: cl.id, value: val, satisfied: true }), undo: (m) => removeConstraint(m, cId) },
    ]);
    ctx.history.push(cmd);
    ctx.setModel(cmd.execute(ctx.model));
    this.reset();
  }

  private createRadial(entityId: EntityId, ctx: ToolContext): void {
    const e = ctx.model.entities.get(entityId);
    if (!e || (e.kind !== "circle" && e.kind !== "arc")) return;
    const r = e.radius;
    const val = this.isDiameter ? r * 2 : r;
    const cId = generateConstraintId();
    const cmd: Command = {
      description: `Create ${this.isDiameter ? "diameter" : "radius"} dim`,
      execute: (m) => addConstraint(m, { id: cId, kind: "radiusDim", entities: [entityId], entity: entityId, value: val, satisfied: true }),
      undo: (m) => removeConstraint(m, cId),
    };
    ctx.history.push(cmd);
    ctx.setModel(cmd.execute(ctx.model));
    this.reset();
  }

  private createAngular(la: EntityId, lb: EntityId, ctx: ToolContext): void {
    const ea = ctx.model.entities.get(la), eb = ctx.model.entities.get(lb);
    if (!ea || ea.kind !== "line" || !eb || eb.kind !== "line") return;
    const a1 = ctx.model.points.get(ea.start), a2 = ctx.model.points.get(ea.end);
    const b1 = ctx.model.points.get(eb.start), b2 = ctx.model.points.get(eb.end);
    if (!a1 || !a2 || !b1 || !b2) return;
    const aa = Math.atan2(a2.y - a1.y, a2.x - a1.x);
    const ab = Math.atan2(b2.y - b1.y, b2.x - b1.x);
    let ang = Math.abs(aa - ab);
    if (ang > Math.PI) ang = 2 * Math.PI - ang;
    const cId = generateConstraintId();
    const cmd: Command = {
      description: "Create angular dim",
      execute: (m) => addConstraint(m, { id: cId, kind: "angleDim", entities: [la, lb], lineA: la, lineB: lb, value: ang * (180 / Math.PI), satisfied: true }),
      undo: (m) => removeConstraint(m, cId),
    };
    ctx.history.push(cmd);
    ctx.setModel(cmd.execute(ctx.model));
    this.reset();
  }

  private reset(): void {
    this.state = "idle";
    this.firstEntityId = null;
    this.firstPoint = null;
  }
}

function hitLine(c: { x: number; y: number }, m: SketchModel): EntityId | null {
  const t = 0.5;
  for (const [id, e] of m.entities) {
    if (e.kind !== "line") continue;
    const s = m.points.get(e.start), en = m.points.get(e.end);
    if (!s || !en) continue;
    if (p2sDist(c.x, c.y, s.x, s.y, en.x, en.y) < t) return id;
  }
  return null;
}

function hitCircleArc(c: { x: number; y: number }, m: SketchModel): EntityId | null {
  const t = 0.5;
  for (const [id, e] of m.entities) {
    if (e.kind !== "circle" && e.kind !== "arc") continue;
    const cp = m.points.get(e.center);
    if (!cp) continue;
    if (Math.abs(Math.hypot(c.x - cp.x, c.y - cp.y) - e.radius) < t) return id;
  }
  return null;
}

function p2sDist(px: number, py: number, ax: number, ay: number, bx: number, by: number): number {
  const abx = bx - ax, aby = by - ay, l2 = abx * abx + aby * aby;
  if (l2 === 0) return Math.hypot(px - ax, py - ay);
  const t = Math.max(0, Math.min(1, ((px - ax) * abx + (py - ay) * aby) / l2));
  return Math.hypot(px - (ax + t * abx), py - (ay + t * aby));
}
