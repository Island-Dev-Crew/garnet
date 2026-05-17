#!/usr/bin/env python3
"""Compile-only evidence for Criterion benchmark harnesses used by phase readiness gates."""
from __future__ import annotations

import argparse
import hashlib
import json
import platform
import subprocess
import sys
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Callable, Sequence

ROOT = Path(__file__).resolve().parents[1]

LOG_FILES = [
    "garnet-benchmark-no-run.json",
    "garnet-benchmark-no-run.md",
]


@dataclass(frozen=True)
class NoRunHarness:
    id: str
    package: str
    bench_name: str
    command: str
    status: str
    return_code: int | None
    stdout_file: str
    stderr_file: str


@dataclass(frozen=True)
class BenchmarkNoRunStatus:
    source: str
    overall_status: str
    measurement_status: str
    mechanized_proof_status: str
    empirical_study_status: str
    current_truth: list[str]
    benchmarks: list[NoRunHarness]
    blocked_by: list[str]
    deferred: list[str]
    forbidden_claims: list[str]
    next_slices: list[str]
    metadata: dict[str, str]


def _planned_harnesses() -> list[tuple[str, str, str, str]]:
    from garnet_proof_benchmark_status import BENCHMARKS

    plans: list[tuple[str, str, str, str]] = []
    for item in BENCHMARKS:
        plans.append((
            item["id"],
            item["package"],
            item["bench_name"],
            f"{item['command']} --no-run",
        ))
    return plans


def _run_command(command: Sequence[str], cwd: Path) -> tuple[int, str, str]:
    proc = subprocess.run(command, cwd=cwd, check=False, text=True, capture_output=True)
    return proc.returncode, proc.stdout, proc.stderr


def _write_manifest(output_dir: Path) -> None:
    rows: list[str] = []
    for path in sorted(output_dir.iterdir()):
        if path.name == "MANIFEST.sha256" or not path.is_file():
            continue
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        rows.append(f"{digest}  {path.name}")
    output_dir.joinpath("MANIFEST.sha256").write_text("\n".join(rows) + "\n", encoding="utf-8")


def _collect_status(
    execute: bool,
    output_dir: Path | None = None,
    run_command: Callable[[Sequence[str], Path], tuple[int, str, str]] = _run_command,
) -> BenchmarkNoRunStatus:
    log_dir = output_dir
    if log_dir is not None:
        log_dir.mkdir(parents=True, exist_ok=True)
    plans = _planned_harnesses()
    harnesses: list[NoRunHarness] = []
    failed = False

    for index, (bench_id, package, bench_name, command) in enumerate(plans):
        cmd = ["cargo", "bench", "-p", package, "--bench", bench_name, "--no-run"]
        status = "planned"
        return_code: int | None = None
        stdout_file = f"{bench_id}.stdout.log"
        stderr_file = f"{bench_id}.stderr.log"

        if execute:
            status = "running"
            return_code, stdout, stderr = run_command(cmd, ROOT)
            if return_code != 0:
                failed = True
                status = "failed"
            else:
                status = "passed"
            if log_dir is not None:
                log_dir.joinpath(stdout_file).write_text(stdout, encoding="utf-8")
                log_dir.joinpath(stderr_file).write_text(stderr, encoding="utf-8")
        harnesses.append(
            NoRunHarness(
                id=bench_id,
                package=package,
                bench_name=bench_name,
                command=command,
                status=status,
                return_code=return_code,
                stdout_file=stdout_file,
                stderr_file=stderr_file,
            )
        )

    if execute:
        overall_status = "failed" if failed else "compile-verified"
    else:
        overall_status = "planned"

    metadata = {
        "command_count": str(len(plans)),
        "executed": str(execute).lower(),
        "platform": platform.platform(),
        "machine": platform.machine(),
        "python": sys.version.split(" ", 1)[0],
        "cargo": "",
        "rustc": "",
        "git": "",
    }
    try:
        metadata["cargo"] = subprocess.check_output(["cargo", "--version"], text=True).strip()
    except Exception:
        metadata["cargo"] = "unavailable"
    try:
        metadata["rustc"] = subprocess.check_output(["rustc", "--version"], text=True).strip()
    except Exception:
        metadata["rustc"] = "unavailable"
    try:
        metadata["git"] = subprocess.check_output(
            ["git", "-C", str(ROOT), "rev-parse", "--short", "HEAD"],
            text=True,
        ).strip()
    except Exception:
        metadata["git"] = "unavailable"

    return BenchmarkNoRunStatus(
        source=str(ROOT),
        overall_status=overall_status,
        measurement_status="not-measured",
        mechanized_proof_status="not-mechanized",
        empirical_study_status="pending",
        current_truth=[
            "Criterion harness compile commands are inventory-only evidence in this slice.",
            "No benchmark runtime timing, throughput, memory, or regression metric is collected.",
            "Mechanized proof and empirical study execution remain deferred in this step.",
        ],
        benchmarks=harnesses,
        blocked_by=[
            "external benchmark measurement campaign",
            "measurement variance handling",
            "mechanized benchmark soundness proof",
        ],
        deferred=[
            "benchmark timing measurement bundle",
            "empirical study execution",
            "native backend performance claims",
        ],
        forbidden_claims=[
            "benchmark performance values",
            "native backend speedup guarantees",
            "regression/coverage verdicts from compile-only runs",
            "production readiness from benchmark timing",
        ],
        next_slices=[
            "Record deterministic no-run evidence in each supported desktop environment.",
            "Add machine/variant metadata and reproducibility hash fields once CI stores matrix artifacts.",
            "Only collect benchmark timing after explicit measurement protocol is implemented.",
        ],
        metadata=metadata,
    )


def _render_markdown(status: BenchmarkNoRunStatus) -> str:
    lines = [
        "# Garnet Benchmark No-Run Compile Evidence",
        "",
        f"Source: `{status.source}`",
        f"Overall status: `{status.overall_status}`",
        f"Measurement status: `{status.measurement_status}`",
        f"Mechanized proof: `{status.mechanized_proof_status}`",
        f"Empirical study: `{status.empirical_study_status}`",
        "",
        "## Current Truth",
        "",
        *[f"- {truth}" for truth in status.current_truth],
        "",
        "## No-Run Harness Results",
        "",
        "| Harness | Package | Command | Status | Return Code | Stdout | Stderr |",
        "| --- | --- | --- | --- | --- | --- |",
    ]

    for bench in status.benchmarks:
        lines.append(
            f"| {bench.id} | `{bench.package}` | `{bench.command}` | "
            f"{bench.status} | {bench.return_code if bench.return_code is not None else '-'} | "
            f"{bench.stdout_file} | {bench.stderr_file} |"
        )

    lines.extend(
        [
            "",
            "## Constraints",
            "",
            "- No benchmark measurements are claimed by this output.",
            "- No mechanized proof evidence is claimed in this slice.",
            "- No performance regression conclusions are valid from compile-only execution.",
            "",
            "## Not Claimed",
            "",
            *[f"- {claim}: not claimed" for claim in status.forbidden_claims],
        ]
    )
    return "\n".join(lines) + "\n"


def write_bundle(status: BenchmarkNoRunStatus, output_dir: Path) -> None:
    output_dir.mkdir(parents=True, exist_ok=True)
    data = json.dumps(asdict(status), indent=2) + "\n"
    output_dir.joinpath("garnet-benchmark-no-run.json").write_text(data, encoding="utf-8")
    output_dir.joinpath("garnet-benchmark-no-run.md").write_text(_render_markdown(status), encoding="utf-8")
    _write_manifest(output_dir)


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output-dir", type=Path, default=None)
    parser.add_argument("--format", choices=("markdown", "json"), default="markdown")
    parser.add_argument(
        "--execute",
        action="store_true",
        help="Run cargo bench --no-run for each harness before writing evidence.",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    status = _collect_status(execute=args.execute, output_dir=args.output_dir)

    if args.output_dir is not None:
        write_bundle(status, args.output_dir)

    if args.format == "json":
        print(json.dumps(asdict(status), indent=2))
        return 0

    print(_render_markdown(status))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
