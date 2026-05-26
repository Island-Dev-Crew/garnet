#!/usr/bin/env python3
"""Fail if the determinism workflow ever invokes the non-deterministic LLM tier."""
from __future__ import annotations

from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_WORKFLOW = ROOT / ".github" / "workflows" / "determinism.yml"


def scan_text(text: str) -> list[tuple[int, str]]:
    violations: list[tuple[int, str]] = []
    for line_no, line in enumerate(text.splitlines(), start=1):
        if "--llm" in line:
            violations.append((line_no, line.rstrip()))
    return violations


def scan_path(path: Path = DEFAULT_WORKFLOW) -> list[tuple[int, str]]:
    if not path.exists():
        return [(0, f"missing determinism workflow: {path}")]
    return scan_text(path.read_text(encoding="utf-8"))


def main(argv: list[str] | None = None) -> int:
    args = sys.argv[1:] if argv is None else argv
    path = Path(args[0]) if args else DEFAULT_WORKFLOW
    violations = scan_path(path)
    if violations:
        for line_no, line in violations:
            if line_no == 0:
                print(f"determinism-no-llm: {line}", file=sys.stderr)
            else:
                print(
                    f"determinism-no-llm: {path}:{line_no}: forbidden --llm in determinism job: {line}",
                    file=sys.stderr,
                )
        return 1
    print(f"determinism-no-llm: ok ({path})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
