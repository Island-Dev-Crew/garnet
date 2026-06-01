#!/usr/bin/env python3
"""Render the S95 5K-LOC Exp 3 analysis boundary."""
from __future__ import annotations

import argparse
import json
from pathlib import Path


def render(data: dict) -> str:
    rows = data.get("lane_counts", {})
    lines = [
        "# Paper VI Exp 3 5K-LOC Rerun Analysis",
        "",
        f"Status: {data.get('status', 'unknown')}",
        "",
        "No 5K h3a measurement is claimed in provider-free mode.",
        "The recorded v4.0 6.5% partial stands until provider-backed 5K runtime rows exist and are reviewed.",
        "",
        "| Lane | Rows |",
        "|---|---:|",
    ]
    for lane in ("stateless", "history_aware"):
        lines.append(f"| {lane} | {rows.get(lane, 0)} |")
    lines.extend(
        [
            "",
            f"- snapshots: {data.get('snapshot_count', 0)}",
            f"- minimum snapshot LOC: {data.get('min_snapshot_loc', 0)}",
            f"- measured rows: {data.get('measured_rows', 0)}",
            f"- h3a status: {data.get('h3a_status', 'unknown')}",
            f"- claim boundary: {data.get('h3a_claim', '')}",
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
    markdown = render(data)
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(markdown, encoding="utf-8")
    print(markdown, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
