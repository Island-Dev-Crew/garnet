#!/usr/bin/env python3
"""Report Garnet proof, benchmark, and empirical evidence boundaries.

This is a status reporter, not a benchmark runner or mechanized proof. It makes
the current Phase 7 research lane falsifiable by inventorying the benchmark
harnesses and study/proof protocols that exist today while keeping measurements,
Coq/Iris/RustBelt mechanization, and external study results unclaimed.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from dataclasses import asdict, dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


@dataclass(frozen=True)
class BenchmarkHarness:
    id: str
    package: str
    bench_name: str
    bench_file: str
    cargo_toml: str
    command: str
    bench_file_exists: bool
    cargo_entry_present: bool
    measurement_status: str


@dataclass(frozen=True)
class ResearchProtocol:
    id: str
    label: str
    path: str
    exists: bool
    status: str
    boundary: str


@dataclass(frozen=True)
class ProofBenchmarkStatus:
    source: str
    overall_status: str
    measurement_status: str
    mechanized_proof_status: str
    empirical_study_status: str
    current_truth: list[str]
    benchmarks: list[BenchmarkHarness]
    protocols: list[ResearchProtocol]
    blocked_by: list[str]
    deferred: list[str]
    forbidden_claims: list[str]
    next_slices: list[str]


BENCHMARKS = (
    {
        "id": "parser_parse",
        "package": "garnet-parser",
        "bench_name": "parse",
        "bench_file": "garnet-parser-v0.3/benches/parse.rs",
        "cargo_toml": "garnet-parser-v0.3/Cargo.toml",
        "command": "cargo bench -p garnet-parser --bench parse",
    },
    {
        "id": "interp_eval",
        "package": "garnet-interp",
        "bench_name": "eval",
        "bench_file": "garnet-interp-v0.3/benches/eval.rs",
        "cargo_toml": "garnet-interp-v0.3/Cargo.toml",
        "command": "cargo bench -p garnet-interp --bench eval",
    },
    {
        "id": "memory_vector",
        "package": "garnet-memory",
        "bench_name": "vector",
        "bench_file": "garnet-memory-v0.3/benches/vector.rs",
        "cargo_toml": "garnet-memory-v0.3/Cargo.toml",
        "command": "cargo bench -p garnet-memory --bench vector",
    },
)

PROTOCOLS = (
    ResearchProtocol(
        id="empirical_validation_plan",
        label="Empirical validation plan",
        path="F_Project_Management/ROADMAPS/GARNET_EMPIRICAL_VALIDATION_PLAN.md",
        exists=(ROOT / "F_Project_Management/ROADMAPS/GARNET_EMPIRICAL_VALIDATION_PLAN.md").is_file(),
        status="protocol-only",
        boundary="No fresh participant or benchmark data is claimed by this reporter.",
    ),
    ResearchProtocol(
        id="developer_comprehension_study",
        label="Developer comprehension study protocol",
        path="F_Project_Management/GARNET_v4_2_Developer_Comprehension_Study_Protocol.md",
        exists=(ROOT / "F_Project_Management/GARNET_v4_2_Developer_Comprehension_Study_Protocol.md").is_file(),
        status="protocol-only",
        boundary="Study execution and participant data remain future work.",
    ),
    ResearchProtocol(
        id="paper_vi_execution",
        label="Paper VI empirical execution notes",
        path="F_Project_Management/GARNET_v4_0_PAPER_VI_EXECUTION.md",
        exists=(ROOT / "F_Project_Management/GARNET_v4_0_PAPER_VI_EXECUTION.md").is_file(),
        status="historical-or-protocol",
        boundary="Historical notes do not substitute for current benchmark measurements.",
    ),
)


def _cargo_entry_present(cargo_toml: Path, bench_name: str) -> bool:
    if not cargo_toml.is_file():
        return False
    text = cargo_toml.read_text(encoding="utf-8")
    return "[[bench]]" in text and f'name = "{bench_name}"' in text


def _benchmarks() -> list[BenchmarkHarness]:
    harnesses: list[BenchmarkHarness] = []
    for item in BENCHMARKS:
        bench_file = ROOT / item["bench_file"]
        cargo_toml = ROOT / item["cargo_toml"]
        harnesses.append(
            BenchmarkHarness(
                id=item["id"],
                package=item["package"],
                bench_name=item["bench_name"],
                bench_file=item["bench_file"],
                cargo_toml=item["cargo_toml"],
                command=item["command"],
                bench_file_exists=bench_file.is_file(),
                cargo_entry_present=_cargo_entry_present(cargo_toml, item["bench_name"]),
                measurement_status="not-run",
            )
        )
    return harnesses


def read_status() -> ProofBenchmarkStatus:
    return ProofBenchmarkStatus(
        source=str(ROOT),
        overall_status="active-scaffold",
        measurement_status="not-run",
        mechanized_proof_status="not-mechanized",
        empirical_study_status="pending",
        current_truth=[
            "Criterion benchmark harnesses exist for parser, interpreter, and memory surfaces.",
            "benchmarks compile/execution must be run separately",
            "No benchmark measurements are embedded in this status.",
            "Formal RustBelt/Iris/Coq mechanization is not present in the repo.",
            "Empirical study protocols exist, but external participant data remains pending.",
        ],
        benchmarks=_benchmarks(),
        protocols=list(PROTOCOLS),
        blocked_by=[
            "mechanized proof is not present",
            "external user study data",
            "fresh benchmark measurement run",
        ],
        deferred=[
            "formal RustBelt/Iris/Coq mechanization",
            "benchmark measurement run",
            "native backend proof",
            "external participant study execution",
        ],
        forbidden_claims=[
            "production native compiler proof",
            "zero-cost native backend guarantee",
            "PLDI-grade empirical validation complete",
            "formal mechanized soundness proof complete",
        ],
        next_slices=[
            "Run cargo bench no-run gates for parser/interpreter/memory in CI or a local evidence bundle.",
            "Add a benchmark measurement bundle only when machine, command, and variance metadata are recorded.",
            "Turn one proof sketch into a checked mechanization artifact before claiming formal proof.",
            "Execute the comprehension study only with consent, recruitment, and data-retention controls in place.",
        ],
    )


def render_markdown(status: ProofBenchmarkStatus) -> str:
    lines = [
        "# Garnet Proof, Benchmark, And Empirical Status",
        "",
        f"Source: `{status.source}`",
        f"Overall status: `{status.overall_status}`",
        f"Benchmark measurements: `{status.measurement_status}`",
        f"Mechanized proof: `{status.mechanized_proof_status}`",
        f"Empirical study: `{status.empirical_study_status}`",
        "",
        "## Current Truth",
        "",
        *[f"- {truth}" for truth in status.current_truth],
        "",
        "## Criterion Benchmark Harnesses",
        "",
        "| Harness | Package | Command | File | Cargo entry | Measurement |",
        "| --- | --- | --- | --- | --- | --- |",
    ]
    for bench in status.benchmarks:
        file_state = "present" if bench.bench_file_exists else "missing"
        cargo_state = "present" if bench.cargo_entry_present else "missing"
        lines.append(
            f"| {bench.id} | `{bench.package}` | `{bench.command}` | "
            f"{file_state} | {cargo_state} | `{bench.measurement_status}` |"
        )

    lines.extend(
        [
            "",
            "## Research Protocols",
            "",
            "| Protocol | Status | Path | Boundary |",
            "| --- | --- | --- | --- |",
        ]
    )
    for protocol in status.protocols:
        exists = "present" if protocol.exists else "missing"
        lines.append(
            f"| {protocol.label} | `{protocol.status}` ({exists}) | `{protocol.path}` | {protocol.boundary} |"
        )

    lines.extend(
        [
            "",
            "## Not Claimed",
            "",
            *[f"- {claim}: not {claim}" for claim in status.forbidden_claims],
            "",
            "## Next Slices",
            "",
            *[f"- {item}" for item in status.next_slices],
        ]
    )
    return "\n".join(lines) + "\n"


def _write_manifest(output_dir: Path) -> None:
    rows: list[str] = []
    for path in sorted(output_dir.iterdir()):
        if path.name == "MANIFEST.sha256" or not path.is_file():
            continue
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        rows.append(f"{digest}  {path.name}")
    (output_dir / "MANIFEST.sha256").write_text("\n".join(rows) + "\n", encoding="utf-8")


def write_bundle(status: ProofBenchmarkStatus, output_dir: Path) -> None:
    output_dir.mkdir(parents=True, exist_ok=True)
    data = json.dumps(asdict(status), indent=2)
    markdown = render_markdown(status)
    (output_dir / "garnet-proof-benchmark-status.json").write_text(data + "\n", encoding="utf-8")
    (output_dir / "garnet-proof-benchmark-status.md").write_text(markdown, encoding="utf-8")
    _write_manifest(output_dir)


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=("markdown", "json"), default="markdown")
    parser.add_argument("--output-dir", type=Path)
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args([] if argv is None else argv)
    status = read_status()
    if args.output_dir is not None:
        write_bundle(status, args.output_dir.expanduser().resolve())
    if args.format == "json":
        print(json.dumps(asdict(status), indent=2))
    else:
        print(render_markdown(status), end="")
    return 0


if __name__ == "__main__":
    import sys

    raise SystemExit(main(sys.argv[1:]))
