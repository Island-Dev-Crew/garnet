#!/usr/bin/env python3
"""Aggregate Paper VI Exp 1 JSONL rows without inventing provider results."""
from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any


def read_records(path: Path) -> list[dict[str, Any]]:
    records: list[dict[str, Any]] = []
    if not path.exists():
        return records
    for line in path.read_text(encoding="utf-8").splitlines():
        if line.strip():
            records.append(json.loads(line))
    return records


def aggregate(path: Path) -> dict[str, Any]:
    records = read_records(path)
    measured = [record for record in records if record.get("measured")]
    pending = [record for record in records if not record.get("measured")]
    pass_rows = [record for record in measured if record.get("pass_at_1") == 1]
    by_language: dict[str, dict[str, Any]] = {}
    for record in records:
        language = record.get("language", "unknown")
        lane = by_language.setdefault(language, {"rows": 0, "measured_rows": 0, "pass_rows": 0})
        lane["rows"] += 1
        if record.get("measured"):
            lane["measured_rows"] += 1
            if record.get("pass_at_1") == 1:
                lane["pass_rows"] += 1
    for lane in by_language.values():
        measured_count = lane["measured_rows"]
        lane["pass_at_1"] = (lane["pass_rows"] / measured_count) if measured_count else None
    return {
        "schema": "garnet.paper_vi_exp1_aggregate/v1",
        "status": "measured-fixture" if measured else "pending-infra",
        "rows": len(records),
        "measured_rows": len(measured),
        "pending_rows": len(pending),
        "pass_rows": len(pass_rows),
        "pass_at_1": (len(pass_rows) / len(measured)) if measured else None,
        "by_language": by_language,
        "claim": (
            "measured rows are fixture-only unless provider != fixture is recorded in the input"
            if measured
            else "No provider-backed pass@1 measurement is claimed"
        ),
    }


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("results_jsonl", type=Path)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args(argv)
    data = aggregate(args.results_jsonl)
    rendered = json.dumps(data, indent=2, sort_keys=True)
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(rendered + "\n", encoding="utf-8")
    print(rendered)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
