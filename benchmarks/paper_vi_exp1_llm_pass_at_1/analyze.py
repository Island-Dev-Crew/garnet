#!/usr/bin/env python3
"""Render a cautious Paper VI Exp 1 analysis summary."""
from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any


def render(data: dict[str, Any]) -> str:
    lines = [
        "# Paper VI Exp 1 Analysis",
        "",
        f"Status: {data.get('status', 'unknown')}",
        "",
        "No provider-backed pass@1 measurement is claimed by this S94 analysis.",
        "Measured rows are valid only for deterministic fixture plumbing unless a",
        "future reviewed run records a real provider and hidden-test scorer.",
        "",
        "| Language | Rows | Measured | Pass rows | pass@1 |",
        "|---|---:|---:|---:|---:|",
    ]
    for language, info in sorted(data.get("by_language", {}).items()):
        pass_at_1 = info.get("pass_at_1")
        pass_at_1_text = "pending" if pass_at_1 is None else f"{pass_at_1:.3f}"
        lines.append(
            f"| {language} | {info.get('rows', 0)} | {info.get('measured_rows', 0)} | "
            f"{info.get('pass_rows', 0)} | {pass_at_1_text} |"
        )
    lines.extend(
        [
            "",
            "Boundary: seed-only corpus; provider credentials and the full registered",
            "benchmark remain pending infrastructure.",
            "",
        ]
    )
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("aggregate_json", type=Path)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args(argv)
    data = json.loads(args.aggregate_json.read_text(encoding="utf-8"))
    rendered = render(data)
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(rendered, encoding="utf-8")
    print(rendered, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
