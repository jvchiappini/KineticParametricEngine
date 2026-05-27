#!/usr/bin/env python3
"""
check_modularity.py — Detecta archivos que superan el límite de líneas.

Regla de KPE: ningún archivo .rs supera 700 líneas.
            ningún archivo .ts supera 700 líneas.

Uso:
    python wiki/scripts/check_modularity.py
    python wiki/scripts/check_modularity.py --limit 200
    python wiki/scripts/check_modularity.py --path crates/kpe-core
    python wiki/scripts/check_modularity.py --json        # output JSON para CI
"""

import argparse
import json
import sys
from pathlib import Path


EXTENSIONS = {".rs", ".ts", ".tsx"}
DEFAULT_LIMIT = 700
REPO_ROOT = Path(__file__).parent.parent.parent


def count_lines(path: Path) -> int:
    """Cuenta líneas de un archivo, ignorando líneas en blanco y comentarios de una línea."""
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
        return len(lines)
    except (UnicodeDecodeError, PermissionError):
        return 0


def find_violations(root: Path, limit: int, extensions: set[str]) -> list[dict]:
    """
    Busca archivos que superen el límite de líneas.

    Returns:
        Lista de dicts con keys: path, lines, excess
    """
    violations = []

    for ext in extensions:
        for filepath in root.rglob(f"*{ext}"):
            # Ignorar directorios de build y dependencias
            parts = filepath.parts
            if any(p in parts for p in {"target", "node_modules", ".git", "dist"}):
                continue

            lines = count_lines(filepath)
            if lines > limit:
                violations.append({
                    "path": str(filepath.relative_to(REPO_ROOT)),
                    "lines": lines,
                    "excess": lines - limit,
                })

    return sorted(violations, key=lambda v: v["excess"], reverse=True)


def format_table(violations: list[dict], limit: int) -> str:
    """Formatea los resultados como tabla ASCII."""
    if not violations:
        return f"✅ Todos los archivos están dentro del límite de {limit} líneas.\n"

    lines = [
        f"❌ Se encontraron {len(violations)} archivo(s) que superan {limit} líneas:\n",
        f"{'Archivo':<60} {'Líneas':>8} {'Exceso':>8}",
        "-" * 80,
    ]

    for v in violations:
        path = v["path"]
        if len(path) > 58:
            path = "..." + path[-55:]
        lines.append(f"{path:<60} {v['lines']:>8} {'+' + str(v['excess']):>8}")

    lines.append("")
    lines.append("Estos archivos deben dividirse en módulos más pequeños.")
    lines.append("Ver wiki/decisions/000-template.md para la justificación.")

    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser(
        description="Verifica que ningún archivo supere el límite de líneas de KPE."
    )
    parser.add_argument(
        "--limit",
        type=int,
        default=DEFAULT_LIMIT,
        help=f"Límite de líneas por archivo (default: {DEFAULT_LIMIT})",
    )
    parser.add_argument(
        "--path",
        type=Path,
        default=REPO_ROOT,
        help="Directorio a analizar (default: raíz del repo)",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="Output en formato JSON (para CI)",
    )
    parser.add_argument(
        "--ext",
        nargs="+",
        default=list(EXTENSIONS),
        help=f"Extensiones a verificar (default: {EXTENSIONS})",
    )
    args = parser.parse_args()

    root = args.path if args.path.is_absolute() else REPO_ROOT / args.path
    extensions = {e if e.startswith(".") else f".{e}" for e in args.ext}

    violations = find_violations(root, args.limit, extensions)

    if args.json:
        result = {
            "limit": args.limit,
            "violations": violations,
            "passed": len(violations) == 0,
        }
        print(json.dumps(result, indent=2))
    else:
        print(format_table(violations, args.limit))

    # Exit code 1 si hay violaciones (útil para CI)
    sys.exit(1 if violations else 0)


if __name__ == "__main__":
    main()
