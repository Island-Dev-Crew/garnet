#!/usr/bin/env python3
"""v0.8.1 release-readiness gate (S119) — evidence for the cut decision.

The whole-runway aggregator for the v0.8.1 milestone (the S60/S80/S86 lineage). It
asks one question: is the v0.8.1 runway (s91–s118) merged, do all its anti-rot
sub-gates still pass, AND is the binary-backed cross-OS + integrity evidence real —
so that the cut decision is supported by evidence, not by a green that hid failures?

It is **binary-strict by default** (the S86 lesson, WIN-S80-001): READY requires the
recorded cross-OS trap-parity matrix to be present AND `cross_os_complete=true` AND
the evidence-integrity gate to verify every sealed bundle — not merely that docs
exist. `--lenient` drops the cross-OS-matrix hard requirement for a python-only CI
job; default is strict.

## CRITICAL honesty scope (do not soften)
This gate does **NOT** cut a tag and does **NOT** push one. "READY TO CUT" means the
evidence supports the decision — it is a recommendation. The v0.8.1 cut (s120) is a
human act reserved to Jon; the release TAG stays Jon's. Garnet remains a
**research-grade prototype (v0.x.x), not production-complete**; the named-deferred
ceilings (OS-sandbox on macOS/Windows, @bounded fuel, memory, time, @mailbox,
live-LLM agent, external signing/SBOM, adopted standard) are surfaced, not hidden.
"""
from __future__ import annotations

import argparse
import json
import subprocess
import sys
from dataclasses import asdict, dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
GOAL_FILE = ROOT / ".dogfood" / "goal.json"
CROSS_OS_MATRIX = (
    ROOT
    / "proofs"
    / "cross-os"
    / "matrix"
    / "cross-os-trap-parity-20260604-s109"
    / "garnet-cross-os-trap-parity-matrix.json"
)

# The v0.8.1 evidence runway that must be merged for the cut to be supported.
# s119 (this gate) and s120 (the cut) are excluded by design — a readiness gate
# cannot require itself or the act it informs to already be done.
RUNWAY = [f"s{n}" for n in range(91, 119)] + ["s105b"]

# The anti-rot sub-gates: each must still pass (run live, --gate).
SUB_GATES = [
    ("red-team (S114)", "garnet_red_team_status.py"),
    ("ultrapunch-dossier (S115)", "garnet_ultrapunch_dossier_status.py"),
    ("domain-proof-artifacts (S116)", "garnet_domain_proof_artifacts_status.py"),
    ("academic-evidence (S118)", "garnet_academic_evidence_status.py"),
    ("evidence-integrity (S113)", "garnet_evidence_integrity_status.py"),
]

DEFERRED_FOR_V0_8_1 = [
    "OS-sandbox APPLICATION on macOS (sandbox-exec) and Windows (AppContainer) — "
    "Linux seccomp is the only kernel where the generated policy is applied.",
    "@bounded (Wasmtime fuel), memory, time, and @mailbox runtime ceilings — "
    "declared-not-enforced (only @caps + @max_depth are enforced).",
    "Live-LLM agent — the agent-loop proposer is simulated/scripted "
    "(provider_api_called=false).",
    "External signing / SBOM (cosign/syft/CycloneDX) — absent; seals unsigned, "
    "transparency log is a local stub (not Rekor).",
    "Adopted capability-manifest standard — RFC-0001 is intent + reference impl; "
    "no OWASP/LF body has adopted anything.",
    "Two LOW red-team findings (caps-log tail; seal subject-digest) — open within "
    "their honest stub/mitigated scope.",
]

HONESTY_ANCHORS = [
    "research-grade prototype (v0.x.x) — not production-complete",
    "no production / 1.0 claim; the v0.8.1 cut (s120) is Jon's; the tag stays Jon's",
    "cross-OS parity is at the language-runtime trap layer; OS-sandbox application is "
    "Linux-only",
]


@dataclass
class SliceState:
    id: str
    status: str
    merged: bool


@dataclass
class SubGate:
    name: str
    script: str
    passed: bool
    exit_code: int


@dataclass
class ReleaseReadiness:
    schema: str
    runway: list[SliceState]
    sub_gates: list[SubGate]
    runway_complete: bool
    sub_gates_pass: bool
    cross_os_complete: bool
    integrity_ok: bool
    binary_strict: bool
    release_ready: bool
    low_confidence_slices: list[str] = field(default_factory=list)
    deferred_for_v0_8_1: list[str] = field(default_factory=list)
    honesty_anchors: list[str] = field(default_factory=list)
    tag_note: str = ""


def _runway(ledger: dict) -> list[SliceState]:
    by_id = {s["id"]: s for s in ledger.get("slices", [])}
    out = []
    for sid in RUNWAY:
        s = by_id.get(sid, {})
        out.append(
            SliceState(
                id=sid,
                status=s.get("status", "absent"),
                merged=s.get("status") == "merged",
            )
        )
    return out


def _low_confidence(ledger: dict) -> list[str]:
    """Merged runway slices whose recorded confidence is below 5 (honest surfacing,
    not a hard fail — e.g. s107's null-confidence Mac-Codex row)."""
    by_id = {s["id"]: s for s in ledger.get("slices", [])}
    out = []
    for sid in RUNWAY:
        s = by_id.get(sid, {})
        if s.get("status") == "merged" and (s.get("merge_confidence") or 0) < 5:
            out.append(sid)
    return out


def _run_sub_gate(name: str, script: str) -> SubGate:
    proc = subprocess.run(
        [sys.executable, str(ROOT / "scripts" / script), "--gate", "--format", "json"],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )
    return SubGate(
        name=name, script=script, passed=proc.returncode == 0, exit_code=proc.returncode
    )


def _cross_os_complete() -> bool:
    if not CROSS_OS_MATRIX.is_file():
        return False
    try:
        d = json.loads(CROSS_OS_MATRIX.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return False
    return d.get("cross_os_complete") is True and d.get("status") == "passed"


def _integrity_ok() -> bool:
    proc = subprocess.run(
        [
            sys.executable,
            str(ROOT / "scripts" / "garnet_evidence_integrity_status.py"),
            "--gate",
            "--format",
            "json",
        ],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )
    return proc.returncode == 0


def read_readiness(binary_strict: bool = True) -> ReleaseReadiness:
    ledger = json.loads(GOAL_FILE.read_text(encoding="utf-8"))
    runway = _runway(ledger)
    sub_gates = [_run_sub_gate(n, s) for (n, s) in SUB_GATES]

    runway_complete = all(s.merged for s in runway)
    sub_gates_pass = all(g.passed for g in sub_gates)
    cross_os = _cross_os_complete()
    integrity = _integrity_ok()

    # Binary-strict (default): READY requires the real cross-OS matrix + integrity.
    # Lenient: the cross-OS-matrix hard requirement is dropped (python-only CI).
    release_ready = runway_complete and sub_gates_pass and integrity and (
        cross_os if binary_strict else True
    )

    return ReleaseReadiness(
        schema="garnet.v0_8_1_release_readiness/v1",
        runway=runway,
        sub_gates=sub_gates,
        runway_complete=runway_complete,
        sub_gates_pass=sub_gates_pass,
        cross_os_complete=cross_os,
        integrity_ok=integrity,
        binary_strict=binary_strict,
        release_ready=release_ready,
        low_confidence_slices=_low_confidence(ledger),
        deferred_for_v0_8_1=DEFERRED_FOR_V0_8_1,
        honesty_anchors=HONESTY_ANCHORS,
        tag_note=(
            "This gate does NOT cut or push a tag. READY TO CUT is a recommendation; "
            "the v0.8.1 cut (s120) and the release tag are reserved to Jon."
        ),
    )


def render_markdown(r: ReleaseReadiness) -> str:
    merged = sum(1 for s in r.runway if s.merged)
    lines = [
        "# Garnet v0.8.1 release-readiness gate (S119)",
        "",
        f"_Schema {r.schema}._",
        "",
        f"**Verdict: {'READY TO CUT (pending Jon) ✅' if r.release_ready else 'NOT READY ❌'}**",
        f"_(mode: {'binary-strict' if r.binary_strict else 'lenient'})_",
        "",
        f"- runway s91–s118 (+s105b) merged: {merged}/{len(r.runway)} "
        f"→ {'complete' if r.runway_complete else 'INCOMPLETE'}",
        f"- anti-rot sub-gates passing: "
        f"{sum(1 for g in r.sub_gates if g.passed)}/{len(r.sub_gates)}",
        f"- cross-OS trap-parity matrix complete (binary-backed): "
        f"{'yes' if r.cross_os_complete else 'NO'}",
        f"- evidence-integrity verified: {'yes' if r.integrity_ok else 'NO'}",
        "",
        "## Sub-gates",
    ]
    for g in r.sub_gates:
        lines.append(f"- {'✅' if g.passed else '❌'} {g.name} (`{g.script}`)")
    if r.low_confidence_slices:
        lines += [
            "",
            f"## Merged-but-sub-5-confidence (surfaced honestly): "
            f"{', '.join(r.low_confidence_slices)}",
        ]
    lines += ["", "## Deferred / out of scope for v0.8.1"]
    lines += [f"- {d}" for d in r.deferred_for_v0_8_1]
    lines += ["", "## Honesty anchors (verbatim — do not soften)"]
    lines += [f"- {a}" for a in r.honesty_anchors]
    lines += ["", f"> {r.tag_note}", ""]
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument(
        "--lenient",
        action="store_true",
        help="drop the cross-OS-matrix hard requirement (python-only CI job); "
        "default is binary-strict (the S86 lesson: READY must carry real results).",
    )
    parser.add_argument(
        "--gate",
        action="store_true",
        help="exit non-zero unless the v0.8.1 runway is merged, every anti-rot "
        "sub-gate passes, evidence-integrity verifies, and (binary-strict) the "
        "cross-OS trap-parity matrix is complete.",
    )
    args = parser.parse_args(list(argv) if argv is not None else None)

    r = read_readiness(binary_strict=not args.lenient)
    print(render_markdown(r) if args.format == "md" else json.dumps(asdict(r), indent=2))

    if args.gate and not r.release_ready:
        print("v0.8.1 release-readiness gate: NOT READY (see sub-gates).", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
