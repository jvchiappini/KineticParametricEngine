import * as THREE from "three";
import type { SketchTool, ToolContext } from "./types";
import type { Command } from "../model/SketchHistory";
import type { Constraint } from "../model/Constraint";
import type { SketchModel } from "../model/SketchModel";
import { addEntity, removeEntity, addConstraint, removeConstraint } from "../model/SketchModel";
import { makePoint, makeLine } from "../model/Entity";
import { generateConstraintId } from "../model/Constraint";
import { compoundCommand } from "../model/SketchHistory";

export class RectangleTool implements SketchTool {
  readonly name = "rectangle";
  readonly cursor = "crosshair";

  private state: "idle" | "placing" = "idle";
  private centerMode = false;
  private firstPoint: { x: number; y: number } | null = null;

  onActivate(_context: ToolContext): void { this.reset(); }
  onDeactivate(context: ToolContext): void { context.setPreview(null); this.reset(); }

  onPointerDown(_event: PointerEvent, context: ToolContext): void {
    if (this.state === "idle") {
      this.firstPoint = context.getSnapAwareCursor();
      this.state = "placing";
    } else {
      this.createRectangle(context);
      this.reset();
      context.setPreview(null);
    }
  }

  onPointerMove(_event: PointerEvent, context: ToolContext): void {
    if (this.state === "placing" && this.firstPoint) {
      const p2 = context.getSnapAwareCursor();
      const preview = buildRectPreview(this.firstPoint, p2, this.centerMode);
      context.setPreview(preview);
    }
  }

  onPointerUp(_event: PointerEvent, _context: ToolContext): void {}

  onKeyDown(event: KeyboardEvent, context: ToolContext): void {
    if (event.key === "c" || event.key === "C") {
      this.centerMode = !this.centerMode;
    }
    if (event.key === "Escape") {
      context.setPreview(null);
      this.reset();
    }
    if (event.key === "Enter" && this.state === "placing") {
      this.createRectangle(context);
      context.setPreview(null);
      this.reset();
    }
  }

  getHint(): string {
    if (this.state === "idle") return `Click to place ${this.centerMode ? "center" : "first corner"} (C: toggle mode)`;
    return `Click or Enter to place ${this.centerMode ? "corner" : "opposite corner"} (C: toggle mode)`;
  }

  private createRectangle(context: ToolContext): void {
    const p1 = this.firstPoint!;
    const p2 = context.getSnapAwareCursor();

    let x1: number, y1: number, x2: number, y2: number;

    if (this.centerMode) {
      x1 = p1.x - Math.abs(p2.x - p1.x);
      y1 = p1.y - Math.abs(p2.y - p1.y);
      x2 = p1.x + Math.abs(p2.x - p1.x);
      y2 = p1.y + Math.abs(p2.y - p1.y);
    } else {
      x1 = Math.min(p1.x, p2.x);
      y1 = Math.min(p1.y, p2.y);
      x2 = Math.max(p1.x, p2.x);
      y2 = Math.max(p1.y, p2.y);
    }

    const ptTL = makePoint(x1, y1);
    const ptTR = makePoint(x2, y1);
    const ptBR = makePoint(x2, y2);
    const ptBL = makePoint(x1, y2);

    const top = makeLine(ptTL.id, ptTR.id);
    const right = makeLine(ptTR.id, ptBR.id);
    const bottom = makeLine(ptBR.id, ptBL.id);
    const left = makeLine(ptBL.id, ptTL.id);

    const coincidentTL: Constraint = {
      id: generateConstraintId(), kind: "coincident",
      pointA: ptTL.id, pointB: ptTL.id, entities: [top.id, left.id], satisfied: true,
    };
    const coincidentTR: Constraint = {
      id: generateConstraintId(), kind: "coincident",
      pointA: ptTR.id, pointB: ptTR.id, entities: [top.id, right.id], satisfied: true,
    };
    const coincidentBR: Constraint = {
      id: generateConstraintId(), kind: "coincident",
      pointA: ptBR.id, pointB: ptBR.id, entities: [right.id, bottom.id], satisfied: true,
    };
    const coincidentBL: Constraint = {
      id: generateConstraintId(), kind: "coincident",
      pointA: ptBL.id, pointB: ptBL.id, entities: [bottom.id, left.id], satisfied: true,
    };

    const cmd = compoundCommand("Create rectangle", [
      { description: "Add TL corner", execute: (m) => addEntity(m, ptTL), undo: (m) => removeEntity(m, ptTL.id) },
      { description: "Add TR corner", execute: (m) => addEntity(m, ptTR), undo: (m) => removeEntity(m, ptTR.id) },
      { description: "Add BR corner", execute: (m) => addEntity(m, ptBR), undo: (m) => removeEntity(m, ptBR.id) },
      { description: "Add BL corner", execute: (m) => addEntity(m, ptBL), undo: (m) => removeEntity(m, ptBL.id) },
      { description: "Add top edge", execute: (m) => addEntity(m, top), undo: (m) => removeEntity(m, top.id) },
      { description: "Add right edge", execute: (m) => addEntity(m, right), undo: (m) => removeEntity(m, right.id) },
      { description: "Add bottom edge", execute: (m) => addEntity(m, bottom), undo: (m) => removeEntity(m, bottom.id) },
      { description: "Add left edge", execute: (m) => addEntity(m, left), undo: (m) => removeEntity(m, left.id) },
      { description: "Constrain TL", execute: (m) => addConstraint(m, coincidentTL), undo: (m) => removeConstraint(m, coincidentTL.id) },
      { description: "Constrain TR", execute: (m) => addConstraint(m, coincidentTR), undo: (m) => removeConstraint(m, coincidentTR.id) },
      { description: "Constrain BR", execute: (m) => addConstraint(m, coincidentBR), undo: (m) => removeConstraint(m, coincidentBR.id) },
      { description: "Constrain BL", execute: (m) => addConstraint(m, coincidentBL), undo: (m) => removeConstraint(m, coincidentBL.id) },
    ]);

    context.history.push(cmd);
    context.setModel(cmd.execute(context.model));
  }

  private reset(): void {
    this.state = "idle";
    this.firstPoint = null;
  }
}

function buildRectPreview(p1: { x: number; y: number }, p2: { x: number; y: number }, center: boolean): THREE.Line {
  let x1: number, y1: number, x2: number, y2: number;
  if (center) {
    x1 = p1.x - Math.abs(p2.x - p1.x); y1 = p1.y - Math.abs(p2.y - p1.y);
    x2 = p1.x + Math.abs(p2.x - p1.x); y2 = p1.y + Math.abs(p2.y - p1.y);
  } else {
    x1 = Math.min(p1.x, p2.x); y1 = Math.min(p1.y, p2.y);
    x2 = Math.max(p1.x, p2.x); y2 = Math.max(p1.y, p2.y);
  }
  const pts = [x1, y1, 0, x2, y1, 0, x2, y2, 0, x1, y2, 0, x1, y1, 0];
  return buildLine(pts);
}

function buildLine(pts: number[]): THREE.Line {
  const geo = new THREE.BufferGeometry();
  geo.setAttribute("position", new THREE.BufferAttribute(new Float32Array(pts), 3));
  const mat = new THREE.LineDashedMaterial({ color: 0xffffff, dashSize: 4, gapSize: 3, transparent: true, opacity: 0.4 });
  const line = new THREE.Line(geo, mat);
  line.computeLineDistances();
  return line;
}
