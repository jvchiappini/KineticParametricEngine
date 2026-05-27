/**
 * Left-side toolbar showing sketch tool buttons.
 *
 * Each button displays a label and its keyboard shortcut letter.
 * The active tool is visually highlighted.
 *
 * @packageDocumentation
 */

import type { JSX } from "react";

interface ToolDef {
  id: string;
  label: string;
  shortcut: string;
}

const TOOLS: ToolDef[] = [
  { id: "select", label: "Select", shortcut: "S" },
  { id: "line", label: "Line", shortcut: "L" },
  { id: "rectangle", label: "Rectangle", shortcut: "R" },
  { id: "circle", label: "Circle", shortcut: "C" },
  { id: "arc", label: "Arc", shortcut: "A" },
  { id: "polyline", label: "Polyline", shortcut: "P" },
  { id: "dimension", label: "Dimension", shortcut: "D" },
];

interface SketchToolbarProps {
  activeTool: string;
  onToolChange: (tool: string) => void;
}

export function SketchToolbar(props: SketchToolbarProps): JSX.Element {
  const { activeTool, onToolChange } = props;

  return (
    <div style={{
      display: "flex",
      flexDirection: "column",
      gap: "2px",
      padding: "4px",
      background: "#1a1a2e",
      borderRight: "1px solid #333",
    }}>
      {TOOLS.map((t) => {
        const isActive = activeTool === t.id;
        return (
          <button
            key={t.id}
            title={`${t.label} (${t.shortcut})`}
            onClick={() => onToolChange(t.id)}
            style={{
              display: "flex",
              flexDirection: "column",
              alignItems: "center",
              justifyContent: "center",
              width: "48px",
              height: "48px",
              border: isActive ? "2px solid #4a9eff" : "2px solid transparent",
              borderRadius: "6px",
              background: isActive ? "#2a3a5e" : "transparent",
              color: isActive ? "#4a9eff" : "#aaa",
              cursor: "pointer",
              fontSize: "10px",
              fontFamily: "inherit",
              transition: "background 0.15s, border-color 0.15s",
            }}
          >
            <span style={{ fontSize: "14px", fontWeight: 600 }}>
              {t.shortcut}
            </span>
            <span>{t.label}</span>
          </button>
        );
      })}
    </div>
  );
}
