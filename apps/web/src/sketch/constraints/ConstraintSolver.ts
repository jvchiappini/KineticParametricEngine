import type { SketchModel } from "../model/SketchModel";
import type { EntityId } from "../model/Entity";
import type { Constraint } from "../model/Constraint";
import { cloneModel } from "../model/SketchModel";
import { ConstraintGraph } from "./ConstraintGraph";
import { evaluateConstraint as evaluateCoincident } from "./constraints/Coincident";
import { evaluateConstraint as evaluateHorizontal } from "./constraints/Horizontal";
import { evaluateConstraint as evaluateVertical } from "./constraints/Vertical";
import { evaluateConstraint as evaluateParallel } from "./constraints/Parallel";
import { evaluateConstraint as evaluatePerpendicular } from "./constraints/Perpendicular";
import { evaluateConstraint as evaluateEqual } from "./constraints/Equal";
import { evaluateConstraint as evaluateSymmetric } from "./constraints/Symmetric";
import { evaluateConstraint as evaluateTangent } from "./constraints/Tangent";
import { evaluateConstraint as evaluateCollinear } from "./constraints/Collinear";
import { evaluateConstraint as evaluateFixed } from "./constraints/Fixed";
import { evaluateConstraint as evaluateLengthDim } from "./constraints/LengthDim";
import { evaluateConstraint as evaluateAngleDim } from "./constraints/AngleDim";
import { evaluateConstraint as evaluateRadiusDim } from "./constraints/RadiusDim";

const EVALUATORS: Record<string, (c: Constraint, m: SketchModel, v: Map<string, number>, g: Map<string, number>) => number> = {
  coincident: evaluateCoincident as any,
  horizontal: evaluateHorizontal as any,
  vertical: evaluateVertical as any,
  parallel: evaluateParallel as any,
  perpendicular: evaluatePerpendicular as any,
  equal: evaluateEqual as any,
  symmetric: evaluateSymmetric as any,
  tangent: evaluateTangent as any,
  collinear: evaluateCollinear as any,
  fixed: evaluateFixed as any,
  lengthDim: evaluateLengthDim as any,
  angleDim: evaluateAngleDim as any,
  radiusDim: evaluateRadiusDim as any,
};

type KeyFn = (id: EntityId) => string;
const keyX: KeyFn = (id) => `${id}.x`;
const keyY: KeyFn = (id) => `${id}.y`;
const keyRadius: KeyFn = (id) => `${id}.radius`;
const keyStartAngle: KeyFn = (id) => `${id}.startAngle`;
const keyEndAngle: KeyFn = (id) => `${id}.endAngle`;

function buildVariables(model: SketchModel): Map<string, number> {
  const vars = new Map<string, number>();

  for (const point of model.points.values()) {
    if (!point.fixed) {
      vars.set(keyX(point.id), point.x);
      vars.set(keyY(point.id), point.y);
    }
  }

  for (const entity of model.entities.values()) {
    if (entity.kind === "circle") {
      vars.set(keyRadius(entity.id), entity.radius);
    } else if (entity.kind === "arc") {
      vars.set(keyRadius(entity.id), entity.radius);
      vars.set(keyStartAngle(entity.id), entity.startAngle);
      vars.set(keyEndAngle(entity.id), entity.endAngle);
    }
  }

  return vars;
}

function applyVariables(model: SketchModel, variables: Map<string, number>): SketchModel {
  const next = cloneModel(model);

  for (const [key, value] of variables) {
    const dotIdx = key.lastIndexOf(".");
    const entityId = key.slice(0, dotIdx);
    const prop = key.slice(dotIdx + 1);

    if (prop === "x" || prop === "y") {
      const point = next.points.get(entityId);
      if (point) {
        const updated = { ...point, [prop]: value };
        next.points.set(entityId, updated);
        next.entities.set(entityId, updated);
      }
    } else if (prop === "radius" || prop === "startAngle" || prop === "endAngle") {
      const entity = next.entities.get(entityId);
      if (entity) {
        next.entities.set(entityId, { ...entity, [prop]: value } as any);
      }
    }
  }

  return next;
}

function isSatisfied(constraint: Constraint, model: SketchModel, variables: Map<string, number>, epsilon: number): boolean {
  const evaluator = EVALUATORS[constraint.kind];
  if (!evaluator) return true;
  const grads = new Map<string, number>();
  const error = Math.abs(evaluator(constraint, model, variables, grads));
  return error < epsilon;
}

export interface SolveOptions {
  maxIterations?: number;
  epsilon?: number;
  stepSize?: number;
}

export interface SolveResult {
  model: SketchModel;
  converged: boolean;
  iterations: number;
  maxError: number;
}

const DEFAULTS = {
  maxIterations: 500,
  epsilon: 1e-8,
  stepSize: 0.1,
};

/**
 * Solves all constraints in the sketch using Gauss-Seidel / gradient descent.
 *
 * 1. Extracts free variables (non-fixed point coordinates, entity parameters)
 * 2. Iteratively computes constraint error and gradient for each constraint
 * 3. Applies corrections: variable -= error * gradient * stepSize
 * 4. Repeats until convergence or max iterations
 * 5. Marks constraints as satisfied/unsatisfied and returns the updated model
 *
 * @param model - The sketch model to solve
 * @param options - Optional solver parameters
 * @returns Updated model with convergence info
 */
export function solveConstraints(
  model: SketchModel,
  options?: SolveOptions
): SolveResult {
  const maxIterations = options?.maxIterations ?? DEFAULTS.maxIterations;
  const epsilon = options?.epsilon ?? DEFAULTS.epsilon;
  const stepSize = options?.stepSize ?? DEFAULTS.stepSize;

  const variables = buildVariables(model);
  if (variables.size === 0) {
    const allSatisfied = true;
    let nextModel = cloneModel(model);
    for (const [cId, constraint] of nextModel.constraints) {
      nextModel.constraints.set(cId, { ...constraint, satisfied: true });
    }
    return { model: nextModel, converged: true, iterations: 0, maxError: 0 };
  }

  let iteration = 0;
  let converged = false;
  let maxError = 0;
  let nextModel = model;

  for (iteration = 0; iteration < maxIterations; iteration++) {
    maxError = 0;

    for (const [cId, constraint] of nextModel.constraints) {
      const evaluator = EVALUATORS[constraint.kind];
      if (!evaluator) continue;

      const gradients = new Map<string, number>();
      const error = evaluator(constraint, nextModel, variables, gradients);

      const absError = Math.abs(error);
      if (absError > maxError) {
        maxError = absError;
      }

      if (absError > epsilon) {
        for (const [varKey, grad] of gradients) {
          const current = variables.get(varKey);
          if (current !== undefined) {
            variables.set(varKey, current - error * grad * stepSize);
          }
        }
      }
    }

    if (maxError < epsilon) {
      converged = true;
      break;
    }
  }

  nextModel = applyVariables(nextModel, variables);

  const finalGradients = new Map<string, number>();
  for (const [cId, constraint] of nextModel.constraints) {
    const evaluator = EVALUATORS[constraint.kind];
    if (!evaluator) {
      nextModel.constraints.set(cId, { ...constraint, satisfied: true });
      continue;
    }
    finalGradients.clear();
    const error = Math.abs(evaluator(constraint, nextModel, variables, finalGradients));
    nextModel.constraints.set(cId, { ...constraint, satisfied: error < epsilon * 10 });
  }

  return { model: nextModel, converged, iterations: iteration + 1, maxError };
}
