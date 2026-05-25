#!/usr/bin/env python3
"""
find_type.py — Encuentra dónde está definido un tipo en el codebase de KPE.

Busca definiciones de structs, enums, traits, type aliases y funciones pub
en archivos .rs y .ts/.tsx.

Uso:
    python wiki/scripts/find_type.py KPERecipe
    python wiki/scripts/find_type.py Joint --crate kpe-schema
    python wiki/scripts/find_type.py solve --kind fn
"""

import argparse
import re
import sys
from dataclasses import dataclass
from pathlib import Path


REPO_ROOT = Path(__file__).parent.parent.parent

# Patrones de definición en Rust
RUST_PATTERNS = {
    "struct":  re.compile(r"^pub(?:\([^)]+\))?\s+struct\s+(\w+)"),
    "enum":    re.compile(r"^pub(?:\([^)]+\))?\s+enum\s+(\w+)"),
    "trait":   re.compile(r"^pub(?:\([^)]+\))?\s+trait\s+(\w+)"),
    "type":    re.compile(r"^pub(?:\([^)]+\))?\s+type\s+(\w+)"),
    "fn":      re.compile(r"^pub(?:\([^)]+\))?\s+fn\s+(\w+)"),
    "const":   re.compile(r"^pub(?:\([^)]+\))?\s+const\s+(\w+)"),
    "mod":     re.compile(r"^pub(?:\([^)]+\))?\s+mod\s+(\w+)"),
}

# Patrones de definición en TypeScript
TS_PATTERNS = {
    "interface": re.compile(r"^export\s+interface\s+(\w+)"),
    "type":      re.compile(r"^export\s+type\s+(\w+)"),
    "class":     re.compile(r"^export\s+(?:default\s+)?class\s+(\w+)"),
    "fn":        re.compile(r"^export\s+(?:async\s+)?function\s+(\w+)"),
    "const":     re.compile(r"^export\s+const\s+(\w+)"),
    "enum":      re.compile(r"^export\s+(?:const\s+)?enum\s+(\w+)"),
}


@dataclass
class Match:
    file: Path
    line_number: int
    line: str
    kind: str
    name: str


def search_file(filepath: Path, query: str, kind_filter: str | None) -> list[Match]:
    """Busca definiciones en un archivo que coincidan con el query."""
    matches = []

    is_rust = filepath.suffix == ".rs"
    patterns = RUST_PATTERNS if is_rust else TS_PATTERNS

    if kind_filter:
        patterns = {k: v for k, v in patterns.items() if k == kind_filter}

    try:
        lines = filepath.read_text(encoding="utf-8").splitlines()
    except (UnicodeDecodeError, PermissionError):
        return []

    query_lower = query.lower()

    for i, line in enumerate(lines, start=1):
        stripped = line.strip()
        for kind, pattern in patterns.items():
            m = pattern.match(stripped)
            if m:
                name = m.group(1)
                if query_lower in name.lower():
                    matches.append(Match(
                        file=filepath,
                        line_number=i,
                        line=stripped,
                        kind=kind,
                        name=name,
                    ))

    return matches


def format_results(matches: list[Match], query: str) -> str:
    """Formatea los resultados para display en terminal."""
    if not matches:
        return f"No se encontró '{query}' en el codebase.\n"

    lines = [f"Se encontraron {len(matches)} definición(es) para '{query}':\n"]

    # Agrupar por crate
    by_crate: dict[str, list[Match]] = {}
    for m in matches:
        parts = m.file.parts
        crate = "unknown"
        for i, part in enumerate(parts):
            if part in {"crates", "apps"}:
                crate = parts[i + 1] if i + 1 < len(parts) else "unknown"
                break
        by_crate.setdefault(crate, []).append(m)

    for crate, crate_matches in sorted(by_crate.items()):
        lines.append(f"  📦 {crate}")
        for m in crate_matches:
            rel = m.file.relative_to(REPO_ROOT)
            lines.append(f"     {m.kind:12} {m.name}")
            lines.append(f"     {'':12} {rel}:{m.line_number}")
            lines.append(f"     {'':12} {m.line[:80]}")
            lines.append("")

    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser(
        description="Encuentra definiciones de tipos en el codebase de KPE."
    )
    parser.add_argument("query", help="Nombre (o parte del nombre) a buscar")
    parser.add_argument(
        "--crate",
        help="Limitar búsqueda a un crate específico (ej: kpe-schema)",
    )
    parser.add_argument(
        "--kind",
        choices=["struct", "enum", "trait", "type", "fn", "const", "mod",
                 "interface", "class"],
        help="Filtrar por tipo de definición",
    )
    parser.add_argument(
        "--no-ts",
        action="store_true",
        help="Ignorar archivos TypeScript",
    )
    args = parser.parse_args()

    # Determinar raíz de búsqueda
    if args.crate:
        search_root = REPO_ROOT / "crates" / args.crate
        if not search_root.exists():
            search_root = REPO_ROOT / "apps" / args.crate
        if not search_root.exists():
            print(f"Crate '{args.crate}' no encontrado.", file=sys.stderr)
            sys.exit(1)
    else:
        search_root = REPO_ROOT

    # Recopilar archivos
    extensions = [".rs"] if args.no_ts else [".rs", ".ts", ".tsx"]
    files = []
    for ext in extensions:
        for f in search_root.rglob(f"*{ext}"):
            parts = f.parts
            if not any(p in parts for p in {"target", "node_modules", ".git"}):
                files.append(f)

    # Buscar
    all_matches = []
    for filepath in files:
        all_matches.extend(search_file(filepath, args.query, args.kind))

    print(format_results(all_matches, args.query))
    sys.exit(0 if all_matches else 1)


if __name__ == "__main__":
    main()
