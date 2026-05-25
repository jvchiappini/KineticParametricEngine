#!/usr/bin/env python3
"""
find_usages.py — Encuentra todos los usos de una función, tipo o constante.

A diferencia de find_type.py (que busca definiciones), este script busca
los lugares donde algo se USA: llamadas a funciones, instanciaciones de
structs, implementaciones de traits, etc.

Uso:
    python wiki/scripts/find_usages.py resolve_expression
    python wiki/scripts/find_usages.py KPERecipe --crate kpe-parametric
    python wiki/scripts/find_usages.py NestingConfig --context 2
"""

import argparse
import sys
from pathlib import Path


REPO_ROOT = Path(__file__).parent.parent.parent
SKIP_DIRS = {"target", "node_modules", ".git", "dist"}


def search_usages(
    query: str,
    root: Path,
    extensions: list[str],
    context_lines: int,
) -> list[dict]:
    """
    Busca todas las ocurrencias de query en el codebase.

    Excluye líneas que son definiciones (pub struct, pub fn, etc.)
    para mostrar solo los usos reales.
    """
    results = []
    definition_prefixes = ("pub struct", "pub enum", "pub fn", "pub trait", "pub type",
                           "pub const", "export interface", "export type", "export function",
                           "export class", "///", "//")

    for ext in extensions:
        for filepath in root.rglob(f"*{ext}"):
            if any(p in filepath.parts for p in SKIP_DIRS):
                continue

            try:
                lines = filepath.read_text(encoding="utf-8").splitlines()
            except (UnicodeDecodeError, PermissionError):
                continue

            for i, line in enumerate(lines):
                if query not in line:
                    continue

                stripped = line.strip()

                # Saltar definiciones — queremos usos
                is_definition = any(stripped.startswith(p) for p in definition_prefixes)
                if is_definition and f" {query}" in line and ("struct" in line or "fn" in line or "enum" in line):
                    continue

                # Capturar contexto
                start = max(0, i - context_lines)
                end = min(len(lines), i + context_lines + 1)
                ctx = lines[start:end]

                results.append({
                    "file": filepath,
                    "line": i + 1,
                    "content": line.rstrip(),
                    "context": [(start + j + 1, l) for j, l in enumerate(ctx)],
                })

    return results


def format_results(results: list[dict], query: str, context_lines: int) -> str:
    """Formatea los resultados para display."""
    if not results:
        return f"No se encontraron usos de '{query}'.\n"

    out = [f"Se encontraron {len(results)} uso(s) de '{query}':\n"]

    for r in results:
        rel = r["file"].relative_to(REPO_ROOT)
        out.append(f"  📄 {rel}:{r['line']}")

        if context_lines > 0:
            for lineno, content in r["context"]:
                marker = "→ " if lineno == r["line"] else "  "
                out.append(f"  {marker}{lineno:4d} │ {content}")
        else:
            out.append(f"       {r['content'].strip()}")

        out.append("")

    return "\n".join(out)


def main():
    parser = argparse.ArgumentParser(
        description="Encuentra todos los usos de una función o tipo en KPE."
    )
    parser.add_argument("query", help="Nombre a buscar")
    parser.add_argument(
        "--crate",
        help="Limitar búsqueda a un crate específico",
    )
    parser.add_argument(
        "--context",
        type=int,
        default=1,
        help="Líneas de contexto alrededor de cada uso (default: 1)",
    )
    parser.add_argument(
        "--no-ts",
        action="store_true",
        help="Ignorar archivos TypeScript",
    )
    args = parser.parse_args()

    if args.crate:
        root = REPO_ROOT / "crates" / args.crate
        if not root.exists():
            root = REPO_ROOT / "apps" / args.crate
        if not root.exists():
            print(f"Crate '{args.crate}' no encontrado.", file=sys.stderr)
            sys.exit(1)
    else:
        root = REPO_ROOT

    extensions = [".rs"] if args.no_ts else [".rs", ".ts", ".tsx"]
    results = search_usages(args.query, root, extensions, args.context)

    print(format_results(results, args.query, args.context))
    sys.exit(0 if results else 1)


if __name__ == "__main__":
    main()
