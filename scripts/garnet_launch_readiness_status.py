#!/usr/bin/env python3
"""Canonical machine-readable launch-readiness ledger (Truth Lock Task 3).

Aggregates the structured readiness reporters into one launch ledger with
explicit gate states. This reporter is machine authority ONLY for
reporter-derived inputs and the evidence-base validator. S114 acceptance,
the not-yet-reported minimum shelf, and launch fire are external/manual
gates: this script names them but can never grade them.

States:
  pass             — every reporter-derived input for the gate is green
  blocked          — a reporter-derived input failed; blockers name it
  partial          — honest middle state (static playground only)
  remaining        — work not yet built (live WASM playground / W-PLAY)
  manual-deferred  — no reporter exists; explicit manual fence (shelf)
  external-pending — a human decision outside this reporter, not yet recorded (S114)
  accepted-scoped  — a human decision recorded in an external acceptance
                     artifact and read (never graded) here (S114)
  pending-human    — human-produced artifact not yet delivered (promo)
  jon-only         — never autonomous (launch fire)

S114 acceptance is read from F_Project_Management/LAUNCH/S114_ACCEPTANCE.json
when present and valid; absent or malformed, the S114 gate stays
external-pending. The promo ledger line is read from the committed snapshot
F_Project_Management/LAUNCH/PROMO_EVIDENCE_SNAPSHOT.json so regeneration is
machine-independent (the live promo probe is still invoked; the snapshot only
pins the rendered value).

`--gate` exits 1 until every launch-critical gate passes. It is
intentionally NOT wired into CI in the Truth Lock slice.
"""
from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from dataclasses import asdict, dataclass, field, replace
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import garnet_evidence_integrity_status  # noqa: E402
import garnet_mit_readiness_status  # noqa: E402
import garnet_native_debian_cli_install_status  # noqa: E402
import garnet_native_linux_studio_status  # noqa: E402
import garnet_playground_readiness  # noqa: E402
import garnet_promo_video_status  # noqa: E402
import garnet_red_team_status  # noqa: E402
import garnet_seccomp_apply_status  # noqa: E402
import garnet_stdlib_layer_gate  # noqa: E402
import garnet_v0_8_1_release_readiness  # noqa: E402
import garnet_wasm_readiness  # noqa: E402

SCHEMA = "garnet.launch_readiness/v1"
REPO_ROOT = Path(__file__).resolve().parents[1]
TRUTH_JSON = REPO_ROOT / "docs" / "truth.json"
S114_ACCEPTANCE_JSON = (
    REPO_ROOT / "F_Project_Management" / "LAUNCH" / "S114_ACCEPTANCE.json"
)
PROMO_SNAPSHOT_JSON = (
    REPO_ROOT / "F_Project_Management" / "LAUNCH" / "PROMO_EVIDENCE_SNAPSHOT.json"
)

LAUNCH_CRITICAL_GATES = (
    "foundation_integrity",
    "native_linux",
    "s114_acceptance",
    "static_playground",
    "live_wasm_playground",
    "minimum_sealed_shelf",
)

DEFERRED_FENCES = [
    "@bounded (Wasmtime fuel) — declared, not enforced",
    "memory limits — declared, not enforced",
    "time limits — declared, not enforced",
    "@mailbox — declared, not enforced",
    "macOS/Windows OS-sandbox application — seccomp applies on Linux only",
    "Core Ring Tier 1 shelf — post-Truth-Lock workstream (Minimum Shelf)",
    "MCP tool-server library — post-Truth-Lock workstream (Minimum Shelf)",
]

JON_ONLY_ACTIONS = [
    "push a git tag",
    "cut or release a version",
    "record the S114 acceptance decision",
    "change CI/release policy or any gate a PR merges under",
    "RB-6 backend decision",
    "RB-8 root-reorg cut",
    "fire the launch/marketing wave",
]


@dataclass(frozen=True)
class LaunchGate:
    id: str
    label: str
    state: str
    evidence: list[str]
    blockers: list[str]


@dataclass(frozen=True)
class LaunchReadinessStatus:
    schema: str
    source: str
    evidence_base: str
    evidence_base_status: str
    release_grade: str
    recommendation: str
    launch_ready: bool
    gates: list[LaunchGate]
    deferred: list[str]
    jon_only: list[str]


@dataclass(frozen=True)
class Dependencies:
    release: object
    red_team: object
    integrity: object
    seccomp: object
    native_cli: object
    native_studio: object
    playground: object
    wasm: object
    stdlib_meets_count_gate: bool
    stdlib_total: int
    stdlib_explicit_stability_percent: float
    promo: object
    mit_overall_status: str
    mit_completion_percent: float
    evidence_base: str
    evidence_base_status: str


def _git(*args: str) -> subprocess.CompletedProcess:
    return subprocess.run(
        ["git", *args], capture_output=True, text=True, cwd=REPO_ROOT
    )


def validate_evidence_base(value: object) -> tuple[str, str]:
    """Validate `workspace_tests.measured_at_commit` from docs/truth.json.

    The truth generator intentionally records `git rev-parse --short HEAD`.
    Accept 7-40 lowercase hex characters only when the value resolves to a
    commit reachable from HEAD. Reject any `-dirty` suffix. A dirty,
    malformed, missing, or unreachable value is never presented as the
    canonical launch evidence base: it returns "unmeasured" with the literal
    retained for diagnosis.
    """
    literal = "" if value is None else str(value)
    if not literal:
        return literal, "unmeasured"
    if literal.endswith("-dirty"):
        return literal, "unmeasured"
    if not re.fullmatch(r"[0-9a-f]{7,40}", literal):
        return literal, "unmeasured"
    resolved = _git("rev-parse", f"{literal}^{{commit}}")
    if resolved.returncode != 0:
        return literal, "unmeasured"
    reachable = _git("merge-base", "--is-ancestor", resolved.stdout.strip(), "HEAD")
    if reachable.returncode != 0:
        return literal, "unmeasured"
    return literal, "measured"


def _read_truth_evidence_base() -> tuple[str, str]:
    try:
        truth = json.loads(TRUTH_JSON.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return "", "unmeasured"
    value = truth.get("workspace_tests", {}).get("measured_at_commit")
    return validate_evidence_base(value)


def read_s114_acceptance() -> dict | None:
    """Read Jon's recorded S114 acceptance decision, if present and valid.

    The reporter never grades S114: acceptance is a human decision recorded
    outside this script. When a valid acceptance artifact is present the S114
    gate reflects it as `accepted-scoped`; otherwise the gate stays
    `external-pending`. A missing, malformed, wrong-schema, wrong-state, or
    scopeless artifact is treated as absent (fail-closed to external-pending)
    so a broken file can never silently accept S114.
    """
    try:
        data = json.loads(S114_ACCEPTANCE_JSON.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return None
    if not isinstance(data, dict):
        return None
    if data.get("schema") != "garnet.s114_acceptance/v1":
        return None
    if data.get("state") != "accepted-scoped":
        return None
    if not str(data.get("scope", "")).strip():
        return None
    return data


def read_promo_snapshot() -> tuple[str, float] | None:
    """Read the committed canonical promo snapshot, if present and valid.

    Returns (status, completion_percent). The live promo probe reads the
    machine-local ~/Desktop/dogfood evidence tree, which is only complete on
    the evidence machine; reading a committed snapshot for the ledger line
    keeps regeneration machine-independent. A missing or malformed snapshot
    returns None and the reporter falls back to the live probe value.
    """
    try:
        data = json.loads(PROMO_SNAPSHOT_JSON.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return None
    if not isinstance(data, dict):
        return None
    if data.get("schema") != "garnet.promo_evidence_snapshot/v1":
        return None
    status = data.get("status")
    percent = data.get("completion_percent")
    if not isinstance(status, str) or not status.strip():
        return None
    if not isinstance(percent, (int, float)) or isinstance(percent, bool):
        return None
    return status, float(percent)


def collect_dependencies() -> Dependencies:
    release = garnet_v0_8_1_release_readiness.read_readiness(binary_strict=True)
    red_team = garnet_red_team_status.read_status()
    integrity = garnet_evidence_integrity_status.read_status()
    seccomp = garnet_seccomp_apply_status.read_status()
    native_cli = garnet_native_debian_cli_install_status.evaluate()
    native_studio = garnet_native_linux_studio_status.evaluate()
    playground = garnet_playground_readiness.read_readiness()
    wasm = garnet_wasm_readiness.read_readiness()
    stdlib = garnet_stdlib_layer_gate.read_status()
    promo = garnet_promo_video_status.read_status()
    # The live promo probe reads the machine-local ~/Desktop/dogfood tree.
    # Pin the rendered ledger line to the committed canonical snapshot so
    # regeneration is machine-independent; the live probe is still invoked
    # above (kept as the evidence-machine source of truth).
    promo_snapshot = read_promo_snapshot()
    if promo_snapshot is not None:
        promo = replace(
            promo, status=promo_snapshot[0], completion_percent=promo_snapshot[1]
        )
    mit = garnet_mit_readiness_status.read_status()
    evidence_base, evidence_base_status = _read_truth_evidence_base()
    return Dependencies(
        release=release,
        red_team=red_team,
        integrity=integrity,
        seccomp=seccomp,
        native_cli=native_cli,
        native_studio=native_studio,
        playground=playground,
        wasm=wasm,
        stdlib_meets_count_gate=bool(stdlib.meets_count_gate),
        stdlib_total=stdlib.total,
        stdlib_explicit_stability_percent=stdlib.explicit_stability_percent,
        promo=promo,
        mit_overall_status=mit.overall_status,
        mit_completion_percent=mit.completion_percent,
        evidence_base=evidence_base,
        evidence_base_status=evidence_base_status,
    )


def _foundation_gate(deps: Dependencies) -> LaunchGate:
    evidence: list[str] = []
    blockers: list[str] = []
    if deps.release.release_ready:
        evidence.append("v0.8.1 release readiness reporter green (binary-strict)")
    else:
        blockers.append("release readiness reporter is not green (binary-strict)")
    if deps.red_team.ok:
        evidence.append("red-team static contract green (report, HIGH fix, regressions)")
    else:
        blockers.append("red-team static contract failed")
    if deps.integrity.ok:
        evidence.append(
            f"evidence-integrity bundles ok ({deps.integrity.bundles_ok}/{deps.integrity.bundles_total})"
        )
    else:
        blockers.append("evidence-integrity bundle verification failed")
    if deps.evidence_base_status == "measured":
        evidence.append(
            f"workspace tests measured at reachable commit `{deps.evidence_base}`"
        )
    else:
        blockers.append(
            f"evidence base `{deps.evidence_base}` is {deps.evidence_base_status}; "
            "re-measure with `cargo run -p xtask -- truth --with-tests` on a clean tree"
        )
    if deps.mit_overall_status == "regressed" or deps.mit_completion_percent < 50.0:
        blockers.append(
            f"MIT productization lane regressed "
            f"({deps.mit_overall_status}, {deps.mit_completion_percent:.1f}%)"
        )
    else:
        evidence.append(
            f"MIT productization lane {deps.mit_overall_status} "
            f"({deps.mit_completion_percent:.1f}%)"
        )
    state = "pass" if not blockers else "blocked"
    return LaunchGate(
        id="foundation_integrity",
        label="Foundation integrity (release + red-team + evidence + measured base)",
        state=state,
        evidence=evidence,
        blockers=blockers,
    )


def _native_linux_gate(deps: Dependencies) -> LaunchGate:
    evidence: list[str] = []
    blockers: list[str] = []
    if deps.native_cli.ok:
        evidence.append("native Debian CLI clean-install proof verified")
    else:
        blockers.append("native Debian CLI install proof failed")
    if deps.native_studio.ok:
        evidence.append("native Linux Studio build+install+launch proof verified")
    else:
        blockers.append("native Linux Studio proof failed")
    if deps.seccomp.ok:
        evidence.append("seccomp-apply proof deterministic and policy-driven")
    else:
        blockers.append("seccomp-apply proof failed")
    state = "pass" if not blockers else "blocked"
    return LaunchGate(
        id="native_linux",
        label="Native Linux lane (CLI install + Studio + seccomp)",
        state=state,
        evidence=evidence,
        blockers=blockers,
    )


def _s114_gate(deps: Dependencies) -> LaunchGate:
    del deps
    acceptance = read_s114_acceptance()
    if acceptance is None:
        return LaunchGate(
            id="s114_acceptance",
            label="S114 independent re-verification acceptance",
            state="external-pending",
            evidence=[
                "F_Project_Management/GARNET_S114_INDEPENDENT_VERIFICATION_DOSSIER.html",
                "F_Project_Management/W_TRUST/S114_INDEPENDENT_REVERIFICATION_PACKAGE_2026-06-14.md",
            ],
            blockers=[
                "Jon has not recorded an S114 acceptance decision; "
                "this reporter proves the static contract only and cannot grade acceptance"
            ],
        )
    scope = str(acceptance["scope"]).strip()
    decided_by = str(acceptance.get("decided_by", "unknown")).strip()
    decision_date = str(acceptance.get("decision_date", "unknown")).strip()
    # accepted-scoped: the gate is satisfied. Scope limits are surfaced as
    # evidence (tracked hardening debt), never as blockers — an accepted gate
    # is not a blocked gate. Independence is NOT relabelled here.
    evidence = [
        f"S114 accepted (scoped) by {decided_by} on {decision_date}: {scope}",
        "recorded in F_Project_Management/LAUNCH/S114_ACCEPTANCE.json "
        "(read, not graded, by this reporter)",
        "not an independence relabel: S114 verdict language is unchanged "
        "(independently-re-verified-with-fixes)",
    ]
    closure = acceptance.get("post_acceptance_closure", {})
    current_limits = (
        closure.get("current_scope_limits", [])
        if isinstance(closure, dict)
        else []
    )
    if isinstance(closure, dict) and closure.get("state"):
        evidence.append(f"post-acceptance closure: {closure['state']}")
    evidence.extend(
        f"current scope limit (tracked): {item}"
        for item in current_limits
        if isinstance(item, str) and item.strip()
    )
    return LaunchGate(
        id="s114_acceptance",
        label="S114 independent re-verification acceptance",
        state="accepted-scoped",
        evidence=evidence,
        blockers=[],
    )


def _static_playground_gate(deps: Dependencies) -> LaunchGate:
    if deps.playground.ok:
        return LaunchGate(
            id="static_playground",
            label="Static playground gallery (honest, recorded outputs)",
            state="partial",
            evidence=[
                f"{deps.playground.example_count} recorded examples, honesty markers present"
            ],
            blockers=["static gallery only; live execution is the W-PLAY workstream"],
        )
    return LaunchGate(
        id="static_playground",
        label="Static playground gallery (honest, recorded outputs)",
        state="blocked",
        evidence=[],
        blockers=["playground readiness reporter failed"]
        + list(deps.playground.missing_markers),
    )


def _live_wasm_gate(deps: Dependencies) -> LaunchGate:
    blockers = list(deps.wasm.blockers)
    evidence: list[str] = []
    if deps.wasm.wasm_build_passed:
        evidence.append(
            "WV-5 clean-Windows evidence: wasm32 + wasm-pack web/node builds passed"
        )
    if deps.wasm.node_execution_passed:
        evidence.append(
            "WV-5 real Node execution passed, including fail-closed authority smoke"
        )
    return LaunchGate(
        id="live_wasm_playground",
        label="Live WASM playground (W-PLAY, launch centerpiece)",
        state="remaining",
        evidence=evidence,
        blockers=blockers,
    )


def _shelf_gate(deps: Dependencies) -> LaunchGate:
    blockers = [
        "no reporter covers the shelf in Truth Lock; this is an explicit manual fence, "
        "never reporter-derived machine truth"
    ]
    if not deps.stdlib_meets_count_gate:
        blockers.append("stdlib layer gate below its count threshold")
    evidence = [
        f"stdlib registry: {deps.stdlib_total} primitives, "
        f"{deps.stdlib_explicit_stability_percent:.1f}% explicit stability"
    ]
    return LaunchGate(
        id="minimum_sealed_shelf",
        label="Minimum sealed shelf (Core Ring Tier 1 + MCP library)",
        state="manual-deferred",
        evidence=evidence,
        blockers=blockers,
    )


def _promo_gate(deps: Dependencies) -> LaunchGate:
    return LaunchGate(
        id="promo_video",
        label="Promo video (human-produced launch asset)",
        state="pending-human",
        evidence=[
            f"promo reporter status: {deps.promo.status} "
            f"({deps.promo.completion_percent:.1f}%)"
        ],
        blockers=["human render/QA decision outside this reporter"],
    )


def _launch_fire_gate(deps: Dependencies) -> LaunchGate:
    del deps
    return LaunchGate(
        id="launch_fire",
        label="Launch fire (marketing wave, tag, release)",
        state="jon-only",
        evidence=[],
        blockers=["FIRE/HOLD is Jon's alone; never autonomous"],
    )


def build_status(deps: Dependencies) -> LaunchReadinessStatus:
    gates = [
        _foundation_gate(deps),
        _native_linux_gate(deps),
        _s114_gate(deps),
        _static_playground_gate(deps),
        _live_wasm_gate(deps),
        _shelf_gate(deps),
        _promo_gate(deps),
        _launch_fire_gate(deps),
    ]
    by_id = {gate.id: gate for gate in gates}
    # A gate satisfies launch readiness when its state is "pass", except the
    # S114 governance gate, which is satisfied by a recorded scoped acceptance.
    # (This does not by itself flip launch_ready: the playground, live-WASM,
    # and shelf gates remain open.)
    satisfactory_states = {"s114_acceptance": ("pass", "accepted-scoped")}
    launch_ready = all(
        by_id[gate_id].state in satisfactory_states.get(gate_id, ("pass",))
        for gate_id in LAUNCH_CRITICAL_GATES
    )
    recommendation = "HELD-AT-LOCK" if launch_ready else "HOLD"
    release_grade = (
        "research-grade v0.x prototype; "
        f"release_ready={deps.release.release_ready} (binary-strict)"
    )
    return LaunchReadinessStatus(
        schema=SCHEMA,
        source=str(Path(__file__).resolve()),
        evidence_base=deps.evidence_base,
        evidence_base_status=deps.evidence_base_status,
        release_grade=release_grade,
        recommendation=recommendation,
        launch_ready=launch_ready,
        gates=gates,
        deferred=list(DEFERRED_FENCES),
        jon_only=list(JON_ONLY_ACTIONS),
    )


def read_status() -> LaunchReadinessStatus:
    return build_status(collect_dependencies())


def render_json(status: LaunchReadinessStatus) -> str:
    return json.dumps(asdict(status), indent=2) + "\n"


def render_human(status: LaunchReadinessStatus) -> str:
    lines = [
        "Garnet launch readiness",
        f"  schema:          {status.schema}",
        f"  evidence base:   {status.evidence_base or '<missing>'} ({status.evidence_base_status})",
        f"  release grade:   {status.release_grade}",
        f"  launch ready:    {status.launch_ready}",
        f"  recommendation:  {status.recommendation}",
        "",
        "Gates:",
    ]
    for gate in status.gates:
        lines.append(f"  [{gate.state:>16}] {gate.id} — {gate.label}")
        for item in gate.evidence:
            lines.append(f"                     + {item}")
        for item in gate.blockers:
            lines.append(f"                     - {item}")
    lines.append("")
    lines.append("Deferred fences:")
    lines.extend(f"  - {item}" for item in status.deferred)
    lines.append("Jon-only actions:")
    lines.extend(f"  - {item}" for item in status.jon_only)
    return "\n".join(lines) + "\n"


def render_markdown(status: LaunchReadinessStatus) -> str:
    lines = [
        "# Garnet Launch Readiness Ledger",
        "",
        "Rendered state — regenerate with:",
        "`python3 scripts/garnet_launch_readiness_status.py --format markdown`.",
        "Machine authority applies to reporter-derived rows and the",
        "evidence-base validator only; S114 acceptance, the minimum shelf,",
        "and launch fire are external/manual gates.",
        "",
        f"- Schema: `{status.schema}`",
        f"- Evidence-base commit: `{status.evidence_base or '<missing>'}` ({status.evidence_base_status})",
        f"- Release grade: {status.release_grade}",
        f"- Launch ready: **{status.launch_ready}**",
        f"- Recommendation: **{status.recommendation}**",
        "",
        "## Gate Ledger",
        "",
    ]
    for gate in status.gates:
        lines.append(f"### `{gate.id}` — {gate.label}")
        lines.append("")
        lines.append(f"State: **{gate.state}**")
        if gate.evidence:
            lines.append("")
            lines.append("Evidence:")
            lines.extend(f"- {item}" for item in gate.evidence)
        if gate.blockers:
            lines.append("")
            lines.append("Blockers:")
            lines.extend(f"- {item}" for item in gate.blockers)
        lines.append("")
    lines.append("## Deferred (named, not enforced)")
    lines.append("")
    lines.extend(f"- {item}" for item in status.deferred)
    lines.append("")
    lines.append("## Jon-only actions")
    lines.append("")
    lines.extend(f"- {item}" for item in status.jon_only)
    lines.append("")
    return "\n".join(lines)


def parse_args(argv: list[str] | None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--format",
        choices=("json", "human", "markdown"),
        default="human",
    )
    parser.add_argument(
        "--gate",
        action="store_true",
        help="exit 1 unless every launch-critical gate passes",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    status = read_status()
    if args.format == "json":
        sys.stdout.write(render_json(status))
    elif args.format == "markdown":
        sys.stdout.write(render_markdown(status))
    else:
        sys.stdout.write(render_human(status))
    if args.gate and not status.launch_ready:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
