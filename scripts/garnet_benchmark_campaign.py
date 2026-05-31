#!/usr/bin/env python3
"""Benchmark campaign inventory + anti-rot gate (S58).

Garnet's Criterion benchmark harnesses span the toolchain (parser, CST, interp,
VM, memory). `garnet_benchmark_no_run.py` already proves they *compile*; this
reporter inventories the full **campaign** — every bench, what it measures, and
the command to run it — and gates that the declared set still exists (a bench
cannot silently vanish).

## Honest scope (do not soften)
This inventories and verifies the harnesses **exist**; it does **not** run them
and reports **no measurements**. Criterion numbers are environment-specific and
are recorded by an explicit campaign run (`cargo bench`), not fabricated here.
This is the falsifiable, no-measurement stance of `garnet_proof_benchmark_status.py`.
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import asdict, dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


@dataclass(frozen=True)
class Bench:
    crate: str
    name: str
    measures: str


# The Criterion benchmark campaign (harness = false in each crate's Cargo.toml).
CAMPAIGN = (
    Bench("garnet-parser-v0.3", "parse", "tokenize + parse throughput on representative sources"),
    Bench("garnet-cst", "parse_cst_vs_ast", "rowan CST build vs. AST parse cost"),
    Bench("garnet-interp-v0.3", "eval", "tree-walking interpreter evaluation throughput"),
    Bench("garnet-vm", "parse_compile_execute", "end-to-end parse → compile → VM execute"),
    Bench("garnet-memory-v0.3", "vector", "kind-aware vector store operations"),
    Bench("garnet-memory-v0.3", "eviction", "memory eviction policy cost"),
)


@dataclass
class BenchStatus:
    crate: str
    name: str
    measures: str
    file_present: bool
    declared_in_cargo: bool
    run_command: str
    ok: bool


@dataclass
class BenchmarkCampaign:
    schema: str
    benches: list[BenchStatus]
    all_present: bool
    note: str


def _cargo_declares_bench(crate: str, name: str) -> bool:
    cargo = ROOT / crate / "Cargo.toml"
    if not cargo.is_file():
        return False
    text = cargo.read_text(encoding="utf-8")
    # A `[[bench]]` block naming `name`.
    return bool(re.search(r'\[\[bench\]\][^\[]*name\s*=\s*"' + re.escape(name) + r'"', text, re.S))


def read_campaign() -> BenchmarkCampaign:
    statuses = []
    for b in CAMPAIGN:
        bench_file = ROOT / b.crate / "benches" / f"{b.name}.rs"
        file_present = bench_file.is_file()
        declared = _cargo_declares_bench(b.crate, b.name)
        statuses.append(
            BenchStatus(
                crate=b.crate,
                name=b.name,
                measures=b.measures,
                file_present=file_present,
                declared_in_cargo=declared,
                run_command=f"cargo bench -p {b.crate} --bench {b.name}",
                ok=file_present and declared,
            )
        )
    return BenchmarkCampaign(
        schema="garnet.benchmark_campaign/v1",
        benches=statuses,
        all_present=all(s.ok for s in statuses),
        note=(
            "Harnesses inventoried + verified present; no measurements are run or "
            "claimed here. Run the campaign with the per-bench commands; record "
            "results separately (numbers are environment-specific)."
        ),
    )


def render_markdown(c: BenchmarkCampaign) -> str:
    lines = [
        "# Garnet benchmark campaign",
        "",
        f"_Schema {c.schema}. Inventory + run protocol — no measurements claimed._",
        "",
        "| crate | bench | measures | present | run |",
        "|---|---|---|---|---|",
    ]
    for s in c.benches:
        mark = "✅" if s.ok else "❌"
        lines.append(f"| {s.crate} | `{s.name}` | {s.measures} | {mark} | `{s.run_command}` |")
    lines += [
        "",
        f"**All {len(c.benches)} benches present + declared: {'yes' if c.all_present else 'NO'}.**",
        "",
        c.note,
        "",
    ]
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument(
        "--gate",
        action="store_true",
        help="exit non-zero if a declared bench's file or Cargo entry is missing",
    )
    args = parser.parse_args(list(argv) if argv is not None else None)

    campaign = read_campaign()
    if args.format == "md":
        print(render_markdown(campaign))
    else:
        print(json.dumps(asdict(campaign), indent=2))

    if args.gate and not campaign.all_present:
        missing = [f"{s.crate}::{s.name}" for s in campaign.benches if not s.ok]
        print(f"benchmark-campaign gate FAILED: missing/undeclared benches: {missing}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
