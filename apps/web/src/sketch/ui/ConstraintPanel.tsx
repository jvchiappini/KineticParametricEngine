/**
 * Right-side panel that lists active constraints and offers buttons
 * to add new constraints based on the current selection.
 *
 * @packageDocumentation
 */

import type { JSX } from "react";
import type { SketchModel } from "../model/SketchModel";
import { allConstraints } from "../model/SketchModel";

interface ConstraintPanelProps {
  model: SketchModel;
  selectedIds: string[];
  onAddConstraint: (kind: string, entityIds: string[], value?: number) => void;
  onRemoveConstraint: (constraintId: string) => void;
}

const CONSTRAINT_LABELS: Record<string, string> = {
  coincident: "Coincident",
  horizontal: "Horizontal",
  vertical: "Vertical",
  parallel: "Parallel",
  perpendicular: "Perpendicular",
  equal: "Equal",
  symmetric: "Symmetric",
  tangent: "Tangent",
  collinear: "Collinear",
  fixed: "Fixed",
  lengthDim: "Length",
  angleDim: "Angle",
  radiusDim: "Radius",
};

/**
 * Determine which constraint types are applicable for the current
 * selection.
 */
function applicableConstraints(selectedIds: string[], model: SketchModel): string[] {
  if (selectedIds.length === 0) return [];

  const entities = selectedIds
    .map((id) => model.entities.get(id))
    .filter(Boolean);

  const kinds = entities.map((e) => e!.kind);
  const uniqueKinds = new Set(kinds);

  if (selectedIds.length === 1) {
    const kind = kinds[0];
    if (kind === "line") return ["horizontal", "vertical", "fixed", "lengthDim"];
    if (kind === "circle" || kind === "arc") return ["fixed", "radiusDim"];
    if (kind === "point") return ["fixed"];
    return ["fixed"];
  }

  if (selectedIds.length === 2 && uniqueKinds.size === 1 && uniqueKinds.has("line")) {
    return ["parallel", "perpendicular", "equal", "angleDim"];
  }

  if (selectedIds.length === 2) {
    const allPoints = entities.every((e) => e!.kind === "point");
    if (allPoints) return ["coincident"];
    return ["coincident", "tangent"];
  }

  return [];
}

export function ConstraintPanel(props: ConstraintPanelProps): JSX.Element {
  const { model, selectedIds, onAddConstraint, onRemoveConstraint } = props;

  const constraints = allConstraints(model);
  const dof = constraints.filter((c) => !c.satisfied).length;
  const applicable = applicableConstraints(selectedIds, model);

  const containerStyle: React.CSSProperties = {
    width: "220px",
    background: "#1a1a2e",
    borderLeft: "1px solid #333",
    padding: "8px",
    display: "flex",
    flexDirection: "column",
    gap: "8px",
    fontSize: "12px",
    color: "#ccc",
    fontFamily: "inherit",
    overflowY: "auto",
  };

  const sectionTitle: React.CSSProperties = {
    fontSize: "11px",
    fontWeight: 700,
    textTransform: "uppercase",
    color: "#888",
    margin: "4px 0",
  };

  return (
    <div style={containerStyle}>
      <div style={sectionTitle}>Constraints ({constraints.length})</div>

      {constraints.length === 0 && (
        <div style={{ color: "#666", fontStyle: "italic" }}>
          No constraints
        </div>
      )}

      <ul style={{
        listStyle: "none",
        padding: 0,
        margin: 0,
        display: "flex",
        flexDirection: "column",
        gap: "2px",
      }}>
        {constraints.map((c) => (
          <li
            key={c.id}
            style={{
              display: "flex",
              justifyContent: "space-between",
              alignItems: "center",
              padding: "3px 6px",
              background: "#222",
              borderRadius: "4px",
            }}
          >
            <span>
              {CONSTRAINT_LABELS[c.kind] ?? c.kind}
              {"value" in c ? `: ${c.value}` : ""}
            </span>
            <button
              title="Remove constraint"
              onClick={() => onRemoveConstraint(c.id)}
              style={{
                background: "none",
                border: "none",
                color: "#f55",
                cursor: "pointer",
                fontSize: "13px",
                padding: "0 2px",
              }}
            >
              ×
            </button>
          </li>
        ))}
      </ul>

      <div style={sectionTitle}>Add Constraint</div>

      {applicable.length === 0 && (
        <div style={{ color: "#666", fontStyle: "italic" }}>
          Select entities
        </div>
      )}

      <div style={{ display: "flex", flexWrap: "wrap", gap: "4px" }}>
        {applicable.map((kind) => (
          <button
            key={kind}
            onClick={() => onAddConstraint(kind, selectedIds)}
            style={{
              padding: "3px 8px",
              border: "1px solid #444",
              borderRadius: "4px",
              background: "#2a2a3e",
              color: "#aaa",
              cursor: "pointer",
              fontSize: "11px",
              fontFamily: "inherit",
            }}
          >
            {CONSTRAINT_LABELS[kind] ?? kind}
          </button>
        ))}
      </div>

      <div style={{ marginTop: "auto", borderTop: "1px solid #333", paddingTop: "8px" }}>
        <div style={sectionTitle}>Degrees of Freedom</div>
        <div style={{ fontSize: "20px", fontWeight: 700, color: dof === 0 ? "#4caf50" : "#ff9800" }}>
          {dof}
        </div>
      </div>
    </div>
  );
}
