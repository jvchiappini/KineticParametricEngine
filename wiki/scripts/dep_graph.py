#!/usr/bin/env python3
"""
dep_graph.py — Genera el grafo de dependencias entre crates de KPE.

Lee los Cargo.toml de cada crate y produce:
  - Un archivo .dot (Graphviz) en wiki/diagrams/deps.dot
  - Una representación ASCII del grafo en stdout

Uso:
    python wiki/scripts/dep_graph.py
    python wiki/scripts/dep_graph.py --format dot   # solo el .dot
    python wiki/scripts/dep_graph.py --format ascii  # solo ASCII
    dot -Tsvg wiki/diagrams/deps.dot -o wiki/diagrams/deps.svg
"""

import argparse
import re
import sys
from pathlib import Path


REPO_ROOT = Path(__file__).parent.parent.parent
DIAGRAMS_DIR = REPO_ROOT / "wiki" / "diagrams"

TOML_DEP = re.compile(r'^\s*(kpe-\w+)\s*=', re.MULTILINE)
CRATE_NAME = re.compile(r'^name\s*=\s*"(kpe-\w+)"', re.MULTILINE)


def find_crates() -> dict[str, Path]:
    """Encuentra todos los crates de KPE en el workspace."""
    crates = {}
    for cargo_toml in (REPO_ROOT / "crates").rglob("Cargo.toml"):
        content = cargo_toml.read_text()
        m = CRATE_NAME.search(content)
        if m:
            crates[m.group(1)] = cargo_toml
    return crates


def build_graph(crates: dict[str, Path]) -> dict[str, list[str]]:
    """Construye el grafo de dependencias entre crates internos."""
    graph: dict[str, list[str]] = {name: [] for name in crates}

    for name, toml_path in crates.items():
        content = toml_path.read_text()
        # Buscar en la sección [dependencies] solo
        deps_section = content.split("[dependencies]")[-1].split("[")[0] if "[dependencies]" in content else ""
        for dep in TOML_DEP.findall(deps_section):
            if dep in crates and dep != name:
                graph[name].append(dep)

    return graph


def to_dot(graph: dict[str, list[str]]) -> str:
    """Genera representación Graphviz DOT."""
    lines = [
        "digraph kpe_deps {",
        '  rankdir=TB;',
        '  node [shape=box, style=filled, fillcolor="#1a1a26", fontcolor="#e8e8f0", color="#6c63ff", fontname="monospace"];',
        '  edge [color="#6c63ff88"];',
        "",
    ]

    # Colorear kpe-schema diferente (es el hub)
    lines.append('  "kpe-schema" [fillcolor="#2a1a4a", color="#00e5ff", fontcolor="#00e5ff"];')
    lines.append("")

    for src, deps in sorted(graph.items()):
        for dst in deps:
            lines.append(f'  "{src}" -> "{dst}";')

    lines.append("}")
    return "\n".join(lines)


def to_ascii(graph: dict[str, list[str]]) -> str:
    """Genera representación ASCII del grafo."""
    lines = ["Grafo de dependencias entre crates de KPE", "=" * 50, ""]

    for crate, deps in sorted(graph.items()):
        lines.append(f"  📦 {crate}")
        if deps:
            for dep in sorted(deps):
                lines.append(f"     └─ {dep}")
        else:
            lines.append("     (sin dependencias internas)")
        lines.append("")

    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser(
        description="Genera el grafo de dependencias entre crates de KPE."
    )
    parser.add_argument(
        "--format",
        choices=["dot", "ascii", "both"],
        default="both",
    )
    args = parser.parse_args()

    crates = find_crates()
    if not crates:
        print("No se encontraron crates de KPE. ¿Estás en la raíz del repo?", file=sys.stderr)
        sys.exit(1)

    graph = build_graph(crates)

    if args.format in ("dot", "both"):
        dot_content = to_dot(graph)
        DIAGRAMS_DIR.mkdir(parents=True, exist_ok=True)
        out = DIAGRAMS_DIR / "deps.dot"
        out.write_text(dot_content)
        print(f"✅ Generado: {out.relative_to(REPO_ROOT)}")
        print("   Para convertir a SVG: dot -Tsvg wiki/diagrams/deps.dot -o wiki/diagrams/deps.svg")

    if args.format in ("ascii", "both"):
        print("\n" + to_ascii(graph))


if __name__ == "__main__":
    main()
