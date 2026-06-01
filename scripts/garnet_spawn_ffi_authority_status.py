#!/usr/bin/env python3
"""S92 subprocess and FFI authority status.

S92 closes the interpreter-visible subprocess laundering gap: a helper function
declaring `@caps(proc)` is not enough to launch a process unless the program
entry point also declares `@caps(proc)`.

Honest scope: FFI is declared/diffed/sandbox-flagged/sealed in the source and
policy surfaces, but there is no executable FFI runtime bridge in this repo yet.
That means S92 can record FFI as scoped and deferred, not runtime-enforced.
"""
from __future__ import annotations

import argparse
import json
import sys
from dataclasses import asdict, dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
EVAL = ROOT / "garnet-interp-v0.3" / "src" / "eval.rs"
BRIDGE = ROOT / "garnet-interp-v0.3" / "src" / "stdlib_bridge.rs"
TEST = ROOT / "garnet-cli" / "tests" / "caps_enforcement.rs"

REQUIRED_ENTRY_GATES = [
    ('require_entry_capability("proc", "std::process::spawn")', "std::process::spawn"),
    (
        'require_entry_capability("proc", "std::process::spawn_args")',
        "std::process::spawn_args",
    ),
    ('require_entry_capability("proc", "std::process::output")', "std::process::output"),
]


@dataclass(frozen=True)
class SpawnFfiAuthorityStatus:
    schema: str
    subprocess_entry_guard_present: bool
    missing_subprocess_entry_gates: list[str]
    runtime_tests_present: bool
    ffi_runtime_bridge_present: bool
    ffi_scope: str
    ok: bool


def _read(path: Path) -> str:
    return path.read_text(encoding="utf-8") if path.is_file() else ""


def read_status() -> SpawnFfiAuthorityStatus:
    eval_src = _read(EVAL)
    bridge_src = _read(BRIDGE)
    test_src = _read(TEST)

    guard_present = (
        "pub(crate) fn require_entry_capability" in eval_src
        and "fn enter_entry" in eval_src
        and "entry_frames" in eval_src
        and "entry_counts" in eval_src
        and "requires program entry @caps" in eval_src
    )
    missing = [label for needle, label in REQUIRED_ENTRY_GATES if needle not in bridge_src]
    tests_present = (
        "proc_helper_laundering_traps_when_entry_lacks_proc" in test_src
        and "proc_helper_runs_when_entry_declares_proc" in test_src
    )

    # There is no FFI bridge in stdlib_bridge today. Keep this intentionally
    # conservative: if an executable FFI host bridge is later added, the status
    # gate should force that slice to add its own runtime authority proof.
    ffi_runtime_bridge_present = (
        'require_entry_capability("ffi"' in bridge_src
        or "std::ffi::" in bridge_src
        or "bridge_ffi" in bridge_src
    )
    ffi_scope = (
        "FFI is declared/diffed/sandbox-flagged/sealed today; no executable FFI "
        "bridge exists, so S92 records runtime FFI enforcement as deferred rather "
        "than shipped."
    )
    ok = guard_present and not missing and tests_present and not ffi_runtime_bridge_present

    return SpawnFfiAuthorityStatus(
        schema="garnet.spawn_ffi_authority/v1",
        subprocess_entry_guard_present=guard_present,
        missing_subprocess_entry_gates=missing,
        runtime_tests_present=tests_present,
        ffi_runtime_bridge_present=ffi_runtime_bridge_present,
        ffi_scope=ffi_scope,
        ok=ok,
    )


def render_markdown(report: SpawnFfiAuthorityStatus) -> str:
    return "\n".join(
        [
            "# Garnet S92 subprocess/FFI authority status",
            "",
            f"_Schema {report.schema}._",
            "",
            "- subprocess program-entry guard present: "
            f"{'yes' if report.subprocess_entry_guard_present else 'NO'}",
            "- subprocess entry-gated launch bridges: "
            + (
                "all present"
                if not report.missing_subprocess_entry_gates
                else f"missing {report.missing_subprocess_entry_gates}"
            ),
            "- runtime laundering regression tests present: "
            f"{'yes' if report.runtime_tests_present else 'NO'}",
            "- executable FFI runtime bridge present: "
            f"{'yes' if report.ffi_runtime_bridge_present else 'no'}",
            "",
            report.ffi_scope,
            "",
        ]
    )


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument("--gate", action="store_true")
    args = parser.parse_args(list(argv) if argv is not None else None)

    report = read_status()
    if args.format == "md":
        print(render_markdown(report))
    else:
        print(json.dumps(asdict(report), indent=2))

    if args.gate and not report.ok:
        print(f"spawn/ffi authority gate FAILED: {asdict(report)}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
