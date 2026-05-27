import type { JSX } from "react";

type SketchPlane = "XY" | "XZ" | "YZ";

interface SketchPropertiesPanelProps {
  plane: SketchPlane;
  planeOffset: number;
  gridVisible: boolean;
  constructionMode: boolean;
  entityCount: number;
  constraintCount: number;
  dof: number;
  depth: number;
  bevel: boolean;
  bevelSize: number;
  onPlaneChange: (plane: SketchPlane) => void;
  onPlaneOffsetChange: (offset: number) => void;
  onGridToggle: () => void;
  onConstructionModeToggle: () => void;
  onDepthChange: (depth: number) => void;
  onBevelToggle: (v: boolean) => void;
  onBevelSizeChange: (v: number) => void;
}

const s: import("react").CSSProperties = {
  marginBottom: "12px",
};

const l: import("react").CSSProperties = {
  fontSize: "10px", fontWeight: 700, textTransform: "uppercase",
  color: "#666", letterSpacing: "0.5px", marginBottom: "6px",
};

const sel: import("react").CSSProperties = {
  width: "100%", background: "#0a0a18", border: "1px solid #336",
  color: "#eee", padding: "5px 8px", borderRadius: "4px", fontSize: "12px",
  fontFamily: "inherit", outline: "none", cursor: "pointer",
};

const inp: import("react").CSSProperties = {
  width: "100%", background: "#0a0a18", border: "1px solid #336",
  color: "#eee", padding: "5px 8px", borderRadius: "4px",
  fontSize: "12px", fontFamily: "inherit", outline: "none",
};

const btn: import("react").CSSProperties = {
  width: "100%", padding: "5px 8px", border: "1px solid #444",
  borderRadius: "4px", background: "#2a2a3e", color: "#aaa",
  cursor: "pointer", fontSize: "11px", fontFamily: "inherit", textAlign: "left",
};

const sr: import("react").CSSProperties = {
  display: "flex", justifyContent: "space-between", alignItems: "center",
  padding: "2px 0", fontSize: "11px", color: "#999",
};

export function SketchPropertiesPanel(props: SketchPropertiesPanelProps): JSX.Element {
  const {
    plane, planeOffset, gridVisible, constructionMode,
    entityCount, constraintCount, dof, depth, bevel, bevelSize,
    onPlaneChange, onPlaneOffsetChange, onGridToggle, onConstructionModeToggle,
    onDepthChange, onBevelToggle, onBevelSizeChange,
  } = props;

  return (
    <div style={{
      width: "200px", background: "#16162a", borderRight: "1px solid #333",
      padding: "10px", display: "flex", flexDirection: "column", gap: "4px",
      fontSize: "12px", color: "#ccc", fontFamily: "inherit",
      overflowY: "auto", flexShrink: 0,
    }}>
      <div style={s}>
        <div style={l}>Work Plane</div>
        <select value={plane} onChange={(e) => onPlaneChange(e.target.value as SketchPlane)} style={sel}>
          <option value="XY">XY — Front</option>
          <option value="XZ">XZ — Top</option>
          <option value="YZ">YZ — Right</option>
        </select>
      </div>

      <div style={s}>
        <div style={l}>Plane Offset</div>
        <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
          <input type="number" value={planeOffset}
            onChange={(e) => onPlaneOffsetChange(parseFloat(e.target.value) || 0)} step={0.5}
            style={{ ...inp, width: "80px" }} />
          <span style={{ color: "#888", fontSize: "11px" }}>mm</span>
        </div>
      </div>

      <div style={s}>
        <div style={l}>Depth</div>
        <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
          <input type="range" min={0} max={100} step={0.5} value={depth}
            onChange={(e) => onDepthChange(parseFloat(e.target.value))}
            style={{ flex: 1, accentColor: "#4488cc" }} />
          <input type="number" value={depth} min={0} max={500} step={0.5}
            onChange={(e) => onDepthChange(Math.max(0, parseFloat(e.target.value) || 0))}
            style={{ ...inp, width: "55px", textAlign: "right" }} />
        </div>
      </div>

      <div style={s}>
        <div style={l}>Bevel</div>
        <button onClick={() => onBevelToggle(!bevel)} style={{
          ...btn, color: bevel ? "#4a9eff" : "#666",
          borderColor: bevel ? "#4a9eff" : "#444",
        }}>
          {bevel ? "● Bevel: On" : "○ Bevel: Off"}
        </button>
        {bevel && (
          <div style={{ display: "flex", alignItems: "center", gap: "6px", marginTop: "4px" }}>
            <input type="range" min={0.1} max={3} step={0.1} value={bevelSize}
              onChange={(e) => onBevelSizeChange(parseFloat(e.target.value))}
              style={{ flex: 1, accentColor: "#4488cc" }} />
            <span style={{ color: "#ccc", fontSize: "11px", fontFamily: "monospace", minWidth: "28px", textAlign: "right" }}>
              {bevelSize.toFixed(1)}
            </span>
          </div>
        )}
      </div>

      <div style={s}>
        <div style={l}>Display</div>
        <button onClick={onGridToggle} style={{
          ...btn, color: gridVisible ? "#4a9eff" : "#666",
          borderColor: gridVisible ? "#4a9eff" : "#444",
        }}>
          {gridVisible ? "● Grid: On" : "○ Grid: Off"}
        </button>
      </div>

      <div style={s}>
        <div style={l}>Mode</div>
        <button onClick={onConstructionModeToggle} style={{
          ...btn, color: constructionMode ? "#00bcd4" : "#aaa",
          borderColor: constructionMode ? "#00bcd4" : "#444",
        }}>
          {constructionMode ? "● Construction: On" : "○ Construction: Off"}
        </button>
      </div>

      <div style={{ ...s, marginTop: "8px", borderTop: "1px solid #333", paddingTop: "8px" }}>
        <div style={l}>Stats</div>
        <div style={sr}><span>Entities</span><span style={{ color: "#ddd", fontWeight: 600 }}>{entityCount}</span></div>
        <div style={sr}><span>Constraints</span><span style={{ color: "#ddd", fontWeight: 600 }}>{constraintCount}</span></div>
        <div style={sr}>
          <span>DOF</span>
          <span style={{ color: dof === 0 ? "#43e97b" : dof < 5 ? "#ffa726" : "#ff6584", fontWeight: 700 }}>
            {dof}{dof === 0 && " (Fully constrained)"}
          </span>
        </div>
      </div>

      <div style={{ ...s, borderTop: "1px solid #333", paddingTop: "8px" }}>
        <div style={l}>Shortcuts</div>
        <div style={{ fontSize: "10px", color: "#666", lineHeight: "1.8" }}>
          <div>S — Select &nbsp; L — Line</div>
          <div>R — Rectangle &nbsp; C — Circle</div>
          <div>A — Arc &nbsp; P — Polyline</div>
          <div>D — Dimension &nbsp; F — Fit</div>
          <div>G — Grid &nbsp; Esc — Cancel</div>
          <div>Ctrl+Z — Undo &nbsp; Ctrl+Shift+Z — Redo</div>
        </div>
      </div>
    </div>
  );
}
