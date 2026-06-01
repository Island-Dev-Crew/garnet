#!/usr/bin/env python3
"""v0.8 beta gate (S50) — closes the S41-S50 hardening band.

A band-completion **checkpoint**, not a release. It verifies that the v0.8
hardening band (S41-S49) is merged and that the band's anti-rot sub-gates still
hold, then reports what the band shipped and what is explicitly deferred for the
v0.8 beta.

## CRITICAL honesty scope (do not soften)

This gate does **not** cut a tag and does **not** claim production readiness.
Garnet remains a *research-grade prototype (v0.x.x) — not production-complete*;
cutting `v0.8.0-beta` (or any tag) is a release-truth decision for Jon, not made
here. The verbatim honesty anchors below are surfaced, not changed.
"""
from __future__ import annotations

import argparse
import json
import subprocess
import sys
from dataclasses import asdict, dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
# The beta gate is historical v0.8 evidence. The active goal ledger moves to
# S91-S110 in v0.8.1, so v0.8 gates read the archived S31-S80 ledger.
GOAL_FILE = ROOT / ".dogfood" / "v0_8_goal.json"

# The v0.8 hardening band's implemented slices. S50 is this gate itself.
HARDENING_BAND = [f"s4{i}" for i in range(1, 10)]  # s41..s49
MIN_CONFIDENCE = 5

# Sub-gates the band installed (each is independently CI-wired; the beta gate
# re-runs them so "band complete" also means "the band's guarantees still hold").
SUB_GATES = [
    ("build-proof (S47 cross-OS coverage)", ["scripts/garnet_build_proof.py", "--gate"]),
    ("proof-matrix (S48 evidence anchors)", ["scripts/garnet_proof_matrix.py", "--gate"]),
]

# Verbatim honesty anchors (GARNET_v0_5_SLICE_DOGFOOD.md § Honesty Anchors).
# Surfaced by the gate; never softened.
HONESTY_ANCHORS = [
    "research-grade prototype (v0.x.x) — not production-complete",
    "tracked-slice ledger is complete, but that is not full MIT/productization completion",
    "Paper VI scorecard: 4 supported, 2 partial (downgraded honestly), 0 refuted, 1 pending-infra",
    "production allocator path tracked in MEMORY_CORE_ROADMAP.md",
    "human/aesthetic acceptance remains open",
]

# What the band deliberately does NOT deliver in beta (honest deferrals).
DEFERRED_FOR_BETA = [
    "Runtime sandbox enforcement (S46 generates seccomp/WASI/egress policy; it does not enforce — needs wasmtime / a Linux seccomp host).",
    "Windows CLI distribution (S47): only the separate Studio installer exists.",
    "LLM advisory tier (compiler-as-agent): rules tier ships; the LLM tier remains pending-infra.",
    "Cross-package LSP precision (S44 deferred the cross-file half to the package resolver line).",
    "Empirical Paper VI measurements / mechanized proofs (S48 is an evidence inventory, not proof).",
]


@dataclass
class SliceState:
    id: str
    title: str
    status: str
    merge_confidence: int | None
    ok: bool


@dataclass
class SubGate:
    name: str
    passed: bool
    exit_code: int


@dataclass
class BetaGate:
    schema: str
    band_slices: list[SliceState]
    sub_gates: list[SubGate]
    band_complete: bool
    sub_gates_pass: bool
    beta_ready: bool
    deferred_for_beta: list[str]
    honesty_anchors: list[str]
    tag_note: str


def _load_ledger() -> dict:
    return json.loads(GOAL_FILE.read_text(encoding="utf-8"))


def _band_states(ledger: dict) -> list[SliceState]:
    by_id = {s["id"]: s for s in ledger.get("slices", [])}
    states: list[SliceState] = []
    for sid in HARDENING_BAND:
        s = by_id.get(sid, {})
        conf = s.get("merge_confidence")
        ok = s.get("status") == "merged" and (conf or 0) >= MIN_CONFIDENCE
        states.append(
            SliceState(
                id=sid,
                title=s.get("title", "<absent>"),
                status=s.get("status", "absent"),
                merge_confidence=conf,
                ok=ok,
            )
        )
    return states


def _run_sub_gates() -> list[SubGate]:
    results: list[SubGate] = []
    for name, argv in SUB_GATES:
        proc = subprocess.run(
            [sys.executable, str(ROOT / argv[0]), *argv[1:]],
            cwd=ROOT,
            capture_output=True,
            text=True,
        )
        results.append(SubGate(name=name, passed=proc.returncode == 0, exit_code=proc.returncode))
    return results


def read_beta_gate() -> BetaGate:
    ledger = _load_ledger()
    band = _band_states(ledger)
    sub_gates = _run_sub_gates()
    band_complete = all(s.ok for s in band)
    sub_gates_pass = all(g.passed for g in sub_gates)
    return BetaGate(
        schema="garnet.v0_8_beta_gate/v1",
        band_slices=band,
        sub_gates=sub_gates,
        band_complete=band_complete,
        sub_gates_pass=sub_gates_pass,
        beta_ready=band_complete and sub_gates_pass,
        deferred_for_beta=DEFERRED_FOR_BETA,
        honesty_anchors=HONESTY_ANCHORS,
        tag_note=(
            "This gate does NOT cut a tag and does NOT claim production readiness. "
            "Cutting v0.8.0-beta (or any tag) is a release-truth decision for Jon."
        ),
    )


def render_markdown(g: BetaGate) -> str:
    lines = [
        "# Garnet v0.8 beta gate (S41-S50 hardening band)",
        "",
        f"_Schema {g.schema}. A band-completion checkpoint — not a release._",
        "",
        f"**Beta gate: {'OPEN ✅' if g.beta_ready else 'NOT OPEN ❌'}** "
        f"(band complete: {g.band_complete}; sub-gates pass: {g.sub_gates_pass})",
        "",
        "## Hardening band (S41-S49)",
        "",
        "| slice | title | status | conf |",
        "|---|---|---|---|",
    ]
    for s in g.band_slices:
        mark = "✅" if s.ok else "❌"
        lines.append(f"| {s.id} | {s.title} | {s.status} {mark} | {s.merge_confidence} |")
    lines += ["", "## Band sub-gates", "", "| gate | result |", "|---|---|"]
    for sg in g.sub_gates:
        lines.append(f"| {sg.name} | {'✅ pass' if sg.passed else f'❌ exit {sg.exit_code}'} |")
    lines += ["", "## Deferred for v0.8 beta (honest)"]
    for d in g.deferred_for_beta:
        lines.append(f"- {d}")
    lines += ["", "## Honesty anchors (verbatim — not softened)"]
    for a in g.honesty_anchors:
        lines.append(f"- \"{a}\"")
    lines += ["", f"> {g.tag_note}", ""]
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument(
        "--gate",
        action="store_true",
        help="exit non-zero unless the hardening band is complete and sub-gates pass",
    )
    args = parser.parse_args(list(argv) if argv is not None else None)

    gate = read_beta_gate()
    if args.format == "md":
        print(render_markdown(gate))
    else:
        print(json.dumps(asdict(gate), indent=2))

    if args.gate and not gate.beta_ready:
        incomplete = [s.id for s in gate.band_slices if not s.ok]
        failed = [g.name for g in gate.sub_gates if not g.passed]
        print(
            "v0.8 beta gate NOT OPEN — "
            f"incomplete slices: {incomplete or 'none'}; failed sub-gates: {failed or 'none'}",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
