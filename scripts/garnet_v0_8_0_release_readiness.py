#!/usr/bin/env python3
"""v0.8.0 release-readiness gate (S60) — evidence for the tag decision.

Aggregates the whole v0.8 train into one verdict: are the hardening band
(S41–S49 + the S50 beta gate) and the adoption band (S51–S59) merged, and do all
their anti-rot sub-gates still pass? It renders a READY / NOT-READY verdict and
the honest in/deferred inventory for a `v0.8.0` release.

## CRITICAL honesty scope (do not soften)
This gate does **NOT** cut a tag. Since S83, `v0.8.0` may already appear in
`existing_tags` because Jon cut it as a separate release-truth act. "READY TO TAG"
means the evidence supports the decision — it is a recommendation, not the act.
Garnet remains a *research-grade prototype (v0.x.x), not production-complete*; the
verbatim honesty anchors are surfaced, not changed.
"""
from __future__ import annotations

import argparse
import json
import subprocess
import sys
from dataclasses import asdict, dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
# The release gate is historical v0.8 evidence. The active goal ledger moves to
# S91-S110 in v0.8.1, so v0.8 gates read the archived S31-S80 ledger.
GOAL_FILE = ROOT / ".dogfood" / "v0_8_goal.json"

HARDENING_BAND = [f"s4{i}" for i in range(1, 10)] + ["s50"]  # s41..s50
ADOPTION_BAND = [f"s5{i}" for i in range(1, 10)]  # s51..s59
MIN_CONFIDENCE = 5

# Anti-rot sub-gates across the train (all dependency-free or node-only).
SUB_GATES = [
    ("beta-gate (S50)", "garnet_v0_8_beta_gate.py"),
    ("build-proof (S47)", "garnet_build_proof.py"),
    ("proof-matrix (S48)", "garnet_proof_matrix.py"),
    ("signed-release-lanes (S51)", "garnet_signed_release_lanes.py"),
    ("install-readme (S52)", "garnet_install_readme_check.py"),
    ("tree-sitter (S53)", "garnet_tree_sitter_check.py"),
    ("vscode-publish (S54)", "garnet_vscode_publish_readiness.py"),
    ("wasm-readiness (S55)", "garnet_wasm_readiness.py"),
    ("playground (S56)", "garnet_playground_readiness.py"),
    ("benchmark-campaign (S58)", "garnet_benchmark_campaign.py"),
    ("fuzz-campaign (S59)", "garnet_fuzz_campaign.py"),
]

DEFERRED_FOR_V0_8_0 = [
    "Runtime sandbox enforcement (S46 generates seccomp/WASI/egress policy; does not enforce).",
    "Release-artifact + supply-chain signing lanes (S51 lanes 2–3 — GPG/minisign/cosign external).",
    "OpenVSX / VS Code Marketplace publish (S54 — needs OVSX_TOKEN/VSCE_PAT credentials).",
    "WASM build + browser playground execution (S55/S56 — wasm32/wasm-pack absent).",
    "LLM advisory tier (compiler-as-agent rules tier ships; LLM tier pending-infra).",
    "Empirical Paper VI measurements / mechanized proofs (S48 inventory, not proof).",
]

HONESTY_ANCHORS = [
    "research-grade prototype (v0.x.x) — not production-complete",
    "Paper VI scorecard: 4 supported, 2 partial (downgraded honestly), 0 refuted, 1 pending-infra",
    "human/aesthetic acceptance remains open",
]


@dataclass
class SliceState:
    id: str
    status: str
    ok: bool


@dataclass
class SubGate:
    name: str
    passed: bool
    exit_code: int


@dataclass
class ReleaseReadiness:
    schema: str
    hardening_band: list[SliceState]
    adoption_band: list[SliceState]
    sub_gates: list[SubGate]
    bands_complete: bool
    sub_gates_pass: bool
    release_ready: bool
    existing_tags: list[str]
    deferred_for_v0_8_0: list[str] = field(default_factory=list)
    honesty_anchors: list[str] = field(default_factory=list)
    tag_note: str = ""


def _band(ledger: dict, ids: list[str]) -> list[SliceState]:
    by_id = {s["id"]: s for s in ledger.get("slices", [])}
    out = []
    for sid in ids:
        s = by_id.get(sid, {})
        ok = s.get("status") == "merged" and (s.get("merge_confidence") or 0) >= MIN_CONFIDENCE
        out.append(SliceState(id=sid, status=s.get("status", "absent"), ok=ok))
    return out


def _run_sub_gate(script: str) -> SubGate:
    proc = subprocess.run(
        [sys.executable, str(ROOT / "scripts" / script), "--gate", "--format", "json"],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )
    return SubGate(name=script, passed=proc.returncode == 0, exit_code=proc.returncode)


def read_readiness() -> ReleaseReadiness:
    ledger = json.loads(GOAL_FILE.read_text(encoding="utf-8"))
    hb = _band(ledger, HARDENING_BAND)
    ab = _band(ledger, ADOPTION_BAND)
    sub_gates = [SubGate(name=n, passed=g.passed, exit_code=g.exit_code)
                 for (n, script) in SUB_GATES
                 for g in [_run_sub_gate(script)]]

    existing_tags = [
        t for t in subprocess.run(["git", "tag", "--list"], cwd=ROOT, capture_output=True, text=True).stdout.split()
        if t.startswith("v")
    ]

    bands_complete = all(s.ok for s in hb) and all(s.ok for s in ab)
    sub_gates_pass = all(g.passed for g in sub_gates)
    return ReleaseReadiness(
        schema="garnet.v0_8_0_release_readiness/v1",
        hardening_band=hb,
        adoption_band=ab,
        sub_gates=sub_gates,
        bands_complete=bands_complete,
        sub_gates_pass=sub_gates_pass,
        release_ready=bands_complete and sub_gates_pass,
        existing_tags=existing_tags,
        deferred_for_v0_8_0=DEFERRED_FOR_V0_8_0,
        honesty_anchors=HONESTY_ANCHORS,
        tag_note=(
            "This gate does NOT cut a tag. v0.8.0, when present in existing_tags, "
            "was cut by Jon as a separate release-truth decision. 'READY TO TAG' "
            "is a recommendation backed by evidence, not the act of tagging."
        ),
    )


def render_markdown(r: ReleaseReadiness) -> str:
    hb = sum(s.ok for s in r.hardening_band)
    ab = sum(s.ok for s in r.adoption_band)
    sg = sum(g.passed for g in r.sub_gates)
    lines = [
        "# Garnet v0.8.0 release readiness",
        "",
        f"_Schema {r.schema}. Evidence for the tag decision — NOT the tag itself._",
        "",
        f"**Verdict: {'READY TO TAG (pending Jon) ✅' if r.release_ready else 'NOT READY ❌'}**",
        "",
        f"- hardening band (S41–S50): {hb}/{len(r.hardening_band)} merged",
        f"- adoption band (S51–S59): {ab}/{len(r.adoption_band)} merged",
        f"- anti-rot sub-gates: {sg}/{len(r.sub_gates)} pass",
        f"- existing release tags: {', '.join(r.existing_tags) or 'none'}",
        "",
        "## Sub-gates",
        "",
        "| gate | result |",
        "|---|---|",
    ]
    for g in r.sub_gates:
        lines.append(f"| {g.name} | {'✅' if g.passed else f'❌ exit {g.exit_code}'} |")
    lines += ["", "## Deferred for v0.8.0 (honest)"]
    for d in r.deferred_for_v0_8_0:
        lines.append(f"- {d}")
    lines += ["", "## Honesty anchors (verbatim)"]
    for a in r.honesty_anchors:
        lines.append(f'- "{a}"')
    lines += ["", f"> {r.tag_note}", ""]
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument(
        "--gate",
        action="store_true",
        help="exit non-zero unless the v0.8 bands are merged and all sub-gates pass (does NOT tag)",
    )
    args = parser.parse_args(list(argv) if argv is not None else None)

    r = read_readiness()
    if args.format == "md":
        print(render_markdown(r))
    else:
        print(json.dumps(asdict(r), indent=2))

    if args.gate and not r.release_ready:
        print("v0.8.0 release-readiness gate: NOT READY (see bands + sub-gates).", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
