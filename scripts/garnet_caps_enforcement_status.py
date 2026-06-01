#!/usr/bin/env python3
"""`@caps` host-authority runtime enforcement status (S91).

S90 extends runtime enforcement (S89) from `@max_depth` to capabilities: the
interpreter (`garnet-interp-v0.3/src/eval.rs` `require_capability` + `CapsGuard`,
wired into the `std::env`/`std::process`/`fs::`/`std::log::to_file` bridges in
`stdlib_bridge.rs`) traps when a managed function invokes a host-authority
primitive whose required capability no frame in the call chain declared. `garnet
run` does not run the static checker, so this is the runtime backstop.

S91 extends that runtime backstop: `net::tcp_connect` is now bridge-gated, and
`garnet run --interp` calls `main` through a program-entry frame so safe-mode
entry points cannot bypass caps just because no managed frame is active.

This static anti-regression gate asserts the enforcement + its bridge wiring stay
in place.

## Honest scope (do not soften)
Host-authority surfaces only (env / process / fs / net / log-to-file); pure
computation is unaffected, and outside any program-entry/direct function frame
(direct host/test calls) there is no `@caps` context to enforce, so such calls
are allowed. The **VM** backend does not yet enforce `@caps`.
"""
from __future__ import annotations

import argparse
import json
import sys
from dataclasses import asdict, dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
EVAL = ROOT / "garnet-interp-v0.3" / "src" / "eval.rs"
INTERP = ROOT / "garnet-interp-v0.3" / "src" / "lib.rs"
RUN = ROOT / "garnet-cli" / "src" / "cmd" / "run.rs"
BRIDGE = ROOT / "garnet-interp-v0.3" / "src" / "stdlib_bridge.rs"
TEST = ROOT / "garnet-cli" / "tests" / "caps_enforcement.rs"

# Each host-authority cap must be gated at its bridge(s).
REQUIRED_GATES = [
    ('require_capability("env"', "env"),
    ('require_capability("proc"', "proc"),
    ('require_capability("fs"', "fs"),
    ('require_capability("net"', "net"),
]


@dataclass
class CapsEnforcementStatus:
    schema: str
    interp_has_require_capability: bool
    program_entry_frame_present: bool
    bridges_gated: list[str]
    missing_gates: list[str]
    enforcement_tests_present: bool
    ok: bool = False


def _read(p: Path) -> str:
    return p.read_text(encoding="utf-8") if p.is_file() else ""


def read_status() -> CapsEnforcementStatus:
    ev = _read(EVAL)
    interp = _read(INTERP)
    run = _read(RUN)
    bridge = _read(BRIDGE)
    test = _read(TEST)
    has_require = (
        "pub(crate) fn require_capability" in ev
        and "CapsGuard" in ev
        and "not declared in the calling chain" in ev
    )
    entry_frame = (
        "call_entry" in interp
        and "call_value_with_entry_caps" in ev
        and 'interp.call_entry("main"' in run
        and "managed_frames == 0" not in ev
    )
    gated = [cap for (needle, cap) in REQUIRED_GATES if needle in bridge]
    missing = [cap for (needle, cap) in REQUIRED_GATES if needle not in bridge]
    tests_present = (
        "undeclared_env_traps" in test
        and "undeclared_proc_traps_before_spawning" in test
        and "undeclared_fs_traps" in test
        and "undeclared_net_traps_before_connect_policy" in test
        and "program_entry_frame_traps_safe_main_env_without_caps" in test
        and "program_entry_frame_allows_safe_main_declared_env" in test
        and "pure_computation_is_unaffected" in test
    )
    ok = has_require and entry_frame and not missing and tests_present
    return CapsEnforcementStatus(
        schema="garnet.caps_enforcement/v1",
        interp_has_require_capability=has_require,
        program_entry_frame_present=entry_frame,
        bridges_gated=gated,
        missing_gates=missing,
        enforcement_tests_present=tests_present,
        ok=ok,
    )


def render_markdown(r: CapsEnforcementStatus) -> str:
    return "\n".join([
        "# Garnet @caps host-authority runtime enforcement status (S91)",
        "",
        f"_Schema {r.schema}._",
        "",
        f"- interpreter has `require_capability` + `CapsGuard` + trap: "
        f"{'yes' if r.interp_has_require_capability else 'NO'}",
        f"- program-entry caps frame wired into `garnet run --interp`: "
        f"{'yes' if r.program_entry_frame_present else 'NO'}",
        f"- bridges gated: {r.bridges_gated or 'none'}"
        + (f" (missing: {r.missing_gates})" if r.missing_gates else ""),
        f"- env/proc/fs/net/program-entry/pure enforcement tests present: "
        f"{'yes' if r.enforcement_tests_present else 'NO'}",
        "",
        "Host-authority surfaces only (env/process/fs/net/log-to-file); pure "
        "computation unaffected; calls outside a program-entry/direct function frame "
        "are allowed. The VM backend does not yet enforce @caps.",
        "",
    ])


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument(
        "--gate",
        action="store_true",
        help="exit non-zero unless the interpreter enforces @caps (require_capability "
        "+ CapsGuard), the env/proc/fs bridges are gated, and the tests exist.",
    )
    args = parser.parse_args(list(argv) if argv is not None else None)

    r = read_status()
    print(render_markdown(r) if args.format == "md" else json.dumps(asdict(r), indent=2))

    if args.gate and not r.ok:
        print(f"caps-enforcement gate FAILED: {asdict(r)}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
