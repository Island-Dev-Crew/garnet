#!/usr/bin/env python3
"""Status gate for the S96 linear/effect safe-mode seed."""
from __future__ import annotations

import argparse
import json
import subprocess
import sys
from dataclasses import asdict, dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
EFFECTS_RS = ROOT / "garnet-check-v0.3" / "src" / "effects.rs"
CHECK_LIB_RS = ROOT / "garnet-check-v0.3" / "src" / "lib.rs"
FOCUSED_TESTS = ROOT / "garnet-check-v0.3" / "tests" / "linear_effects.rs"
READINESS = ROOT / "scripts" / "garnet_mit_readiness_status.py"


@dataclass(frozen=True)
class LinearEffectStatus:
    schema: str
    effects_module_present: bool
    check_error_present: bool
    focused_tests_present: bool
    readiness_lane_present: bool
    focused_gate_ok: bool | None
    scope_summary: str
    ok: bool


def _text(path: Path) -> str:
    if not path.is_file():
        return ""
    return path.read_text(encoding="utf-8")


def _run_focused_gate() -> bool:
    completed = subprocess.run(
        ["cargo", "test", "-p", "garnet-check", "--test", "linear_effects", "--no-fail-fast"],
        cwd=ROOT,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=False,
    )
    return completed.returncode == 0


def read_status(run_gate: bool = False) -> LinearEffectStatus:
    effects_text = _text(EFFECTS_RS)
    lib_text = _text(CHECK_LIB_RS)
    tests_text = _text(FOCUSED_TESTS)
    readiness_text = _text(READINESS)

    effects_module_present = (
        "linear_effect_report" in effects_text
        and "LinearEffectViolation" in effects_text
        and "ownership-qualified parameter boundary" in effects_text
    )
    check_error_present = (
        "LinearEffect" in lib_text
        and "check.linear_effect" in lib_text
        and "effects::linear_effect_report" in lib_text
    )
    focused_tests_present = (
        "effectful_safe_helper_without_linear_param_is_fatal" in tests_text
        and "main_is_the_program_authority_boundary_for_s96" in tests_text
    )
    readiness_lane_present = "linear_effect_safe_mode_seed" in readiness_text
    focused_gate_ok = _run_focused_gate() if run_gate else None

    inventory_ok = all(
        [
            effects_module_present,
            check_error_present,
            focused_tests_present,
            readiness_lane_present,
        ]
    )
    ok = inventory_ok and (focused_gate_ok is not False)
    return LinearEffectStatus(
        schema="garnet.linear_effect_safe_mode_status/v1",
        effects_module_present=effects_module_present,
        check_error_present=check_error_present,
        focused_tests_present=focused_tests_present,
        readiness_lane_present=readiness_lane_present,
        focused_gate_ok=focused_gate_ok,
        scope_summary=(
            "S96 is a static, first-increment linear/effect safe-mode seed; "
            "it is not whole-language verification and does not claim VM/runtime "
            "or OS sandbox enforcement."
        ),
        ok=ok,
    )


def render_markdown(status: LinearEffectStatus) -> str:
    return "\n".join(
        [
            "# Garnet S96 linear/effect safe-mode status",
            "",
            f"_Schema {status.schema}._",
            "",
            f"- effects module: {'yes' if status.effects_module_present else 'NO'}",
            f"- fatal checker diagnostic: {'yes' if status.check_error_present else 'NO'}",
            f"- focused tests: {'yes' if status.focused_tests_present else 'NO'}",
            f"- readiness lane: {'yes' if status.readiness_lane_present else 'NO'}",
            f"- focused gate: {status.focused_gate_ok}",
            "",
            status.scope_summary,
            "No VM or OS sandbox enforcement is claimed by this slice.",
            "",
        ]
    )


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument("--gate", action="store_true")
    args = parser.parse_args(list(argv) if argv is not None else None)

    status = read_status(run_gate=args.gate)
    if args.format == "md":
        print(render_markdown(status))
    else:
        print(json.dumps(asdict(status), indent=2, sort_keys=True))

    if args.gate and not status.ok:
        print(f"S96 linear/effect safe-mode gate FAILED: {asdict(status)}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
