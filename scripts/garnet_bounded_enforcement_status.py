#!/usr/bin/env python3
"""`@max_depth` runtime enforcement status (S89).

S89 is the first slice that makes the trust kernel *enforce* at runtime: the
interpreter now traps deterministically when a function declaring `@max_depth(N)`
recurses past its ceiling (`garnet-interp-v0.3/src/eval.rs`, `call_fn`). This is
real enforcement — the interpreter refuses to recurse further — distinct from the
S85 host-stack raise.

This static anti-regression gate asserts the enforcement and its honest-scope
boundary stay in place: the `@max_depth` ceiling lookup + the trapping path live
in the interpreter, and the trap/within/unannotated integration tests exist.

## Honest scope (do not soften)
This is the ONE enforced ceiling. `@bounded` (Wasmtime fuel — S39/S88), memory,
time, and mailbox ceilings remain **declared-not-enforced**. Functions without
`@max_depth` are not capped (they recurse up to the host stack, S85). Mac-authored
+ Mac-tested; the Windows trap re-proves via the cross-OS `cargo test` matrix
(recorded Windows-proof-pending in `WINDOWS_AUDIT_S1_S80.md`).
"""
from __future__ import annotations

import argparse
import json
import sys
from dataclasses import asdict, dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
EVAL = ROOT / "garnet-interp-v0.3" / "src" / "eval.rs"
TEST = ROOT / "garnet-cli" / "tests" / "bounded_enforcement.rs"


@dataclass
class BoundedEnforcementStatus:
    schema: str
    interp_reads_max_depth: bool
    interp_traps_on_exceed: bool
    enforcement_tests_present: bool
    ok: bool = False


def _read(p: Path) -> str:
    return p.read_text(encoding="utf-8") if p.is_file() else ""


def read_status() -> BoundedEnforcementStatus:
    ev = _read(EVAL)
    test = _read(TEST)
    reads = "max_depth_ceiling" in ev and "Annotation::MaxDepth" in ev
    traps = "@max_depth(" in ev and "exceeded for" in ev and "MaxDepthGuard" in ev
    tests_present = (
        "over_ceiling_recursion_traps_deterministically" in test
        and "within_ceiling_recursion_runs" in test
        and "unannotated_recursion_is_not_capped" in test
    )
    ok = reads and traps and tests_present
    return BoundedEnforcementStatus(
        schema="garnet.bounded_enforcement/v1",
        interp_reads_max_depth=reads,
        interp_traps_on_exceed=traps,
        enforcement_tests_present=tests_present,
        ok=ok,
    )


def render_markdown(r: BoundedEnforcementStatus) -> str:
    return "\n".join([
        "# Garnet @max_depth runtime enforcement status (S89)",
        "",
        f"_Schema {r.schema}._",
        "",
        f"- interpreter reads the `@max_depth(N)` ceiling: "
        f"{'yes' if r.interp_reads_max_depth else 'NO'}",
        f"- interpreter traps deterministically on exceed: "
        f"{'yes' if r.interp_traps_on_exceed else 'NO'}",
        f"- trap / within / unannotated tests present: "
        f"{'yes' if r.enforcement_tests_present else 'NO'}",
        "",
        "The ONE enforced ceiling: `@max_depth(N)` recursion. `@bounded` (Wasmtime "
        "fuel), memory, time, and mailbox remain declared-not-enforced. Unannotated "
        "functions are not capped (host stack, S85). Windows trap re-proves via the "
        "cross-OS matrix (Windows-proof-pending).",
        "",
    ])


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument(
        "--gate",
        action="store_true",
        help="exit non-zero unless the interpreter enforces @max_depth (reads the "
        "ceiling + traps on exceed) and the enforcement tests exist.",
    )
    args = parser.parse_args(list(argv) if argv is not None else None)

    r = read_status()
    print(render_markdown(r) if args.format == "md" else json.dumps(asdict(r), indent=2))

    if args.gate and not r.ok:
        print(f"bounded-enforcement gate FAILED: {asdict(r)}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
