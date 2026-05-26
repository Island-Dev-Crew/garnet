#!/usr/bin/env python3
"""Render a cautious Paper VI Exp 3 analysis stub from aggregate JSON."""
from __future__ import annotations

import argparse
import json
from pathlib import Path


def render(data: dict) -> str:
    lines = [
        "# Paper VI Exp 3 Analysis",
        "",
        "Status: harness-only",
        "",
        "This file does not claim h3a, h3b, or h3c. It reports whether the stateless",
        "and history-aware lanes produced records that a later reviewed study can",
        "analyze.",
        "",
        "| Lane | Records | Missing LLM log | Snapshots |",
        "|---|---:|---:|---|",
    ]
    for lane, info in sorted(data.get("lanes", {}).items()):
        snapshots = ", ".join(info.get("snapshots", [])) or "none"
        lines.append(
            f"| {lane} | {info.get('records', 0)} | {info.get('missing_llm_log', 0)} | {snapshots} |"
        )
    return "\n".join(lines) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("aggregate_json", type=Path)
    args = parser.parse_args()
    data = json.loads(args.aggregate_json.read_text(encoding="utf-8"))
    print(render(data), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
