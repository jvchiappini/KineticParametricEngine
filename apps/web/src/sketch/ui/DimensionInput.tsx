/**
 * Inline dimension value input positioned absolutely over the canvas.
 *
 * Auto-focuses on mount.  Enter submits the value; Escape cancels.
 *
 * @packageDocumentation
 */

import { useEffect, useRef, type JSX } from "react";

interface DimensionInputProps {
  value: number;
  onSubmit: (value: number) => void;
  onCancel: () => void;
  position: { x: number; y: number };
  unit?: string;
}

export function DimensionInput(props: DimensionInputProps): JSX.Element {
  const { value, onSubmit, onCancel, position, unit = "mm" } = props;
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    inputRef.current?.focus();
    inputRef.current?.select();
  }, []);

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter") {
      const parsed = parseFloat(e.currentTarget.value);
      if (!isNaN(parsed)) {
        onSubmit(parsed);
      }
    } else if (e.key === "Escape") {
      onCancel();
    }
    e.stopPropagation();
  };

  return (
    <div
      style={{
        position: "fixed",
        left: position.x,
        top: position.y,
        zIndex: 1000,
        transform: "translate(-50%, -100%)",
        marginTop: "-8px",
      }}
    >
      <div style={{
        display: "flex",
        alignItems: "center",
        background: "#1a1a2e",
        border: "1px solid #4a9eff",
        borderRadius: "4px",
        padding: "2px 6px",
        boxShadow: "0 2px 8px rgba(0,0,0,0.5)",
      }}>
        <input
          ref={inputRef}
          type="text"
          defaultValue={value}
          onKeyDown={handleKeyDown}
          onBlur={() => onCancel()}
          onClick={(e) => e.stopPropagation()}
          style={{
            width: "70px",
            background: "transparent",
            border: "none",
            color: "#fff",
            fontSize: "13px",
            fontFamily: "monospace",
            outline: "none",
            padding: "2px 0",
          }}
        />
        <span style={{
          color: "#888",
          fontSize: "11px",
          marginLeft: "4px",
          fontFamily: "monospace",
        }}>
          {unit}
        </span>
      </div>
    </div>
  );
}
