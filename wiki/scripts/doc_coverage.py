#!/usr/bin/env python3
"""
doc_coverage.py — Verifica que todos los items públicos de Rust tengan docstring.

En KPE, cada pub struct, enum, trait, fn y const DEBE tener un docstring (///).
Este script detecta violaciones y reporta el porcentaje de cobertura.

Uso:
    python wiki/scripts/doc_coverage.py
    python wiki/scripts/doc_coverage.py --crate kpe-schema
    python wiki/scripts/doc_coverage.py --fail-under 90   # falla si < 90%
    python wiki/scripts/doc_coverage.py --json
"""

import argparse
import json
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path


REPO_ROOT = Path(__file__).parent.parent.parent

PUB_ITEM = re.compile(
    r"^pub(?:\([^)]+\))?\s+(struct|enum|trait|fn|type|const)\s+(\w+)"
)
DOC_COMMENT = re.compile(r"^\s*///")
SKIP_DIRS = {"target", "node_modules", ".git", "dist"}


@dataclass
class Item:
    file: Path
    line: int
    kind: str
    name: str
    has_doc: bool


@dataclass
class FileReport:
    path: Path
    items: list[Item] = field(default_factory=list)

    @property
    def total(self) -> int:
        return len(self.items)

    @property
    def documented(self) -> int:
        return sum(1 for i in self.items if i.has_doc)

    @property
    def coverage(self) -> float:
        return (self.documented / self.total * 100) if self.total > 0 else 100.0

    @property
    def violations(self) -> list[Item]:
        return [i for i in self.items if not i.has_doc]


def analyze_file(filepath: Path) -> FileReport:
    """Analiza un archivo Rust y detecta items pub sin docstring."""
    report = FileReport(path=filepath)

    try:
        lines = filepath.read_text(encoding="utf-8").splitlines()
    except (UnicodeDecodeError, PermissionError):
        return report

    for i, line in enumerate(lines):
        stripped = line.strip()
        m = PUB_ITEM.match(stripped)
        if not m:
            continue

        kind, name = m.group(1), m.group(2)

        # Buscar doc comment en las líneas anteriores (ignora líneas en blanco y attrs)
        has_doc = False
        j = i - 1
        while j >= 0:
            prev = lines[j].strip()
            if DOC_COMMENT.match(prev):
                has_doc = True
                break
            elif prev.startswith("#[") or prev == "" or prev.startswith("//"):
                j -= 1
                continue
            else:
                break

        report.items.append(Item(
            file=filepath,
            line=i + 1,
            kind=kind,
            name=name,
            has_doc=has_doc,
        ))

    return report


def collect_reports(root: Path) -> list[FileReport]:
    """Recopila reportes de todos los archivos Rust en el directorio."""
    reports = []
    for filepath in root.rglob("*.rs"):
        if any(p in filepath.parts for p in SKIP_DIRS):
            continue
        report = analyze_file(filepath)
        if report.total > 0:
            reports.append(report)
    return reports


def print_report(reports: list[FileReport], fail_under: float) -> bool:
    """Imprime el reporte y retorna True si pasa el umbral."""
    total_items = sum(r.total for r in reports)
    total_documented = sum(r.documented for r in reports)
    global_coverage = (total_documented / total_items * 100) if total_items > 0 else 100.0

    # Mostrar violaciones
    violations_found = False
    for report in sorted(reports, key=lambda r: r.coverage):
        if report.violations:
            violations_found = True
            rel = report.path.relative_to(REPO_ROOT)
            print(f"\n  📄 {rel}  ({report.coverage:.0f}% documentado)")
            for item in report.violations:
                print(f"     línea {item.line:4d}  {item.kind:8}  {item.name}  ← falta ///")

    # Resumen
    print(f"\n{'─' * 60}")
    print(f"  Total items pub:    {total_items}")
    print(f"  Documentados:       {total_documented}")
    print(f"  Sin docstring:      {total_items - total_documented}")
    print(f"  Cobertura global:   {global_coverage:.1f}%")

    passed = global_coverage >= fail_under
    status = "✅ PASA" if passed else f"❌ FALLA (mínimo requerido: {fail_under:.0f}%)"
    print(f"  Estado:             {status}")

    if not violations_found:
        print("\n  ✅ Todos los items públicos tienen docstring.")

    return passed


def main():
    parser = argparse.ArgumentParser(
        description="Verifica cobertura de docstrings en items públicos de Rust."
    )
    parser.add_argument(
        "--crate",
        help="Limitar a un crate específico (ej: kpe-schema)",
    )
    parser.add_argument(
        "--fail-under",
        type=float,
        default=100.0,
        help="Porcentaje mínimo requerido (default: 100)",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="Output en formato JSON",
    )
    args = parser.parse_args()

    if args.crate:
        root = REPO_ROOT / "crates" / args.crate
        if not root.exists():
            print(f"Crate '{args.crate}' no encontrado.", file=sys.stderr)
            sys.exit(1)
    else:
        root = REPO_ROOT / "crates"

    reports = collect_reports(root)

    if args.json:
        data = {
            "coverage": sum(r.documented for r in reports) / max(sum(r.total for r in reports), 1) * 100,
            "files": [
                {
                    "path": str(r.path.relative_to(REPO_ROOT)),
                    "coverage": r.coverage,
                    "violations": [
                        {"line": i.line, "kind": i.kind, "name": i.name}
                        for i in r.violations
                    ],
                }
                for r in reports if r.violations
            ],
        }
        print(json.dumps(data, indent=2))
        sys.exit(0 if data["coverage"] >= args.fail_under else 1)

    passed = print_report(reports, args.fail_under)
    sys.exit(0 if passed else 1)


if __name__ == "__main__":
    main()
