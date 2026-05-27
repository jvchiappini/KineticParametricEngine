import type { SketchTool, ToolContext } from "./types";
import type { EntityId } from "../model/Entity";
import type { Command } from "../model/SketchHistory";
import type { SketchModel } from "../model/SketchModel";
import {
  removeEntity,
  updatePoint,
  cloneModel,
} from "../model/SketchModel";

const HIT_THRESHOLD = 0.5;
const DRAG_THRESHOLD = 2;

interface HitResult { entityId: EntityId; pointId?: EntityId }

export class SelectTool implements SketchTool {
  readonly name = "select";
  readonly cursor = "default";

  private dragStart: { x: number; y: number } | null = null;
  private dragMoved = false;
  private isBoxSelect = false;
  private hitEntityId: EntityId | null = null;
  private hitPointId: EntityId | null = null;
  private initPos = new Map<EntityId, { x: number; y: number }>();

  onActivate(_c: ToolContext): void { this.reset(); }
  onDeactivate(_c: ToolContext): void { this.reset(); }

  onPointerDown(e: PointerEvent, ctx: ToolContext): void {
    const c = ctx.getCursorInSketch();
    this.dragStart = c;
    this.dragMoved = false;
    this.isBoxSelect = false;
    const hit = hitTest(c, ctx.model);
    this.hitEntityId = hit?.entityId ?? null;
    this.hitPointId = hit?.pointId ?? null;
    if (e.shiftKey && this.hitEntityId) return;
    if (!this.hitEntityId) { ctx.clearSelection(); return; }
    if (!ctx.selectedIds.includes(this.hitEntityId)) {
      ctx.clearSelection();
      ctx.addToSelection([this.hitEntityId]);
    }
  }

  onPointerMove(_e: PointerEvent, ctx: ToolContext): void {
    if (!this.dragStart) return;
    const c = ctx.getCursorInSketch();
    const dx = c.x - this.dragStart.x;
    const dy = c.y - this.dragStart.y;
    if (!this.dragMoved && Math.hypot(dx, dy) > DRAG_THRESHOLD) {
      this.dragMoved = true;
      if (this.hitEntityId || this.hitPointId) {
        this.isBoxSelect = false;
        this.collectInit(ctx);
      } else this.isBoxSelect = true;
    }
    if (!this.dragMoved) return;
    if (this.isBoxSelect) { ctx.onRequestRender(); return; }
    const ids = [...this.initPos.keys()];
    let m = ctx.model;
    if (this.hitPointId) {
      const p = ctx.model.points.get(this.hitPointId);
      if (p) m = updatePoint(m, this.hitPointId, p.x + dx, p.y + dy);
    } else for (const [pid, pos] of this.initPos) m = updatePoint(m, pid, pos.x + dx, pos.y + dy);
    if (m !== ctx.model) ctx.setModel(m);
  }

  onPointerUp(e: PointerEvent, ctx: ToolContext): void {
    if (!this.dragStart) return;
    if (!this.dragMoved) {
      if (e.shiftKey && this.hitEntityId) this.toggleSel(ctx);
      this.reset(); return;
    }
    if (this.isBoxSelect) this.finishBox(ctx);
    else this.finishMove(ctx);
    this.reset();
  }

  onKeyDown(e: KeyboardEvent, ctx: ToolContext): void {
    if (e.key === "Delete" || e.key === "Backspace") this.handleDelete(ctx);
    if (e.key === "Escape") ctx.clearSelection();
  }

  getHint(): string {
    return "Click to select, drag to move or box-select, Shift+click toggle, Delete remove";
  }

  private toggleSel(ctx: ToolContext): void {
    const was = ctx.selectedIds.includes(this.hitEntityId!);
    if (was) {
      const rem = ctx.selectedIds.filter(id => id !== this.hitEntityId);
      ctx.clearSelection();
      ctx.addToSelection(rem);
    } else ctx.addToSelection([this.hitEntityId!]);
  }

  private finishBox(ctx: ToolContext): void {
    const c = ctx.getCursorInSketch();
    const x1 = Math.min(this.dragStart!.x, c.x);
    const y1 = Math.min(this.dragStart!.y, c.y);
    const x2 = Math.max(this.dragStart!.x, c.x);
    const y2 = Math.max(this.dragStart!.y, c.y);
    const ids: EntityId[] = [];
    for (const [id, e] of ctx.model.entities) {
      const pts: EntityId[] = e.kind === "point" ? [id]
        : e.kind === "line" ? [e.start, e.end]
        : e.kind === "circle" ? [e.center]
        : e.kind === "arc" ? [e.center, e.startPoint, e.endPoint]
        : e.points;
      if (pts.some(pid => { const p = ctx.model.points.get(pid); return p && p.x >= x1 && p.x <= x2 && p.y >= y1 && p.y <= y2; }))
        ids.push(id);
    }
    ctx.clearSelection();
    ctx.addToSelection(ids);
  }

  private finishMove(ctx: ToolContext): void {
    const c = ctx.getCursorInSketch();
    const dx = c.x - this.dragStart!.x;
    const dy = c.y - this.dragStart!.y;
    if (dx === 0 && dy === 0) return;
    const entries = [...this.initPos];
    const cmd: Command = {
      description: `Move ${entries.length} point(s)`,
      execute: (m) => { let r = m; for (const [pid, p] of entries) r = updatePoint(r, pid, p.x + dx, p.y + dy); return r; },
      undo: (m) => { let r = m; for (const [pid, p] of entries) r = updatePoint(r, pid, p.x, p.y); return r; },
    };
    ctx.history.push(cmd);
  }

  private handleDelete(ctx: ToolContext): void {
    if (ctx.selectedIds.length === 0) return;
    const snap = cloneModel(ctx.model);
    const ids = [...ctx.selectedIds];
    const cmd: Command = {
      description: `Delete ${ids.length} entity(ies)`,
      execute: (m) => { let r = m; for (const id of ids) r = removeEntity(r, id); return r; },
      undo: () => snap,
    };
    ctx.history.push(cmd);
    ctx.setModel(cmd.execute(ctx.model));
    ctx.clearSelection();
  }

  private collectInit(ctx: ToolContext): void {
    this.initPos.clear();
    if (this.hitPointId) {
      const p = ctx.model.points.get(this.hitPointId);
      if (p) this.initPos.set(p.id, { x: p.x, y: p.y });
      return;
    }
    for (const id of ctx.selectedIds) {
      const e = ctx.model.entities.get(id);
      if (!e) continue;
      const pointIds: EntityId[] = e.kind === "point" ? [id]
        : e.kind === "line" ? [e.start, e.end]
        : e.kind === "circle" ? [e.center]
        : e.kind === "arc" ? [e.center, e.startPoint, e.endPoint]
        : e.points;
      for (const pid of pointIds) {
        const p = ctx.model.points.get(pid);
        if (p && !this.initPos.has(pid)) {
          this.initPos.set(pid, { x: p.x, y: p.y });
        }
      }
    }
  }

  private reset(): void {
    this.dragStart = null;
    this.dragMoved = false;
    this.isBoxSelect = false;
    this.hitEntityId = null;
    this.hitPointId = null;
    this.initPos.clear();
  }
}

function hitTest(c: { x: number; y: number }, m: SketchModel): HitResult | null {
  const t = HIT_THRESHOLD;
  for (const [, p] of m.points)
    if (Math.hypot(p.x - c.x, p.y - c.y) < t) return { entityId: p.id, pointId: p.id };
  for (const [id, e] of m.entities) {
    if (e.kind === "line") {
      const s = m.points.get(e.start), en = m.points.get(e.end);
      if (!s || !en) continue;
      if (p2sDist(c.x, c.y, s.x, s.y, en.x, en.y) < t) return { entityId: id };
    } else if (e.kind === "circle" || e.kind === "arc") {
      const cp = m.points.get(e.center);
      if (!cp) continue;
      if (Math.abs(Math.hypot(c.x - cp.x, c.y - cp.y) - e.radius) < t) return { entityId: id };
    }
  }
  return null;
}

function p2sDist(px: number, py: number, ax: number, ay: number, bx: number, by: number): number {
  const abx = bx - ax, aby = by - ay, l2 = abx * abx + aby * aby;
  if (l2 === 0) return Math.hypot(px - ax, py - ay);
  const t = Math.max(0, Math.min(1, ((px - ax) * abx + (py - ay) * aby) / l2));
  return Math.hypot(px - (ax + t * abx), py - (ay + t * aby));
}
