#!/usr/bin/env python3
"""Run the Paper VI Exp 1 pass@1 harness without overstating measurements."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parent
DEFAULT_TASKS = ROOT / "tasks" / "manifest.json"
REAL_PROVIDER_ENVS = {
    "openai": "OPENAI_API_KEY",
    "anthropic": "ANTHROPIC_API_KEY",
    "gemini": "GEMINI_API_KEY",
    "ollama": "OLLAMA_BASE_URL",
}


def load_manifest(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def task_prompt_hash(task_root: Path, task: dict[str, Any]) -> str:
    chunks = [
        task.get("id", ""),
        (task_root / task.get("prompt_file", "")).read_text(encoding="utf-8"),
        (task_root / task.get("public_spec_file", "")).read_text(encoding="utf-8"),
    ]
    return hashlib.sha256("\n".join(chunks).encode("utf-8")).hexdigest()


def iter_task_language_rows(manifest: dict[str, Any], max_tasks: int | None = None) -> list[tuple[dict[str, Any], str, dict[str, Any]]]:
    tasks = manifest.get("seed_tasks", [])
    if max_tasks is not None:
        tasks = tasks[:max_tasks]
    rows: list[tuple[dict[str, Any], str, dict[str, Any]]] = []
    for task in tasks:
        for language, language_info in sorted(task.get("languages", {}).items()):
            rows.append((task, language, language_info))
    return rows


def pending_row(
    *,
    provider: str,
    model: str,
    task: dict[str, Any],
    language: str,
    status: str,
    prompt_hash: str,
    reason: str,
) -> dict[str, Any]:
    return {
        "schema": "garnet.paper_vi_exp1_result/v1",
        "experiment": "paper_vi_exp1_llm_pass_at_1",
        "provider": provider,
        "model": model,
        "task_id": task["id"],
        "mode": task["mode"],
        "difficulty": task["difficulty"],
        "language": language,
        "prompt_hash": prompt_hash,
        "status": status,
        "measured": False,
        "pass_at_1": None,
        "reason": reason,
    }


def fixture_row(
    *,
    task: dict[str, Any],
    language: str,
    language_info: dict[str, Any],
    prompt_hash: str,
) -> dict[str, Any]:
    # Fixture mode proves scoring plumbing only. It is not a model result.
    passed = language == "garnet"
    return {
        "schema": "garnet.paper_vi_exp1_result/v1",
        "experiment": "paper_vi_exp1_llm_pass_at_1",
        "provider": "fixture",
        "model": "fixture-det-v0",
        "task_id": task["id"],
        "mode": task["mode"],
        "difficulty": task["difficulty"],
        "language": language,
        "prompt_hash": prompt_hash,
        "status": "measured-fixture",
        "measured": True,
        "pass_at_1": 1 if passed else 0,
        "reference_file": language_info["reference_file"],
        "expected_stdout": language_info["expected_stdout"],
        "claim": "fixture-only; not provider-backed",
    }


def write_jsonl(path: Path, records: list[dict[str, Any]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        "".join(json.dumps(record, sort_keys=True) + "\n" for record in records),
        encoding="utf-8",
    )


def write_json(path: Path, data: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def run(args: argparse.Namespace) -> dict[str, Any]:
    manifest = load_manifest(args.tasks)
    task_root = args.tasks.parent
    rows: list[dict[str, Any]] = []
    provider = args.provider
    model = args.model or ("fixture-det-v0" if provider == "fixture" else provider)
    provider_status = "measured-fixture" if provider == "fixture" else "pending-infra"

    if provider in REAL_PROVIDER_ENVS:
        env_name = REAL_PROVIDER_ENVS[provider]
        if not args.execute_provider or not os.environ.get(env_name):
            provider_status = "pending-credentials"
            reason = (
                f"{env_name} plus --execute-provider is required for a live run; "
                "S94 records the gate without claiming a model result"
            )
        else:
            provider_status = "pending-provider-adapter"
            reason = (
                "credentials are present, but this S94 harness does not score live "
                "provider completions yet"
            )
    else:
        reason = "provider-free harness run; no model call executed"

    for task, language, language_info in iter_task_language_rows(manifest, args.max_tasks):
        prompt_hash = task_prompt_hash(task_root, task)
        if provider == "fixture":
            rows.append(
                fixture_row(
                    task=task,
                    language=language,
                    language_info=language_info,
                    prompt_hash=prompt_hash,
                )
            )
        else:
            rows.append(
                pending_row(
                    provider=provider,
                    model=model,
                    task=task,
                    language=language,
                    status=provider_status,
                    prompt_hash=prompt_hash,
                    reason=reason,
                )
            )

    measured_rows = [row for row in rows if row.get("measured")]
    pass_rows = [row for row in measured_rows if row.get("pass_at_1") == 1]
    summary = {
        "schema": "garnet.paper_vi_exp1_run_summary/v1",
        "provider": provider,
        "model": model,
        "status": provider_status,
        "task_manifest_status": manifest.get("status", "unknown"),
        "seed_task_count": len(manifest.get("seed_tasks", [])),
        "target_task_count": manifest.get("target_task_count"),
        "rows": len(rows),
        "measured_rows": len(measured_rows),
        "pending_rows": len(rows) - len(measured_rows),
        "pass_rows": len(pass_rows),
        "claim": (
            "fixture-only plumbing proof; not a provider-backed measurement"
            if provider == "fixture"
            else "No provider-backed pass@1 measurement is claimed"
        ),
    }
    write_jsonl(args.output / "results.jsonl", rows)
    write_json(args.output / "summary.json", summary)
    return summary


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--provider", choices=["none", "fixture", "openai", "anthropic", "gemini", "ollama"], default="none")
    parser.add_argument("--tasks", type=Path, default=DEFAULT_TASKS)
    parser.add_argument("--output", type=Path, default=ROOT / "out")
    parser.add_argument("--model", default="")
    parser.add_argument("--max-tasks", type=int, default=None)
    parser.add_argument("--execute-provider", action="store_true")
    args = parser.parse_args(argv)
    summary = run(args)
    print(json.dumps(summary, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
