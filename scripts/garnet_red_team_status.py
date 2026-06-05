#!/usr/bin/env python3
"""Kernel red-team status (S114).

S114 red-teamed the enforced trust kernel: six attackers ran real attacks; a
skeptical referee classified each as HELD / HOLE / DECLARED-NOT-ENFORCED. One
HIGH-severity hole was found (impl-method capability-surface blindness) and FIXED;
two LOW stub-scoped holes were recorded for follow-up. This static gate asserts the
fix is in place (with regression tests) and the honest report stays recorded.

## Honest scope (do not soften)
The HIGH impl-method surface hole is fixed (capability_surface recurses into
Item::Impl + nested modules). Two LOW holes (caps-log tail forgery; seal subject
digest is capability-blind) are recorded as open/mitigated within the honest stub
scope. @bounded/memory/time/@mailbox and macOS/Windows OS-sandbox stay named-deferred.
"""
from __future__ import annotations

import argparse
import json
import sys
from dataclasses import asdict, dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "C_Language_Specification" / "GARNET_RED_TEAM.md"
SURFACE = ROOT / "garnet-check-v0.3" / "src" / "capability_surface.rs"


@dataclass
class RedTeamStatus:
    schema: str
    report_present: bool
    high_hole_fixed: bool
    regression_tests_present: bool
    low_holes_recorded: bool
    held_and_deferred_recorded: bool
    honesty_anchor_present: bool
    ok: bool = False


def _read(p: Path) -> str:
    return p.read_text(encoding="utf-8") if p.is_file() else ""


def read_status() -> RedTeamStatus:
    doc = _read(REPORT)
    surf = _read(SURFACE)
    report_present = bool(doc)
    # The HIGH fix is in place: the surface recurses into impl methods + modules.
    high_fixed = (
        "Item::Impl(block)" in surf
        and "block.methods" in surf
        and "Item::Module(m) => collect_cap_fns" in surf
    )
    regression = (
        "impl_method_caps_are_in_the_surface" in surf
        and "nested_module_fn_caps_are_in_the_surface" in surf
    )
    low_recorded = (
        "caps-log" in doc
        and "forged TAIL" in doc
        and "subject.digest" in doc
        and "capability-blind" in doc
    )
    held_deferred = "HELD" in doc and "DECLARED-NOT-ENFORCED" in doc
    honesty = (
        "named-deferred" in doc
        and "production / 1.0 claim" in doc
        and "not a \"nothing broke\" claim" in doc
    )
    ok = (
        report_present
        and high_fixed
        and regression
        and low_recorded
        and held_deferred
        and honesty
    )
    return RedTeamStatus(
        schema="garnet.red_team/v1",
        report_present=report_present,
        high_hole_fixed=high_fixed,
        regression_tests_present=regression,
        low_holes_recorded=low_recorded,
        held_and_deferred_recorded=held_deferred,
        honesty_anchor_present=honesty,
        ok=ok,
    )


def render_markdown(r: RedTeamStatus) -> str:
    return "\n".join([
        "# Garnet kernel red-team status (S114)",
        "",
        f"_Schema {r.schema}._",
        "",
        f"- red-team report present: {'yes' if r.report_present else 'NO'}",
        f"- HIGH hole (impl-method surface blindness) FIXED: "
        f"{'yes' if r.high_hole_fixed else 'NO'}",
        f"- regression tests present: {'yes' if r.regression_tests_present else 'NO'}",
        f"- LOW holes recorded (caps-log tail; seal subject digest): "
        f"{'yes' if r.low_holes_recorded else 'NO'}",
        f"- HELD + DECLARED-NOT-ENFORCED recorded: "
        f"{'yes' if r.held_and_deferred_recorded else 'NO'}",
        f"- honesty anchor: {'yes' if r.honesty_anchor_present else 'NO'}",
        "",
        "One HIGH enforced-ceiling hole found + fixed; two LOW stub-scoped holes "
        "recorded. The enforced kernel withstood laundering/bypass/forgery on the "
        "other vectors. Named-deferred ceilings unchanged; v0.8.1 research-grade.",
        "",
    ])


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument(
        "--gate",
        action="store_true",
        help="exit non-zero unless the red-team report, the HIGH-hole fix + its "
        "regression tests, the recorded LOW holes, and the held/deferred record "
        "are all present.",
    )
    args = parser.parse_args(list(argv) if argv is not None else None)

    r = read_status()
    print(render_markdown(r) if args.format == "md" else json.dumps(asdict(r), indent=2))

    if args.gate and not r.ok:
        print(f"red-team gate FAILED: {asdict(r)}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
