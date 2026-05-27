import type { SketchTool, ToolContext } from "./types";
import type { Command } from "../model/SketchHistory";
import type { SketchModel } from "../model/SketchModel";
import { addEntity, removeEntity } from "../model/SketchModel";
import { makePoint, makeArc } from "../model/Entity";
import { compoundCommand } from "../model/SketchHistory";

/**
 * Arc tool with two modes:
 * - Center+start+end (default): click center, click start angle, click end angle.
 * - 3-point (toggle with 3): click start, midpoint, then end on the arc.
 */
export class ArcTool implements SketchTool {
  readonly name = "arc";
  readonly cursor = "crosshair";

  private state:
    | "idle"
    | "centerPlaced"
    | "startPlaced"
    | "threePt1"
    | "threePt2"
    = "idle";
  private threePointMode = false;
  private center: { x: number; y: number } | null = null;
  private startPt: { x: number; y: number } | null = null;
  private p1: { x: number; y: number } | null = null;
  private p2: { x: number; y: number } | null = null;

  onActivate(_context: ToolContext): void { this.reset(); }
  onDeactivate(context: ToolContext): void { context.setPreview(null); this.reset(); }

  onPointerDown(_event: PointerEvent, context: ToolContext): void {
    const cursor = context.getSnapAwareCursor();

    if (this.threePointMode) {
      this.handleThreePointClick(cursor, context);
    } else {
      this.handleCenterStartEndClick(cursor, context);
    }
  }

  onPointerMove(_event: PointerEvent, context: ToolContext): void {
    if (this.state !== "idle") {
      context.onRequestRender();
    }
  }

  onPointerUp(_event: PointerEvent, _context: ToolContext): void {
  }

  onKeyDown(event: KeyboardEvent, context: ToolContext): void {
    if (event.key === "3") {
      this.threePointMode = !this.threePointMode;
      this.reset();
      context.onRequestRender();
    }
    if (event.key === "Escape") {
      this.reset();
      context.onRequestRender();
    }
    if (event.key === "Enter") {
      const cursor = context.getSnapAwareCursor();
      if (this.state === "startPlaced") {
        this.createArcCenterMode(context, cursor);
      } else if (this.state === "threePt2") {
        this.createArcThreePoint(context, cursor);
      }
    }
  }

  getHint(): string {
    if (this.threePointMode) {
      const hints: Record<string, string> = {
        idle: "Click arc start point (3: toggle 3-point mode)",
        threePt1: "Click midpoint on arc",
        threePt2: "Click or Enter to set arc end",
      };
      return hints[this.state] ?? "";
    }
    const hints: Record<string, string> = {
      idle: "Click arc center (3: toggle 3-point mode)",
      centerPlaced: "Click start angle point",
      startPlaced: "Click or Enter to set end angle",
    };
    return hints[this.state] ?? "";
  }

  private handleCenterStartEndClick(
    cursor: { x: number; y: number }, context: ToolContext
  ): void {
    if (this.state === "idle") {
      this.center = cursor;
      this.state = "centerPlaced";
    } else if (this.state === "centerPlaced") {
      this.startPt = cursor;
      this.state = "startPlaced";
    } else if (this.state === "startPlaced") {
      this.createArcCenterMode(context, cursor);
    }
  }

  private handleThreePointClick(
    cursor: { x: number; y: number }, context: ToolContext
  ): void {
    if (this.state === "idle") {
      this.p1 = cursor;
      this.state = "threePt1";
    } else if (this.state === "threePt1") {
      this.p2 = cursor;
      this.state = "threePt2";
    } else if (this.state === "threePt2") {
      this.createArcThreePoint(context, cursor);
    }
  }

  private createArcCenterMode(
    context: ToolContext, endCursor: { x: number; y: number }
  ): void {
    const cx = this.center!.x;
    const cy = this.center!.y;
    const r = Math.hypot(this.startPt!.x - cx, this.startPt!.y - cy);
    const startAngle = Math.atan2(this.startPt!.y - cy, this.startPt!.x - cx);
    const endAngle = Math.atan2(endCursor.y - cy, endCursor.x - cx);
    const arcStart = makePoint(
      cx + r * Math.cos(startAngle),
      cy + r * Math.sin(startAngle)
    );
    const arcEnd = makePoint(
      cx + r * Math.cos(endAngle),
      cy + r * Math.sin(endAngle)
    );
    const centerPt = makePoint(cx, cy);

    const arc = makeArc(centerPt.id, r, startAngle, endAngle, arcStart.id, arcEnd.id);

    const cmd = compoundCommand("Create arc", [
      { description: "Add center", execute: (m) => addEntity(m, centerPt), undo: (m) => removeEntity(m, centerPt.id) },
      { description: "Add arc start", execute: (m) => addEntity(m, arcStart), undo: (m) => removeEntity(m, arcStart.id) },
      { description: "Add arc end", execute: (m) => addEntity(m, arcEnd), undo: (m) => removeEntity(m, arcEnd.id) },
      { description: "Add arc", execute: (m) => addEntity(m, arc), undo: (m) => removeEntity(m, arc.id) },
    ]);

    context.history.push(cmd);
    context.setModel(cmd.execute(context.model));
    this.reset();
  }

  private createArcThreePoint(
    context: ToolContext, p3: { x: number; y: number }
  ): void {
    const result = circleFromThreePoints(
      this.p1!.x, this.p1!.y,
      this.p2!.x, this.p2!.y,
      p3.x, p3.y
    );
    if (!result) return;

    const startAngle = Math.atan2(this.p1!.y - result.cy, this.p1!.x - result.cx);
    const endAngle = Math.atan2(p3.y - result.cy, p3.x - result.cx);
    const midAngle = Math.atan2(this.p2!.y - result.cy, this.p2!.x - result.cx);

    let sweep = endAngle - startAngle;
    if (sweep < 0) sweep += Math.PI * 2;
    const midRel = (midAngle - startAngle + Math.PI * 2) % (Math.PI * 2);
    if (!isAngleBetween(midRel, 0, sweep)) {
      // Arc should go the other way; swap start and end
      const adj: { t: number; x: number; y: number }[] = [
        { t: startAngle, x: this.p1!.x, y: this.p1!.y },
        { t: endAngle, x: p3.x, y: p3.y },
      ];
      adj.sort((a, b) => {
        let da = (a.t - startAngle + Math.PI * 2) % (Math.PI * 2);
        let db = (b.t - startAngle + Math.PI * 2) % (Math.PI * 2);
        return da - db;
      });
    }

    const cx = result.cx;
    const cy = result.cy;
    const r = result.r;
    const centerPt = makePoint(cx, cy);
    const arcStartPt = makePoint(this.p1!.x, this.p1!.y);
    const arcEndPt = makePoint(p3.x, p3.y);
    const arc = makeArc(centerPt.id, r, startAngle, endAngle, arcStartPt.id, arcEndPt.id);

    const cmd = compoundCommand("Create arc (3-point)", [
      { description: "Add center", execute: (m) => addEntity(m, centerPt), undo: (m) => removeEntity(m, centerPt.id) },
      { description: "Add arc start", execute: (m) => addEntity(m, arcStartPt), undo: (m) => removeEntity(m, arcStartPt.id) },
      { description: "Add arc end", execute: (m) => addEntity(m, arcEndPt), undo: (m) => removeEntity(m, arcEndPt.id) },
      { description: "Add arc", execute: (m) => addEntity(m, arc), undo: (m) => removeEntity(m, arc.id) },
    ]);

    context.history.push(cmd);
    context.setModel(cmd.execute(context.model));
    this.reset();
  }

  private reset(): void {
    this.state = "idle";
    this.center = null;
    this.startPt = null;
    this.p1 = null;
    this.p2 = null;
  }
}

function circleFromThreePoints(
  x1: number, y1: number, x2: number, y2: number, x3: number, y3: number
): { cx: number; cy: number; r: number } | null {
  const d = 2 * (x1 * (y2 - y3) + x2 * (y3 - y1) + x3 * (y1 - y2));
  if (Math.abs(d) < 1e-10) return null;
  const ux = ((x1 * x1 + y1 * y1) * (y2 - y3) + (x2 * x2 + y2 * y2) * (y3 - y1) + (x3 * x3 + y3 * y3) * (y1 - y2)) / d;
  const uy = ((x1 * x1 + y1 * y1) * (x3 - x2) + (x2 * x2 + y2 * y2) * (x1 - x3) + (x3 * x3 + y3 * y3) * (x2 - x1)) / d;
  const r = Math.hypot(x1 - ux, y1 - uy);
  return { cx: ux, cy: uy, r };
}

function isAngleBetween(a: number, start: number, end: number): boolean {
  if (start <= end) return a >= start && a <= end;
  return a >= start || a <= end;
}
