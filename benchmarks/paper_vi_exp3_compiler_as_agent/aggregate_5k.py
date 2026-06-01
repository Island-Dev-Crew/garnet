#!/usr/bin/env python3
"""Aggregate S95 5K-LOC Exp 3 rows with an honest h3a boundary."""
from __future__ import annotations

import argparse
import json
from pathlib import Path
from statistics import mean

SCHEMA = "garnet.paper_vi_exp3_5k_aggregate/v1"


def _records(path: Path) -> list[dict]:
    rows: list[dict] = []
    for line in path.read_text(encoding="utf-8").splitlines():
        if line.strip():
            rows.append(json.loads(line))
    return rows


def aggregate(results_jsonl: Path) -> dict:
    rows = _records(results_jsonl)
    lane_counts = {
        "stateless": sum(1 for row in rows if row.get("lane") == "stateless"),
        "history_aware": sum(1 for row in rows if row.get("lane") == "history_aware"),
    }
    measured = [row for row in rows if row.get("measured")]
    locs = [int(row.get("snapshot_loc", 0)) for row in rows]
    h3a_ratio = None
    h3a_speedup_percent = None
    if measured:
        stateless = [float(row["duration_ms"]) for row in measured if row.get("lane") == "stateless"]
        history = [float(row["duration_ms"]) for row in measured if row.get("lane") == "history_aware"]
        if stateless and history:
            stateless_mean = mean(stateless)
            history_mean = mean(history)
            h3a_ratio = history_mean / stateless_mean
            h3a_speedup_percent = (1.0 - h3a_ratio) * 100.0

    pending = not measured
    return {
        "schema": SCHEMA,
        "status": "pending-provider-rerun" if pending else "measured-needs-review",
        "row_count": len(rows),
        "measured_rows": len(measured),
        "lane_counts": lane_counts,
        "snapshot_count": len({row.get("snapshot") for row in rows}),
        "min_snapshot_loc": min(locs) if locs else 0,
        "total_snapshot_loc": sum(locs) // max(len(lane_counts), 1) if locs else 0,
        "h3a_status": "pending-provider-rerun" if pending else "measured-needs-review",
        "h3a_ratio": h3a_ratio,
        "h3a_speedup_percent": h3a_speedup_percent,
        "h3a_claim": (
            "No 5K h3a measurement is claimed; recorded 6.5% partial stands."
            if pending
            else "Measured rows exist, but reviewer signoff is required before updating Paper VI claims."
        ),
    }


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("results_jsonl", type=Path)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args(argv)

    data = aggregate(args.results_jsonl)
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(json.dumps(data, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
