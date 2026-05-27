import * as THREE from "three";
import type { SketchTool, ToolContext } from "./types";
import type { Command } from "../model/SketchHistory";
import type { SketchModel } from "../model/SketchModel";
import { addEntity, removeEntity } from "../model/SketchModel";
import { makePoint, makeCircle } from "../model/Entity";
import { compoundCommand } from "../model/SketchHistory";

export class CircleTool implements SketchTool {
  readonly name = "circle";
  readonly cursor = "crosshair";

  private state: "idle" | "centerPlacing" | "radiusPlacing" | "threePoint1" | "threePoint2" = "idle";
  private threePointMode = false;
  private center: { x: number; y: number } | null = null;
  private p1: { x: number; y: number } | null = null;
  private p2: { x: number; y: number } | null = null;

  onActivate(_context: ToolContext): void { this.reset(); }
  onDeactivate(context: ToolContext): void { context.setPreview(null); this.reset(); }

  onPointerDown(_event: PointerEvent, context: ToolContext): void {
    const cursor = context.getSnapAwareCursor();
    if (this.threePointMode) {
      this.handleThreePointClick(cursor, context);
    } else {
      this.handleCenterRadiusClick(cursor, context);
    }
  }

  onPointerMove(_event: PointerEvent, context: ToolContext): void {
    if (this.state === "centerPlacing" && this.center) {
      const c = context.getSnapAwareCursor();
      const r = Math.hypot(c.x - this.center.x, c.y - this.center.y);
      context.setPreview(buildCirclePreview(this.center.x, this.center.y, r));
    }
  }

  onPointerUp(_event: PointerEvent, _context: ToolContext): void {}

  onKeyDown(event: KeyboardEvent, context: ToolContext): void {
    if (event.key === "3") {
      this.threePointMode = !this.threePointMode;
      this.reset();
    }
    if (event.key === "Escape") {
      context.setPreview(null);
      this.reset();
    }
    if (event.key === "Enter" && this.state === "radiusPlacing") {
      this.createCircleFromCenter(context, context.getSnapAwareCursor());
    }
  }

  getHint(): string {
    if (this.threePointMode) {
      const stateMap: Record<string, string> = {
        idle: "Click first point on circumference (3-point mode, 3: toggle)",
        threePoint1: "Click second point on circumference",
        threePoint2: "Click third point on circumference",
      };
      return stateMap[this.state] ?? "";
    }
    const stateMap: Record<string, string> = {
      idle: "Click circle center (3: toggle 3-point mode)",
      centerPlacing: "Click or Enter to set radius",
    };
    return stateMap[this.state] ?? "";
  }

  private handleCenterRadiusClick(cursor: { x: number; y: number }, context: ToolContext): void {
    if (this.state === "idle") {
      this.center = cursor;
      this.state = "centerPlacing";
    } else if (this.state === "centerPlacing") {
      this.createCircleFromCenter(context, cursor);
    }
  }

  private handleThreePointClick(cursor: { x: number; y: number }, context: ToolContext): void {
    if (this.state === "idle") {
      this.p1 = cursor;
      this.state = "threePoint1";
    } else if (this.state === "threePoint1") {
      this.p2 = cursor;
      this.state = "threePoint2";
    } else if (this.state === "threePoint2") {
      this.createCircleFromThreePoints(context, cursor);
    }
  }

  private createCircleFromCenter(context: ToolContext, radiusPoint: { x: number; y: number }): void {
    const centerPt = makePoint(this.center!.x, this.center!.y);
    const r = Math.hypot(radiusPoint.x - this.center!.x, radiusPoint.y - this.center!.y);
    const circle = makeCircle(centerPt.id, r);

    const cmd = compoundCommand("Create circle", [
      { description: "Add center point", execute: (m) => addEntity(m, centerPt), undo: (m) => removeEntity(m, centerPt.id) },
      { description: "Add circle", execute: (m) => addEntity(m, circle), undo: (m) => removeEntity(m, circle.id) },
    ]);

    context.history.push(cmd);
    context.setModel(cmd.execute(context.model));
    this.reset();
  }

  private createCircleFromThreePoints(
    context: ToolContext, p3: { x: number; y: number }
  ): void {
    const result = circleFromThreePoints(
      this.p1!.x, this.p1!.y,
      this.p2!.x, this.p2!.y,
      p3.x, p3.y
    );
    if (!result) return;

    const centerPt = makePoint(result.cx, result.cy);
    const circle = makeCircle(centerPt.id, result.r);

    const cmd = compoundCommand("Create circle (3-point)", [
      { description: "Add center point", execute: (m) => addEntity(m, centerPt), undo: (m) => removeEntity(m, centerPt.id) },
      { description: "Add circle", execute: (m) => addEntity(m, circle), undo: (m) => removeEntity(m, circle.id) },
    ]);

    context.history.push(cmd);
    context.setModel(cmd.execute(context.model));
    this.reset();
  }

  private reset(): void {
    this.state = "idle";
    this.center = null;
    this.p1 = null;
    this.p2 = null;
  }
}

function buildCirclePreview(cx: number, cy: number, r: number): THREE.Line {
  const pts: number[] = [];
  const steps = 48;
  for (let i = 0; i <= steps; i++) {
    const theta = (i / steps) * Math.PI * 2;
    pts.push(cx + r * Math.cos(theta), cy + r * Math.sin(theta), 0);
  }
  const geo = new THREE.BufferGeometry();
  geo.setAttribute("position", new THREE.BufferAttribute(new Float32Array(pts), 3));
  const mat = new THREE.LineDashedMaterial({ color: 0xffffff, dashSize: 4, gapSize: 3, transparent: true, opacity: 0.4 });
  const line = new THREE.Line(geo, mat);
  line.computeLineDistances();
  return line;
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
