#!/usr/bin/env python3
"""Aggregate Paper VI Exp 3 JSONL logs without claiming study results."""
from __future__ import annotations

import argparse
import json
from pathlib import Path


def read_records(path: Path) -> list[dict]:
    records: list[dict] = []
    if not path.exists():
        return records
    for line in path.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        records.append(json.loads(line))
    return records


def aggregate(root: Path) -> dict:
    lanes = {}
    for lane_dir in sorted(path for path in root.iterdir() if path.is_dir()):
        records = []
        for log in sorted(lane_dir.glob("*.jsonl")):
            records.extend(read_records(log))
        lanes[lane_dir.name] = {
            "records": len(records),
            "snapshots": sorted({record.get("snapshot", "unknown") for record in records}),
            "missing_llm_log": sum(1 for record in records if record.get("status") == "missing-llm-log"),
        }
    return {
        "status": "harness-only",
        "claim": "no h3a/h3b/h3c results are produced by this aggregate step",
        "lanes": lanes,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("root", nargs="?", default="out", type=Path)
    args = parser.parse_args()
    print(json.dumps(aggregate(args.root), indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
