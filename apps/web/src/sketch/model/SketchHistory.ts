import type { SketchModel } from "./SketchModel";

export interface Command {
  description: string;
  execute: (model: SketchModel) => SketchModel;
  undo: (model: SketchModel) => SketchModel;
}

const MAX_HISTORY = 200;

export class SketchHistory {
  private undoStack: Command[] = [];
  private redoStack: Command[] = [];

  push(command: Command): void {
    this.undoStack.push(command);
    if (this.undoStack.length > MAX_HISTORY) {
      this.undoStack.shift();
    }
    this.redoStack = [];
  }

  undo(model: SketchModel): { model: SketchModel; command?: Command } {
    const cmd = this.undoStack.pop();
    if (!cmd) return { model };
    this.redoStack.push(cmd);
    return { model: cmd.undo(model), command: cmd };
  }

  redo(model: SketchModel): { model: SketchModel; command?: Command } {
    const cmd = this.redoStack.pop();
    if (!cmd) return { model };
    this.undoStack.push(cmd);
    return { model: cmd.execute(model), command: cmd };
  }

  canUndo(): boolean {
    return this.undoStack.length > 0;
  }

  canRedo(): boolean {
    return this.redoStack.length > 0;
  }

  clear(): void {
    this.undoStack = [];
    this.redoStack = [];
  }

  getUndoDescription(): string | undefined {
    return this.undoStack[this.undoStack.length - 1]?.description;
  }

  getRedoDescription(): string | undefined {
    return this.redoStack[this.redoStack.length - 1]?.description;
  }
}

export function compoundCommand(
  description: string,
  commands: Command[]
): Command {
  return {
    description,
    execute: (m: SketchModel) => {
      let model = m;
      for (const cmd of commands) {
        model = cmd.execute(model);
      }
      return model;
    },
    undo: (m: SketchModel) => {
      let model = m;
      for (let i = commands.length - 1; i >= 0; i--) {
        model = commands[i].undo(model);
      }
      return model;
    },
  };
}

import { addEntity, removeEntity, addConstraint, removeConstraint } from "./SketchModel";
import type { Entity, EntityId } from "./Entity";
import type { Constraint } from "./Constraint";

export function addEntityCommand(entity: Entity): Command {
  const id = entity.id;
  return {
    description: `Add ${entity.kind}`,
    execute: (m: SketchModel) => addEntity(m, entity),
    undo: (m: SketchModel) => removeEntity(m, id),
  };
}

export function removeEntityCommand(
  entityId: EntityId,
  snapshot: SketchModel
): Command {
  return {
    description: `Remove entity ${entityId}`,
    execute: (m: SketchModel) => {
      const ent = snapshot.entities.get(entityId);
      if (!ent) return m;
      let model = addEntity(m, ent);
      if (ent.kind === "point" && snapshot.points.has(entityId)) {
        const pt = snapshot.points.get(entityId)!;
        model = addEntity(model, pt);
      }
      return model;
    },
    undo: (m: SketchModel) => removeEntity(m, entityId),
  };
}

export function addConstraintCommand(constraint: Constraint): Command {
  const id = constraint.id;
  return {
    description: `Add ${constraint.kind} constraint`,
    execute: (m: SketchModel) => addConstraint(m, constraint),
    undo: (m: SketchModel) => removeConstraint(m, id),
  };
}
