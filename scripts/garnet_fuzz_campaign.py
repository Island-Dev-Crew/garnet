#!/usr/bin/env python3
"""Fuzz campaign inventory + anti-rot gate (S59).

Garnet fuzzes its parser with libFuzzer (`cargo fuzz`). This reporter inventories
the campaign — the target, the crate it exercises, the nightly run protocol, and
the seed corpus — and gates that the harness stays wired (the target, its Cargo
`[[bin]]`, the nightly workflow reference, and a non-empty seed corpus).

## Honest scope (do not soften)
This inventories + verifies the fuzz harness **exists and is wired**; it does
**not** run the fuzzer and makes **no** claim about bugs found (or not found).
Crashes are surfaced by the nightly `cargo fuzz run` job, not by this gate.
`cargo-fuzz` is not present in this environment, so the harness is verified
structurally (files + workflow wiring + seeds), not built here.
"""
from __future__ import annotations

import argparse
import json
import sys
from dataclasses import asdict, dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FUZZ_DIR = ROOT / "garnet-parser-v0.3" / "fuzz"
TARGET = "parse_input"
TARGET_FILE = FUZZ_DIR / "fuzz_targets" / f"{TARGET}.rs"
FUZZ_CARGO = FUZZ_DIR / "Cargo.toml"
CORPUS_DIR = FUZZ_DIR / "corpus" / TARGET
WORKFLOW = ROOT / ".github" / "workflows" / "fuzz-nightly.yml"


@dataclass
class FuzzCampaign:
    schema: str
    target: str
    crate: str
    exercises: str
    target_file_present: bool
    declared_in_cargo: bool
    workflow_wired: bool
    seed_count: int
    run_command: str
    note: str
    ok: bool


def read_campaign() -> FuzzCampaign:
    cargo_text = FUZZ_CARGO.read_text(encoding="utf-8") if FUZZ_CARGO.is_file() else ""
    workflow_text = WORKFLOW.read_text(encoding="utf-8") if WORKFLOW.is_file() else ""
    seeds = list(CORPUS_DIR.glob("*.garnet")) if CORPUS_DIR.is_dir() else []

    target_present = TARGET_FILE.is_file()
    declared = f'name = "{TARGET}"' in cargo_text
    workflow_wired = TARGET in workflow_text and "cargo fuzz" in workflow_text
    seed_count = len(seeds)

    return FuzzCampaign(
        schema="garnet.fuzz_campaign/v1",
        target=TARGET,
        crate="garnet-parser-v0.3",
        exercises="parse_source_with_budget (lexer + parser) under a strict ParseBudget",
        target_file_present=target_present,
        declared_in_cargo=declared,
        workflow_wired=workflow_wired,
        seed_count=seed_count,
        run_command=f"cargo fuzz run {TARGET} (nightly, >= 1 hour — fuzz-nightly.yml)",
        note=(
            "Harness inventoried + verified wired; the fuzzer is NOT run here and no "
            "bug-found (or bug-free) claim is made. Crashes surface in the nightly "
            "`cargo fuzz run` job. cargo-fuzz is absent in this environment."
        ),
        ok=target_present and declared and workflow_wired and seed_count > 0,
    )


def render_markdown(c: FuzzCampaign) -> str:
    return "\n".join(
        [
            "# Garnet fuzz campaign",
            "",
            f"_Schema {c.schema}. Inventory + run protocol — no bug claims._",
            "",
            f"- target: `{c.target}` ({c.crate})",
            f"- exercises: {c.exercises}",
            f"- target file present: {c.target_file_present}",
            f"- declared in fuzz Cargo.toml: {c.declared_in_cargo}",
            f"- nightly workflow wired: {c.workflow_wired}",
            f"- seed corpus: {c.seed_count} seeds",
            f"- run: `{c.run_command}`",
            "",
            f"**Fuzz harness wired + seeded: {'yes' if c.ok else 'NO'}.**",
            "",
            c.note,
            "",
        ]
    )


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument(
        "--gate",
        action="store_true",
        help="exit non-zero if the fuzz target, Cargo entry, workflow wiring, or seeds are missing",
    )
    args = parser.parse_args(list(argv) if argv is not None else None)

    c = read_campaign()
    if args.format == "md":
        print(render_markdown(c))
    else:
        print(json.dumps(asdict(c), indent=2))

    if args.gate and not c.ok:
        print(
            "fuzz-campaign gate FAILED: "
            f"target_file={c.target_file_present}, declared={c.declared_in_cargo}, "
            f"workflow_wired={c.workflow_wired}, seeds={c.seed_count}",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
