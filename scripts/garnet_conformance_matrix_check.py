#!/usr/bin/env python3
"""Verify that every Conformance Matrix row's evidence column references files
that actually exist in the repository.

This is intentionally narrow. It does NOT verify that the implementation matches
the spec — that would be the whole project. It verifies the weaker, cheaper
invariant that no row points at a deleted, moved, or renamed file. Drift between
the matrix and the file system is one of the most common ways the conformance
status loses calibration.

Output: deterministic Markdown to stdout. Optionally JSON via --format json.
Exit code: 0 if every backticked path in the evidence column exists; 1 if any
row has an unresolved path (printed with row label so it's easy to fix).
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import asdict, dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_MATRIX = (
    ROOT / "C_Language_Specification" / "GARNET_v0_4_2_Conformance_Matrix.md"
)

# Matches backticked tokens in the evidence column.
# We resolve only tokens that clearly look like file paths: at least one `/`
# segment AND a file extension. This filters out grammar references like
# `do...end` and enum forms like `KwTry/KwRescue/...` that share punctuation
# with paths but never resolve on disk.
_BACKTICK = re.compile(r"`([^`]+)`")
_PATH_LIKE = re.compile(r"^[A-Za-z0-9_./-]+$")
_KNOWN_EXTENSIONS = (
    ".rs",
    ".md",
    ".toml",
    ".yml",
    ".yaml",
    ".json",
    ".py",
    ".sh",
    ".html",
    ".css",
    ".js",
    ".ts",
    ".garnet",
    ".lock",
)


@dataclass(frozen=True)
class MatrixRow:
    section: str
    status_symbol: str
    evidence_paths: list[str]


@dataclass(frozen=True)
class CheckFinding:
    section: str
    missing_path: str


@dataclass(frozen=True)
class CheckResult:
    matrix_path: str
    total_rows: int
    rows_with_paths: int
    findings: list[CheckFinding]


def _is_table_row(line: str) -> bool:
    s = line.lstrip()
    return s.startswith("|") and not s.startswith("|---")


def _looks_like_path(token: str) -> bool:
    if not _PATH_LIKE.match(token):
        return False
    if ".." in token:
        # Sequences like `do...end` look path-shaped but are prose. Real paths
        # don't contain repeated dots.
        return False
    # Require a recognized file extension to avoid false positives on enum
    # references such as `KwTry/KwRescue/KwEnsure/KwRaise`.
    return any(token.endswith(ext) for ext in _KNOWN_EXTENSIONS)


def _parse_matrix(text: str) -> list[MatrixRow]:
    rows: list[MatrixRow] = []
    in_status_legend = False
    seen_header = False
    for raw in text.splitlines():
        line = raw.rstrip()
        if line.startswith("## Status legend"):
            in_status_legend = True
            continue
        if line.startswith("## ") and in_status_legend:
            in_status_legend = False
        if not _is_table_row(line):
            continue
        if in_status_legend:
            continue
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        if len(cells) < 3:
            continue
        # Skip the table header row (column titles) once.
        if not seen_header and cells[0].lower().startswith("mini-spec"):
            seen_header = True
            continue
        if not seen_header:
            # Pre-header table rows (status legend tables already filtered above)
            # are skipped to be safe.
            continue
        section = cells[0]
        status = cells[1]
        evidence = cells[2] if len(cells) >= 3 else ""
        paths = [
            tok
            for tok in _BACKTICK.findall(evidence)
            if _looks_like_path(tok)
        ]
        rows.append(MatrixRow(section=section, status_symbol=status, evidence_paths=paths))
    return rows


def check_matrix(matrix_path: Path, root: Path | None = None) -> CheckResult:
    """Run the file-existence check against `matrix_path`.

    `root` defaults to the workspace root; tests override it to point at a
    temporary scratch directory.
    """
    resolve_root = root if root is not None else ROOT
    text = matrix_path.read_text(encoding="utf-8")
    rows = _parse_matrix(text)
    findings: list[CheckFinding] = []
    rows_with_paths = 0
    for row in rows:
        if not row.evidence_paths:
            continue
        rows_with_paths += 1
        for p in row.evidence_paths:
            candidate = resolve_root / p
            if not candidate.exists():
                findings.append(CheckFinding(section=row.section, missing_path=p))
    # Display the matrix path as relative-to-root when possible; otherwise raw.
    try:
        display_path = str(matrix_path.relative_to(resolve_root))
    except ValueError:
        display_path = str(matrix_path)
    return CheckResult(
        matrix_path=display_path,
        total_rows=len(rows),
        rows_with_paths=rows_with_paths,
        findings=findings,
    )


def render_markdown(result: CheckResult) -> str:
    lines = [
        "# Garnet Conformance Matrix File-Existence Check",
        "",
        f"Matrix: `{result.matrix_path}`",
        "",
        f"Rows parsed: {result.total_rows}",
        f"Rows with path-like evidence: {result.rows_with_paths}",
        f"Unresolved paths: {len(result.findings)}",
        "",
    ]
    if result.findings:
        lines.append("## Unresolved evidence paths")
        lines.append("")
        lines.append("| Mini-Spec section | Missing path |")
        lines.append("|---|---|")
        for f in result.findings:
            lines.append(f"| {f.section} | `{f.missing_path}` |")
    else:
        lines.append("All path-like evidence resolves on disk.")
    lines.append("")
    return "\n".join(lines)


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--matrix",
        type=Path,
        default=DEFAULT_MATRIX,
        help=f"Path to the conformance matrix Markdown (default: {DEFAULT_MATRIX.relative_to(ROOT)})",
    )
    parser.add_argument(
        "--format",
        choices=("markdown", "json"),
        default="markdown",
        help="output format",
    )
    parser.add_argument(
        "--strict",
        action="store_true",
        help=(
            "exit 1 if any path-like evidence does not resolve. Default is "
            "advisory mode (exit 0 with findings reported) so this gate can "
            "land before the matrix's existing shorthand is repaired. A "
            "future slice flips CI to --strict once the matrix is clean."
        ),
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    result = check_matrix(args.matrix)
    if args.format == "json":
        print(json.dumps(asdict(result), indent=2))
    else:
        print(render_markdown(result), end="")
    if args.strict:
        return 0 if not result.findings else 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
