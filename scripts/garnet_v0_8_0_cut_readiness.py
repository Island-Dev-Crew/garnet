#!/usr/bin/env python3
"""Garnet v0.8.0 CUT readiness — the whole S30–S80 run (S80).

The corrected version map cuts the entire S30–S80 completion run as ONE `v0.8.0`
tag at the end of S80 (see `GARNET_v0_8_VERSION_MAP.md`). This aggregates the
whole run into a single READY / NOT-READY-TO-CUT verdict:

  1. **Ledger** — every slice S31..S79 is `merged` (S80 is this decision slice).
  2. **Foundation/hardening/adoption** — the S60 release-readiness gate passes
     (which itself re-runs the band gates + 11 anti-rot sub-gates).
  3. **Runway gates (S69..S79)** — each slice gate passes.

S86 keeps the default mode lenient for Python-only CI, but adds
`--binary-strict` / `--windows-audit` to run S71/S72/S73 direct proofs without
`--no-run`. That mode is the Windows-audit proof for WIN-S80-001.

> **This does NOT cut, push, or authorize any tag.** Cutting `v0.8.0` is a
> release-truth decision reserved for Jon. "READY TO CUT" is evidence-backed
> advice, not the act of tagging. Only `v0.4.2` / `v0.5.0` are tagged today.

## Honest scope (do not soften)
v0.8.0 is a **research-grade prototype** milestone, not a production/1.0 claim.
The deferred-for-v0.8.0 list (runtime sandbox enforcement, external signing,
marketplace publish, WASM execution, LLM tier, empirical Paper VI proofs) stands.
"""
from __future__ import annotations

import argparse
import json
import subprocess
import sys
from dataclasses import asdict, dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
# S91 makes .dogfood/goal.json the active v0.8.1 ledger. The v0.8.0 cut gate is
# historical release evidence, so it reads the archived S31-S80 ledger.
LEDGER = ROOT / ".dogfood" / "v0_8_goal.json"

# Every slice that must be merged before the run can be cut (S80 is the decision).
REQUIRED_MERGED = [f"s{n}" for n in range(31, 80)]  # s31..s79

# The S60 release-readiness gate (covers S41..S59 bands + 11 anti-rot sub-gates).
RELEASE_GATE = "garnet_v0_8_0_release_readiness.py"

# Runway slice gates (S69..S79). Binary-backed gates keep --no-run in the
# default mode so python-only CI remains deterministic without a built compiler.
# S86 adds an explicit binary-strict / Windows-audit mode that removes --no-run
# for those direct runtime proofs and treats failures as blocking.
RUNWAY_GATES = [
    ("llm-suggest (S69)", ["garnet_llm_suggest_readiness.py", "--gate"], False),
    ("version-map (S70)", ["garnet_version_map_check.py", "--gate"], False),
    ("paper-vi-exp3 (S71)", ["garnet_paper_vi_exp3_status.py", "--gate", "--no-run"], True),
    ("self-hosted-parser (S72)", ["garnet_self_hosted_parser_seed_status.py", "--gate", "--no-run"], True),
    ("vm-interp-parity (S73)", ["garnet_vm_interp_parity.py", "--gate", "--no-run"], True),
    ("safe-subset (S74)", ["garnet_safe_subset_status.py", "--gate"], False),
    ("formal-verification (S75)", ["garnet_formal_verification_feasibility.py", "--gate"], False),
    ("stdlib-promotion (S76)", ["garnet_stdlib_promotion_status.py", "--gate"], False),
    ("external-package (S77)", ["garnet_external_package_pilot_status.py", "--gate"], False),
    ("governance (S78)", ["garnet_governance_status.py", "--gate"], False),
    ("positioning (S79)", ["garnet_positioning_status.py", "--gate"], False),
]

HONESTY_ANCHORS = [
    "research-grade prototype (v0.x.x) — not production-complete",
    "Paper VI scorecard: 4 supported, 2 partial (downgraded honestly), 0 refuted, 1 pending-infra",
    "the v0.8.0 cut is a release-truth decision for Jon — not made by this gate",
]


@dataclass
class SubGate:
    name: str
    passed: bool
    exit_code: int


@dataclass
class CutReadiness:
    schema: str
    missing_merged: list[str]
    ledger_complete: bool
    release_gate: SubGate
    runway_gates: list[SubGate]
    runway_pass: bool
    cut_ready: bool
    mode: str = "lenient"
    binary_strict: bool = False
    honesty_anchors: list[str] = field(default_factory=lambda: list(HONESTY_ANCHORS))


def _run_gate(argv: list[str]) -> int:
    try:
        return subprocess.run(
            [sys.executable, str(ROOT / "scripts" / argv[0]), *argv[1:], "--format", "json"],
            capture_output=True,
            text=True,
            timeout=300,
        ).returncode
    except Exception:
        return 1


def runway_gate_specs(binary_strict: bool = False) -> list[tuple[str, list[str]]]:
    specs: list[tuple[str, list[str]]] = []
    for name, argv, binary_backed in RUNWAY_GATES:
        gate_argv = list(argv)
        if binary_strict and binary_backed:
            gate_argv = [arg for arg in gate_argv if arg != "--no-run"]
        specs.append((name, gate_argv))
    return specs


def read_readiness(binary_strict: bool = False) -> CutReadiness:
    ledger = json.loads(LEDGER.read_text(encoding="utf-8")) if LEDGER.is_file() else {"slices": []}
    status = {s["id"]: s.get("status") for s in ledger.get("slices", [])}
    missing = [i for i in REQUIRED_MERGED if status.get(i) != "merged"]
    ledger_complete = not missing

    rc = _run_gate([RELEASE_GATE, "--gate"])
    release = SubGate(name="release-readiness (S41–S59)", passed=rc == 0, exit_code=rc)

    runway = []
    for name, argv in runway_gate_specs(binary_strict=binary_strict):
        grc = _run_gate(argv)
        runway.append(SubGate(name=name, passed=grc == 0, exit_code=grc))
    runway_pass = all(g.passed for g in runway)

    cut_ready = ledger_complete and release.passed and runway_pass
    return CutReadiness(
        schema="garnet.v0_8_0_cut_readiness/v1",
        missing_merged=missing,
        ledger_complete=ledger_complete,
        release_gate=release,
        runway_gates=runway,
        runway_pass=runway_pass,
        cut_ready=cut_ready,
        mode="binary-strict" if binary_strict else "lenient",
        binary_strict=binary_strict,
    )


def render_markdown(r: CutReadiness) -> str:
    lines = [
        "# Garnet v0.8.0 CUT readiness — whole S30–S80 run (S80)",
        "",
        f"_Schema {r.schema}._",
        f"_Mode {r.mode}._",
        "",
        f"**Verdict: {'READY TO CUT (pending Jon) ✅' if r.cut_ready else 'NOT READY ❌'}**",
        "",
        f"- ledger S31–S79 all merged: {'yes' if r.ledger_complete else f'NO (missing {r.missing_merged})'}",
        f"- release-readiness gate (S41–S59 + 11 sub-gates): "
        f"{'pass' if r.release_gate.passed else 'FAIL'}",
        f"- runway gates (S69–S79): {sum(g.passed for g in r.runway_gates)}/{len(r.runway_gates)} pass",
    ]
    for g in r.runway_gates:
        lines.append(f"  - {g.name}: {'pass' if g.passed else 'FAIL'}")
    lines += [
        "",
        "**This gate does NOT cut a tag.** Cutting `v0.8.0` is a release-truth "
        "decision reserved for Jon; only `v0.4.2`/`v0.5.0` are tagged today. "
        "v0.8.0 is a research-grade-prototype milestone, not a production/1.0 claim.",
        "",
    ]
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument(
        "--gate",
        action="store_true",
        help="exit non-zero unless the whole run is cut-ready (ledger complete + "
        "release gate + all runway gates). Does NOT cut or authorize any tag.",
    )
    parser.add_argument(
        "--binary-strict",
        action="store_true",
        help="run S71/S72/S73 direct binary/provider-free gates instead of the "
        "default --no-run inventory mode; failures are blocking.",
    )
    parser.add_argument(
        "--windows-audit",
        action="store_true",
        help="alias for --binary-strict, named for the Windows audit lane that "
        "proved WIN-S80-001.",
    )
    args = parser.parse_args(list(argv) if argv is not None else None)

    binary_strict = args.binary_strict or args.windows_audit
    r = read_readiness(binary_strict=binary_strict)
    if args.format == "md":
        print(render_markdown(r))
    else:
        print(json.dumps(asdict(r), indent=2))

    if args.gate and not r.cut_ready:
        print(
            "v0.8.0-cut-readiness gate FAILED: "
            f"mode={r.mode} "
            f"ledger_complete={r.ledger_complete} missing={r.missing_merged} "
            f"release={r.release_gate.passed} runway_pass={r.runway_pass}",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
