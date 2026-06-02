#!/usr/bin/env python3
"""`garnet agent-loop` — agent-acceptance loop status (S102, Stage U).

S102 adds the real agent-acceptance loop: a simulated agent proposes a Garnet
change and the loop ACCEPTS it ONLY on enforced evidence —
  1. `diff-caps` (S37): the declared capability surface must not widen (a widening
     is a true gate FAILURE — refused, never run, never sealed; Rule 2);
  2. the enforced kernel (S99 `@max_depth` + S100 `@caps` traps): the proposal must
     run without tripping an enforced ceiling;
  3. `seal` (S38): an accepted proposal is attested, recording the autonomous
     acceptance + agent/model/gate-version provenance (S65/S66; Rule 3).

This static anti-regression gate asserts the harness, its honesty anchor, and the
accept/reject/trap tests stay in place. (It is intentionally NOT wired into CI:
adding a CI gate is a gate-definition change and would require a human merge per
the gate-independence rule; the loop's CI-enforced proof is the
`garnet-cli/tests/agent_loop.rs` integration suite under `cargo test --workspace`.)

## Honest scope (do not soften)
Acceptance rests ONLY on the two ENFORCED ceilings — `@caps` host-authority +
`@max_depth` recursion. The verdict is "accepted on capability+depth evidence",
never "fully bounded"/"sandboxed"/"safe". `@bounded` (Wasmtime fuel), memory, time,
`@mailbox`, and OS-level sandbox remain declared-not-enforced. The agent is
simulated/scripted (the proposal is an on-disk file), not a live LLM (S94).
"""
from __future__ import annotations

import argparse
import json
import sys
from dataclasses import asdict, dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
HARNESS = ROOT / "garnet-cli" / "src" / "cmd" / "agent_loop.rs"
DISPATCH = ROOT / "garnet-cli" / "src" / "bin" / "garnet.rs"
TEST = ROOT / "garnet-cli" / "tests" / "agent_loop.rs"


@dataclass
class AgentLoopStatus:
    schema: str
    harness_present: bool
    three_stage_gate: bool
    rule2_widening_refused: bool
    rule3_provenance_recorded: bool
    honesty_anchor_present: bool
    dispatched: bool
    accept_reject_tests_present: bool
    ok: bool = False


def _read(p: Path) -> str:
    return p.read_text(encoding="utf-8") if p.is_file() else ""


def read_status() -> AgentLoopStatus:
    h = _read(HARNESS)
    d = _read(DISPATCH)
    t = _read(TEST)
    harness_present = bool(h)
    # The three gated stages, orchestrating the real subcommands.
    three_stage = (
        '"diff-caps"' in h and '"run"' in h and '"seal"' in h and "current_exe" in h
    )
    # Rule 2: a capability widening is a hard refusal before run/seal.
    rule2 = "REJECTED at stage diff-caps" in h and "ExitCode::from(1)" in h
    # Rule 3: the autonomous acceptance + gate version are stamped into the seal.
    rule3 = (
        "autonomous=true" in h
        and "tool=garnet-agent-loop" in h
        and "gate_version" in h
    )
    honesty = "ACCEPTED on capability+depth evidence" in h and "declared-not-enforced" in h
    dispatched = '"agent-loop" => cmd::agent_loop::run' in d
    tests_present = (
        "accept_path_passes_gate_runs_and_seals" in t
        and "reject_path_widening_hardfails_and_is_refused" in t
        and "enforced_kernel_traps_overceiling_proposal" in t
    )
    ok = (
        harness_present
        and three_stage
        and rule2
        and rule3
        and honesty
        and dispatched
        and tests_present
    )
    return AgentLoopStatus(
        schema="garnet.agent_loop/v1",
        harness_present=harness_present,
        three_stage_gate=three_stage,
        rule2_widening_refused=rule2,
        rule3_provenance_recorded=rule3,
        honesty_anchor_present=honesty,
        dispatched=dispatched,
        accept_reject_tests_present=tests_present,
        ok=ok,
    )


def render_markdown(r: AgentLoopStatus) -> str:
    return "\n".join([
        "# Garnet agent-acceptance loop status (S102)",
        "",
        f"_Schema {r.schema}._",
        "",
        f"- harness present (`garnet agent-loop`): {'yes' if r.harness_present else 'NO'}",
        f"- three gated stages (diff-caps -> enforced run -> seal): "
        f"{'yes' if r.three_stage_gate else 'NO'}",
        f"- Rule 2 — a capability widening is REFUSED (no run, no seal): "
        f"{'yes' if r.rule2_widening_refused else 'NO'}",
        f"- Rule 3 — autonomous acceptance + gate version recorded in the seal: "
        f"{'yes' if r.rule3_provenance_recorded else 'NO'}",
        f"- honesty anchor (\"accepted on capability+depth evidence\"): "
        f"{'yes' if r.honesty_anchor_present else 'NO'}",
        f"- dispatched + accept/reject/trap tests present: "
        f"{'yes' if r.dispatched and r.accept_reject_tests_present else 'NO'}",
        "",
        "Acceptance rests ONLY on the enforced ceilings (`@caps` + `@max_depth`); "
        "`@bounded`/memory/time/`@mailbox`/OS-sandbox remain declared-not-enforced. "
        "The agent is simulated/scripted, not a live LLM (S94).",
        "",
    ])


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument(
        "--gate",
        action="store_true",
        help="exit non-zero unless the agent-loop harness, its 3-stage gate, the "
        "Rule-2 refusal, the Rule-3 provenance, the honesty anchor, and the "
        "accept/reject/trap tests are all present.",
    )
    args = parser.parse_args(list(argv) if argv is not None else None)

    r = read_status()
    print(render_markdown(r) if args.format == "md" else json.dumps(asdict(r), indent=2))

    if args.gate and not r.ok:
        print(f"agent-loop gate FAILED: {asdict(r)}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
