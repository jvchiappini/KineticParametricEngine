/**
 * Right-click context menu for constraints and entities.
 *
 * Closes on click outside or pressing Escape.
 *
 * @packageDocumentation
 */

import { useEffect, useRef, type JSX } from "react";

interface ConstraintMenuProps {
  x: number;
  y: number;
  constraintId?: string;
  entityId?: string;
  onDeleteConstraint: (id: string) => void;
  onDeleteEntity: (id: string) => void;
  onClose: () => void;
}

export function ConstraintMenu(props: ConstraintMenuProps): JSX.Element {
  const { x, y, constraintId, entityId, onDeleteConstraint, onDeleteEntity, onClose } = props;
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        onClose();
      }
    };

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        onClose();
      }
    };

    document.addEventListener("mousedown", handleClickOutside);
    document.addEventListener("keydown", handleKeyDown);

    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, [onClose]);

  const menuStyle: React.CSSProperties = {
    position: "fixed",
    left: x,
    top: y,
    zIndex: 2000,
    background: "#1a1a2e",
    border: "1px solid #444",
    borderRadius: "6px",
    boxShadow: "0 4px 16px rgba(0,0,0,0.6)",
    padding: "4px 0",
    minWidth: "160px",
    fontFamily: "inherit",
    fontSize: "13px",
    color: "#ccc",
  };

  const itemStyle: React.CSSProperties = {
    display: "block",
    width: "100%",
    padding: "6px 16px",
    border: "none",
    background: "none",
    color: "inherit",
    textAlign: "left",
    cursor: "pointer",
    fontFamily: "inherit",
    fontSize: "inherit",
  };

  return (
    <div ref={menuRef} style={menuStyle}>
      {constraintId && (
        <button
          style={{ ...itemStyle, color: "#f55" }}
          onClick={() => {
            onDeleteConstraint(constraintId);
            onClose();
          }}
        >
          Delete constraint
        </button>
      )}

      {entityId && (
        <button
          style={{ ...itemStyle, color: "#f55" }}
          onClick={() => {
            onDeleteEntity(entityId);
            onClose();
          }}
        >
          Delete entity
        </button>
      )}

      {!constraintId && !entityId && (
        <div style={{ ...itemStyle, cursor: "default", color: "#666" }}>
          No action available
        </div>
      )}

      <hr style={{ border: "none", borderTop: "1px solid #333", margin: "4px 0" }} />

      <button style={itemStyle} onClick={onClose}>
        Cancel
      </button>
    </div>
  );
}
