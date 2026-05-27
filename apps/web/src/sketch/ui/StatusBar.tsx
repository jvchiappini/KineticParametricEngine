/**
 * Bottom status bar showing tool hints, degrees of freedom, cursor
 * coordinates, zoom level, and the active snap type.
 *
 * @packageDocumentation
 */

import type { JSX } from "react";

interface StatusBarProps {
  hint: string;
  dof: number;
  cursorX: number;
  cursorY: number;
  zoomLevel: number;
  snapKind: string;
}

export function StatusBar(props: StatusBarProps): JSX.Element {
  const { hint, dof, cursorX, cursorY, zoomLevel, snapKind } = props;

  const barStyle: React.CSSProperties = {
    display: "flex",
    alignItems: "center",
    justifyContent: "space-between",
    height: "24px",
    padding: "0 12px",
    background: "#0d0d1a",
    borderTop: "1px solid #333",
    color: "#888",
    fontSize: "11px",
    fontFamily: "monospace",
    userSelect: "none",
    flexShrink: 0,
  };

  const sectionStyle: React.CSSProperties = {
    display: "flex",
    alignItems: "center",
    gap: "16px",
  };

  return (
    <div style={barStyle}>
      <div style={sectionStyle}>
        <span style={{ color: "#aaa", maxWidth: "400px", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
          {hint || "Ready"}
        </span>
      </div>

      <div style={sectionStyle}>
        {snapKind && snapKind !== "none" && (
          <span style={{ color: "#4a9eff" }}>
            {snapKind}
          </span>
        )}

        <span>
          ({cursorX.toFixed(2)}, {cursorY.toFixed(2)})
        </span>

        <span>
          DOF: {dof}
        </span>

        <span>
          {zoomLevel.toFixed(0)}%
        </span>
      </div>
    </div>
  );
}
