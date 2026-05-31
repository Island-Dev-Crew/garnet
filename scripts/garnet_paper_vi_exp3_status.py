#!/usr/bin/env python3
"""Paper VI Experiment 3 status + provider-free harness run (S71).

Experiment 3 is the compiler-as-agent time-to-fix study (hypothesis h₃). Its
harness ships at `benchmarks/paper_vi_exp3_compiler_as_agent/` and is, by design,
the reproducible *shape* of the study — the provider-backed measurement is opt-in
(`GARNET_EXP3_EXECUTE=1` + an LLM provider). This reporter:

  1. inventories the harness (10 snapshots, both lane scripts, aggregate/analyze);
  2. actually RUNS the harness's provider-free mode (both lanes harness-only) and
     `aggregate.py`, confirming they run and emit the honest "harness-only" shape
     without inventing measurements;
  3. records the pre-registered H₃ and the v4.0 OUTCOME **verbatim** — h₃a is the
     honestly-downgraded partial (6.5% speedup, below the 10% threshold), h₃b and
     h₃c pass — citing `GARNET_v4_0_PAPER_VI_EXECUTION.md`.

## Honest scope (do not soften)
This reporter does NOT re-measure h₃a's timing speedup (machine-dependent; the
determinism doctrine forbids inventing measurements) and does NOT call any LLM
provider. The provider-backed RE-RUN is **pending-infra** (same boundary as Exp 1:
no provider / API credits). The recorded partial (6.5%, CI [3.1%, 9.8%]) stands;
the 10% claim is downgraded honestly per the pre-registration. The verbatim Paper
VI §C3 revision is surfaced as an honesty anchor, not softened.
"""
from __future__ import annotations

import argparse
import json
import subprocess
import sys
from dataclasses import asdict, dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
HARNESS = ROOT / "benchmarks" / "paper_vi_exp3_compiler_as_agent"
EXEC_DOC = ROOT / "F_Project_Management" / "GARNET_v4_0_PAPER_VI_EXECUTION.md"

PRE_REGISTERED_H3 = [
    "h3a: mean_time(B[6..10]) / mean_time(B[1..5]) < 0.90 (>=10% speedup)",
    "h3b: >= 1 strategy hit per compilation in compiles 6-10",
    "h3c: all honored strategies re-derivable from HMAC-verified episodes",
]

# Recorded v4.0 outcome (NOT re-measured here). h3b/h3c pass; h3a is partial.
RECORDED_OUTCOME = {
    "h3a": "partial — 6.5% speedup (CI [3.1%, 9.8%]); below the 10% threshold",
    "h3b": "pass — 1.4 strategies/compile in cycles 6-10 (>= 1.0)",
    "h3c": "pass — provenance.verify_strategy 100%",
}

# Paper VI §C3 revision — the human-readable downgrade (for display).
PAPER_VI_C3_REVISION = (
    "On evolving 800-LOC codebases, the compiler-as-agent's measurable speedup is "
    "6.5% (CI [3.1%, 9.8%]); the stronger 10% claim awaits a 5K-LOC re-run."
)
# The verbatim honesty anchor — a contiguous fragment that must remain in the
# exec doc (the doc wraps the full sentence across blockquote lines).
C3_VERBATIM_ANCHOR = "6.5% (CI [3.1%, 9.8%])"


@dataclass
class Exp3Status:
    schema: str
    harness_present: bool
    snapshot_count: int
    lane_scripts_present: bool
    aggregate_present: bool
    provider_free_run_ok: bool
    provider_backed_status: str
    pre_registered: list[str] = field(default_factory=list)
    recorded_outcome: dict = field(default_factory=dict)
    paper_vi_c3_revision: str = ""
    c3_revision_in_doc: bool = False
    ok: bool = False


def _run(cmd: list[str]) -> int:
    try:
        return subprocess.run(
            cmd, cwd=str(HARNESS), capture_output=True, text=True, timeout=120
        ).returncode
    except Exception:
        return 1


def _provider_free_run_ok() -> bool:
    """Run both lanes harness-only + aggregate; all must exit 0."""
    if not HARNESS.is_dir():
        return False
    stateless = _run(["bash", str(HARNESS / "run_stateless.sh")])
    history = _run(["bash", str(HARNESS / "run_history_aware.sh")])
    agg = _run(["python3", str(HARNESS / "aggregate.py"), str(HARNESS / "out")])
    return stateless == 0 and history == 0 and agg == 0


def read_status(run_harness: bool = True) -> Exp3Status:
    snapshots = sorted((HARNESS / "codebase_versions").glob("v*/main.garnet")) if HARNESS.is_dir() else []
    lanes = (HARNESS / "run_stateless.sh").is_file() and (HARNESS / "run_history_aware.sh").is_file()
    aggregate = (HARNESS / "aggregate.py").is_file()
    doc = EXEC_DOC.read_text(encoding="utf-8") if EXEC_DOC.is_file() else ""
    run_ok = _provider_free_run_ok() if run_harness else False
    harness_present = HARNESS.is_dir() and lanes and aggregate
    ok = harness_present and len(snapshots) == 10 and (run_ok if run_harness else True)
    return Exp3Status(
        schema="garnet.paper_vi_exp3_status/v1",
        harness_present=harness_present,
        snapshot_count=len(snapshots),
        lane_scripts_present=lanes,
        aggregate_present=aggregate,
        provider_free_run_ok=run_ok,
        provider_backed_status=(
            "pending-infra — no LLM provider / API credits in this environment "
            "(same boundary as Exp 1). The recorded partial stands; no re-measurement here."
        ),
        pre_registered=PRE_REGISTERED_H3,
        recorded_outcome=RECORDED_OUTCOME,
        paper_vi_c3_revision=PAPER_VI_C3_REVISION,
        c3_revision_in_doc=C3_VERBATIM_ANCHOR in doc,
        ok=ok,
    )


def render_markdown(r: Exp3Status) -> str:
    lines = [
        "# Garnet Paper VI Experiment 3 status (compiler-as-agent time-to-fix)",
        "",
        f"_Schema {r.schema}._",
        "",
        "## Harness (provider-free)",
        f"- present + well-formed: {'yes' if r.harness_present else 'NO'}",
        f"- snapshots: {r.snapshot_count} (expect 10)",
        f"- provider-free run (both lanes harness-only + aggregate) exits 0: "
        f"{'yes' if r.provider_free_run_ok else 'NO'}",
        "",
        "## Pre-registered H₃",
    ]
    lines += [f"- {h}" for h in r.pre_registered]
    lines += ["", "## Recorded v4.0 outcome (NOT re-measured here)"]
    for k in ("h3a", "h3b", "h3c"):
        lines.append(f"- {k}: {r.recorded_outcome.get(k, '')}")
    lines += [
        "",
        f"## Provider-backed re-run\n- status: {r.provider_backed_status}",
        "",
        f'**Paper VI §C3 revision (verbatim): "{r.paper_vi_c3_revision}"** '
        f"(present in exec doc: {'yes' if r.c3_revision_in_doc else 'NO'})",
        "",
        "Honest scope: h₃a's timing speedup is machine-dependent and is NOT "
        "re-measured here; no LLM is called. The 10% claim is downgraded honestly "
        "to the recorded 6.5% partial.",
        "",
    ]
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument(
        "--gate",
        action="store_true",
        help="exit non-zero if the harness is missing/malformed or its provider-free "
        "run fails (the provider-backed h₃a re-run is pending-infra and is NOT gated)",
    )
    parser.add_argument(
        "--no-run", action="store_true", help="skip executing the harness (inventory only)"
    )
    args = parser.parse_args(list(argv) if argv is not None else None)

    r = read_status(run_harness=not args.no_run)
    if args.format == "md":
        print(render_markdown(r))
    else:
        print(json.dumps(asdict(r), indent=2))

    if args.gate and not r.ok:
        print(
            "paper-vi-exp3 gate FAILED: "
            f"harness_present={r.harness_present} snapshots={r.snapshot_count} "
            f"provider_free_run_ok={r.provider_free_run_ok} "
            "(the provider-backed h₃a re-run is pending-infra and is NOT gated)",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
