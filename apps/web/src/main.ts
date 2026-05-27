import { createElement } from "react";
import { createRoot } from "react-dom/client";
import { SketchEditor } from "./sketch";

const root = document.getElementById("root");
if (root) {
  createRoot(root).render(createElement(SketchEditor));
}
