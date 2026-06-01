#!/usr/bin/env python3
"""S93 bounded-loop verifier status gate.

This reporter checks for the S93 static verifier surface without overstating
runtime enforcement. Passing means the repo has a checker-visible loop report,
a fatal `garnet check` diagnostic for uncheckable safe/@bounded loops, and
focused checker/CLI tests for the pass/reject cases.

Honest boundary: S93 is a static verifier only. It does not claim Wasmtime fuel,
VM loop enforcement, or OS sandbox enforcement.
"""
from __future__ import annotations

import argparse
import json
import sys
from dataclasses import asdict, dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CHECK_BOUNDS = ROOT / "garnet-check-v0.3" / "src" / "bounds.rs"
CHECK_LIB = ROOT / "garnet-check-v0.3" / "src" / "lib.rs"
CHECK_TEST = ROOT / "garnet-check-v0.3" / "tests" / "bounded_loops.rs"
CLI_TEST = ROOT / "garnet-cli" / "tests" / "bounded_loop_verifier.rs"


@dataclass
class BoundedLoopVerifierStatus:
    schema: str
    checker_exports_report: bool
    rejects_uncheckable_safe_loops: bool
    accepts_literal_bounds: bool
    accepts_counter_and_exit_bounds: bool
    cli_tests_present: bool
    no_wasmtime_boundary_stated: bool
    ok: bool = False


def _read(path: Path) -> str:
    return path.read_text(encoding="utf-8") if path.is_file() else ""


def read_status() -> BoundedLoopVerifierStatus:
    bounds = _read(CHECK_BOUNDS)
    lib = _read(CHECK_LIB)
    check_test = _read(CHECK_TEST)
    cli_test = _read(CLI_TEST)
    checker_exports_report = (
        "pub struct BoundedLoopReport" in bounds
        and "pub fn bounded_loop_report" in bounds
        and "pub use bounds::{bounded_functions, bounded_loop_report" in lib
    )
    rejects_uncheckable_safe_loops = (
        "CheckError::BoundedLoop" in lib
        and "check.bounded_loop" in lib
        and "safe_while_loop_with_nonliteral_condition_is_rejected" in check_test
        and "bounded_managed_function_enters_static_loop_scope" in check_test
    )
    accepts_literal_bounds = (
        "static_iter_bound" in bounds
        and "BinOp::Range" in bounds
        and "safe_for_loop_over_literal_range_is_static_bounded" in check_test
        and "safe_for_loop_over_literal_array_is_static_bounded" in check_test
    )
    accepts_counter_and_exit_bounds = (
        "literal_counter_while_bound" in bounds
        and "block_exits_before_next_turn" in bounds
        and "safe_counter_while_loop_with_literal_limit_is_static_bounded" in check_test
        and "safe_for_loop_with_immediate_return_is_static_bounded" in check_test
        and "safe_while_loop_with_immediate_return_is_static_bounded" in check_test
    )
    cli_tests_present = (
        "check_accepts_safe_literal_range_loop" in cli_test
        and "check_rejects_safe_uncheckable_while_loop" in cli_test
        and "json_diagnostic_uses_bounded_loop_code" in cli_test
    )
    no_wasmtime_boundary_stated = (
        "No Wasmtime fuel" in bounds
        and "No Wasmtime fuel" in cli_test
        and "runtime loop enforcement is claimed" in bounds
    )
    ok = all(
        [
            checker_exports_report,
            rejects_uncheckable_safe_loops,
            accepts_literal_bounds,
            accepts_counter_and_exit_bounds,
            cli_tests_present,
            no_wasmtime_boundary_stated,
        ]
    )
    return BoundedLoopVerifierStatus(
        schema="garnet.bounded_loop_verifier/v1",
        checker_exports_report=checker_exports_report,
        rejects_uncheckable_safe_loops=rejects_uncheckable_safe_loops,
        accepts_literal_bounds=accepts_literal_bounds,
        accepts_counter_and_exit_bounds=accepts_counter_and_exit_bounds,
        cli_tests_present=cli_tests_present,
        no_wasmtime_boundary_stated=no_wasmtime_boundary_stated,
        ok=ok,
    )


def render_markdown(status: BoundedLoopVerifierStatus) -> str:
    return "\n".join(
        [
            "# Garnet S93 bounded-loop verifier status",
            "",
            f"_Schema {status.schema}._",
            "",
            f"- checker exports static verifier report: {'yes' if status.checker_exports_report else 'NO'}",
            f"- rejects uncheckable safe/@bounded loops: {'yes' if status.rejects_uncheckable_safe_loops else 'NO'}",
            f"- accepts literal range/array bounds: {'yes' if status.accepts_literal_bounds else 'NO'}",
            f"- accepts literal counter / immediate-exit loop bounds: {'yes' if status.accepts_counter_and_exit_bounds else 'NO'}",
            f"- CLI pass/reject tests present: {'yes' if status.cli_tests_present else 'NO'}",
            f"- No Wasmtime fuel / runtime loop-enforcement claim: {'yes' if status.no_wasmtime_boundary_stated else 'NO'}",
            "",
            "S93 is a static verifier for the safe subset. It proves only the "
            "literal finite-loop and immediate-exit cases currently implemented "
            "and rejects uncheckable loops in safe or `@bounded` scope. No "
            "Wasmtime fuel, VM loop enforcement, or OS sandbox enforcement is "
            "claimed.",
            "",
        ]
    )


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument("--gate", action="store_true")
    args = parser.parse_args(list(argv) if argv is not None else None)

    status = read_status()
    if args.format == "md":
        print(render_markdown(status))
    else:
        print(json.dumps(asdict(status), indent=2))

    if args.gate and not status.ok:
        print(f"bounded-loop-verifier gate FAILED: {asdict(status)}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
