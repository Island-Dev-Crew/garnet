#!/usr/bin/env python3
"""S94 Paper VI Exp 1 provider-gated harness status.

Passing means the repo contains a reproducible provider-free harness and a
deterministic fixture scorer for the Paper VI Exp 1 task shape. It does not mean
Garnet has measured provider-backed LLM pass@1.
"""
from __future__ import annotations

import argparse
import json
import subprocess
import sys
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
HARNESS = ROOT / "benchmarks" / "paper_vi_exp1_llm_pass_at_1"
RUNNER = HARNESS / "run.py"
AGGREGATE = HARNESS / "aggregate.py"
ANALYZE = HARNESS / "analyze.py"
MANIFEST = HARNESS / "tasks" / "manifest.json"
STATUS_OUT = ROOT / "target" / "paper_vi_exp1_status"


@dataclass
class PaperViExp1Status:
    schema: str
    harness_dir_present: bool
    runner_present: bool
    aggregate_present: bool
    analyzer_present: bool
    manifest_present: bool
    seed_task_count: int
    full_corpus_status: str
    provider_flag_present: bool
    provider_free_run_ok: bool
    provider_free_status: str
    provider_free_claims_measurement: bool
    fixture_run_ok: bool
    fixture_measured_rows: int
    fixture_pass_rows: int
    fixture_aggregate_ok: bool
    analyzer_ok: bool
    provider_backed_status: str
    ok: bool = False


def _read(path: Path) -> str:
    return path.read_text(encoding="utf-8") if path.is_file() else ""


def _load_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8")) if path.is_file() else {}


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


def _run_harness(provider: str, out_dir: Path) -> tuple[bool, dict[str, Any]]:
    ok = _run([sys.executable, str(RUNNER), "--provider", provider, "--output", str(out_dir)])
    return ok, _load_json(out_dir / "summary.json")


def _run_aggregate(out_dir: Path) -> tuple[bool, dict[str, Any]]:
    aggregate_json = out_dir / "aggregate.json"
    ok = _run(
        [
            sys.executable,
            str(AGGREGATE),
            str(out_dir / "results.jsonl"),
            "--output",
            str(aggregate_json),
        ]
    )
    return ok, _load_json(aggregate_json)


def _run_analyzer(out_dir: Path) -> bool:
    return _run(
        [
            sys.executable,
            str(ANALYZE),
            str(out_dir / "aggregate.json"),
            "--output",
            str(out_dir / "analysis.md"),
        ]
    ) and (out_dir / "analysis.md").is_file()


def read_status() -> PaperViExp1Status:
    manifest = _load_json(MANIFEST)
    seed_task_count = len(manifest.get("seed_tasks", []))
    runner_text = _read(RUNNER)
    provider_flag_present = (
        "--provider" in runner_text
        and "openai" in runner_text
        and "anthropic" in runner_text
        and "gemini" in runner_text
        and "ollama" in runner_text
    )

    none_dir = STATUS_OUT / "none"
    fixture_dir = STATUS_OUT / "fixture"
    provider_dir = STATUS_OUT / "openai-no-creds"

    provider_free_run_ok, provider_free_summary = _run_harness("none", none_dir)
    fixture_run_ok, fixture_summary = _run_harness("fixture", fixture_dir)
    fixture_aggregate_ok, fixture_aggregate = _run_aggregate(fixture_dir)
    analyzer_ok = _run_analyzer(fixture_dir)
    _, provider_summary = _run_harness("openai", provider_dir)

    provider_free_claims_measurement = provider_free_summary.get("measured_rows", 0) > 0
    ok = all(
        [
            HARNESS.is_dir(),
            RUNNER.is_file(),
            AGGREGATE.is_file(),
            ANALYZE.is_file(),
            MANIFEST.is_file(),
            seed_task_count >= 3,
            manifest.get("status") == "seed-only",
            provider_flag_present,
            provider_free_run_ok,
            provider_free_summary.get("status") == "pending-infra",
            not provider_free_claims_measurement,
            fixture_run_ok,
            fixture_summary.get("measured_rows") == 6,
            fixture_summary.get("pass_rows") == 3,
            fixture_aggregate_ok,
            fixture_aggregate.get("measured_rows") == 6,
            fixture_aggregate.get("pass_rows") == 3,
            analyzer_ok,
            provider_summary.get("status") == "pending-credentials",
        ]
    )
    return PaperViExp1Status(
        schema="garnet.paper_vi_exp1_status/v1",
        harness_dir_present=HARNESS.is_dir(),
        runner_present=RUNNER.is_file(),
        aggregate_present=AGGREGATE.is_file(),
        analyzer_present=ANALYZE.is_file(),
        manifest_present=MANIFEST.is_file(),
        seed_task_count=seed_task_count,
        full_corpus_status=manifest.get("status", "missing"),
        provider_flag_present=provider_flag_present,
        provider_free_run_ok=provider_free_run_ok,
        provider_free_status=provider_free_summary.get("status", "missing"),
        provider_free_claims_measurement=provider_free_claims_measurement,
        fixture_run_ok=fixture_run_ok,
        fixture_measured_rows=int(fixture_summary.get("measured_rows", 0)),
        fixture_pass_rows=int(fixture_summary.get("pass_rows", 0)),
        fixture_aggregate_ok=fixture_aggregate_ok,
        analyzer_ok=analyzer_ok,
        provider_backed_status=provider_summary.get("status", "missing"),
        ok=ok,
    )


def render_markdown(status: PaperViExp1Status) -> str:
    return "\n".join(
        [
            "# Garnet S94 Paper VI Exp 1 harness status",
            "",
            f"_Schema {status.schema}._",
            "",
            f"- harness present: {'yes' if status.harness_dir_present else 'NO'}",
            f"- seed task count: {status.seed_task_count}",
            f"- full corpus status: {status.full_corpus_status}",
            f"- provider flag present: {'yes' if status.provider_flag_present else 'NO'}",
            f"- provider-free run: {status.provider_free_status}",
            f"- fixture measured rows: {status.fixture_measured_rows}",
            f"- fixture pass rows: {status.fixture_pass_rows}",
            f"- real provider path: {status.provider_backed_status}",
            "",
            "No provider-backed pass@1 measurement is claimed. S94 wires the "
            "harness and proves provider-free plus deterministic fixture paths; "
            "credentials, hidden-test scoring, and the full registered corpus "
            "remain pending infrastructure.",
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
        print(f"Paper VI Exp 1 harness gate FAILED: {asdict(status)}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
