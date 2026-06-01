#!/usr/bin/env python3
"""Status gate for the S95 Paper VI Exp 3 5K-LOC rerun harness."""
from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
HARNESS = ROOT / "benchmarks" / "paper_vi_exp3_compiler_as_agent"
RUNNER = HARNESS / "run_5k.py"
AGGREGATE = HARNESS / "aggregate_5k.py"
ANALYZE = HARNESS / "analyze_5k.py"
STATUS_ROOT = ROOT / "target" / "paper_vi_exp3_5k_status"


@dataclass(frozen=True)
class PaperViExp3FiveKStatus:
    schema: str
    runner_present: bool
    aggregate_present: bool
    analyzer_present: bool
    snapshot_count: int
    min_snapshot_loc: int
    total_generated_loc: int
    provider_free_run_ok: bool
    stateless_rows: int
    history_aware_rows: int
    measured_rows: int
    h3a_status: str
    analysis_summary: str
    aggregate_ok: bool
    analyzer_ok: bool
    ok: bool


def _load_json(path: Path) -> dict[str, Any]:
    if not path.is_file():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))


def _run(command: list[str]) -> bool:
    completed = subprocess.run(
        command,
        cwd=ROOT,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=False,
    )
    return completed.returncode == 0


def _status_out() -> Path:
    return STATUS_ROOT / f"run-{os.getpid()}"


def _provider_free_run(status_out: Path) -> tuple[bool, dict[str, Any]]:
    ok = _run([sys.executable, str(RUNNER), "--provider", "none", "--output", str(status_out)])
    return ok, _load_json(status_out / "summary.json")


def _aggregate(status_out: Path) -> tuple[bool, dict[str, Any]]:
    output = status_out / "aggregate.json"
    ok = _run(
        [
            sys.executable,
            str(AGGREGATE),
            str(status_out / "results.jsonl"),
            "--output",
            str(output),
        ]
    )
    return ok, _load_json(output)


def _analyze(status_out: Path) -> tuple[bool, str]:
    output = status_out / "analysis.md"
    ok = _run([sys.executable, str(ANALYZE), str(status_out / "aggregate.json"), "--output", str(output)])
    text = output.read_text(encoding="utf-8") if output.is_file() else ""
    return ok and output.is_file(), text


def read_status() -> PaperViExp3FiveKStatus:
    status_out = _status_out()
    if status_out.exists():
        shutil.rmtree(status_out)
    provider_free_ok, summary = _provider_free_run(status_out) if RUNNER.is_file() else (False, {})
    aggregate_ok, aggregate = _aggregate(status_out) if provider_free_ok and AGGREGATE.is_file() else (False, {})
    analyzer_ok, analysis = _analyze(status_out) if aggregate_ok and ANALYZE.is_file() else (False, "")

    snapshot_count = int(summary.get("snapshot_count", 0))
    min_snapshot_loc = int(summary.get("min_snapshot_loc", 0))
    total_generated_loc = int(summary.get("total_generated_loc", 0))
    stateless_rows = int(summary.get("stateless_rows", 0))
    history_aware_rows = int(summary.get("history_aware_rows", 0))
    measured_rows = int(summary.get("measured_rows", 0))
    h3a_status = str(aggregate.get("h3a_status", summary.get("h3a_status", "missing")))

    ok = all(
        [
            RUNNER.is_file(),
            AGGREGATE.is_file(),
            ANALYZE.is_file(),
            provider_free_ok,
            aggregate_ok,
            analyzer_ok,
            snapshot_count == 10,
            min_snapshot_loc >= 5000,
            total_generated_loc >= 50000,
            stateless_rows == 10,
            history_aware_rows == 10,
            measured_rows == 0,
            h3a_status == "pending-provider-rerun",
            "No 5K h3a measurement is claimed" in analysis,
            "6.5% partial stands" in analysis,
        ]
    )
    return PaperViExp3FiveKStatus(
        schema="garnet.paper_vi_exp3_5k_status/v1",
        runner_present=RUNNER.is_file(),
        aggregate_present=AGGREGATE.is_file(),
        analyzer_present=ANALYZE.is_file(),
        snapshot_count=snapshot_count,
        min_snapshot_loc=min_snapshot_loc,
        total_generated_loc=total_generated_loc,
        provider_free_run_ok=provider_free_ok,
        stateless_rows=stateless_rows,
        history_aware_rows=history_aware_rows,
        measured_rows=measured_rows,
        h3a_status=h3a_status,
        analysis_summary=(
            "No 5K h3a measurement is claimed; the recorded 6.5% partial stands. "
            f"Provider-free harness rows: {stateless_rows} stateless, {history_aware_rows} history-aware."
        ),
        aggregate_ok=aggregate_ok,
        analyzer_ok=analyzer_ok,
        ok=ok,
    )


def render_markdown(status: PaperViExp3FiveKStatus) -> str:
    return "\n".join(
        [
            "# Garnet S95 Paper VI Exp 3 5K-LOC harness status",
            "",
            f"_Schema {status.schema}._",
            "",
            f"- snapshots generated: {status.snapshot_count}",
            f"- minimum snapshot LOC: {status.min_snapshot_loc}",
            f"- total generated LOC: {status.total_generated_loc}",
            f"- provider-free rows: {status.stateless_rows} stateless, {status.history_aware_rows} history-aware",
            f"- measured rows: {status.measured_rows}",
            f"- h3a status: {status.h3a_status}",
            f"- aggregate output: {'yes' if status.aggregate_ok else 'NO'}",
            f"- analysis output: {'yes' if status.analyzer_ok else 'NO'}",
            "",
            "No 5K h3a measurement is claimed. The recorded 6.5% partial stands until "
            "provider-backed 5K runtime rows exist and are reviewed.",
            "",
        ]
    )


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument("--gate", action="store_true")
    args = parser.parse_args(list(argv) if argv is not None else None)

    status = read_status()
    if args.format == "md":
        print(render_markdown(status))
    else:
        print(json.dumps(asdict(status), indent=2, sort_keys=True))

    if args.gate and not status.ok:
        print(f"Paper VI Exp 3 5K harness gate FAILED: {asdict(status)}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
