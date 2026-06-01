#!/usr/bin/env python3
"""Interpreter large-stack robustness status (S85 / WIN-S73-001).

The tree-walking interpreter (`garnet run --interp`) stack-overflowed on Windows
(default ~1 MiB thread stack) for `mvp_function_call_demo.garnet` while the VM
succeeded, so the binary-backed VM/interpreter parity campaign diverged on Windows.
S85 runs the interpreter evaluation on a thread with a large explicit stack
(`std::thread::Builder::stack_size`) on every platform, so deep-but-finite
recursion no longer overflows.

This is a static anti-regression gate: it asserts the interpreter entry in
`garnet-cli/src/cmd/run.rs` still spawns a large-stack thread and routes through
`run_interpreter_inner`, so a future edit cannot quietly drop back to the default
stack.

## Honest scope (do not soften)
Raising the stack covers deep-but-finite recursion (e.g. the audit fixture); it is
NOT an unbounded guarantee — recursion past the large stack still overflows, which
is the `@bounded` enforcement story (S89). Mac-authored + Mac-tested (a 5000-deep
recursion that overflows the default stack passes on the large one); the original
Windows fixture re-proves via the cross-OS `cargo test` matrix (Windows-proof-pending
end-to-end check is recorded in `WINDOWS_AUDIT_S1_S80.md`).
"""
from __future__ import annotations

import argparse
import json
import sys
from dataclasses import asdict, dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
RUN = ROOT / "garnet-cli" / "src" / "cmd" / "run.rs"
TEST = ROOT / "garnet-cli" / "tests" / "interp_deep_recursion.rs"


@dataclass
class InterpStackStatus:
    schema: str
    spawns_large_stack_thread: bool
    routes_through_inner: bool
    deep_recursion_test_present: bool
    ok: bool = False


def _read(p: Path) -> str:
    return p.read_text(encoding="utf-8") if p.is_file() else ""


def read_status() -> InterpStackStatus:
    run = _read(RUN)
    test = _read(TEST)
    large_stack = (
        "thread::Builder::new()" in run
        and ".stack_size(" in run
        and '"garnet-interp"' in run
    )
    routes = "run_interpreter_inner(" in run
    test_present = "deep_finite_recursion_uses_the_large_stack" in test and "audit_fixture_runs_on_interpreter" in test
    ok = large_stack and routes and test_present
    return InterpStackStatus(
        schema="garnet.interp_stack/v1",
        spawns_large_stack_thread=large_stack,
        routes_through_inner=routes,
        deep_recursion_test_present=test_present,
        ok=ok,
    )


def render_markdown(r: InterpStackStatus) -> str:
    return "\n".join([
        "# Garnet interpreter large-stack status (S85)",
        "",
        f"_Schema {r.schema}._",
        "",
        f"- interpreter spawns a large-stack thread (`stack_size`): "
        f"{'yes' if r.spawns_large_stack_thread else 'NO'}",
        f"- routes through `run_interpreter_inner`: {'yes' if r.routes_through_inner else 'NO'}",
        f"- deep-recursion + audit-fixture tests present: "
        f"{'yes' if r.deep_recursion_test_present else 'NO'}",
        "",
        "Raises the recursion ceiling (closes WIN-S73-001 for deep-but-finite "
        "recursion); NOT unbounded — past the large stack still overflows, which is "
        "the @bounded enforcement story (S89). Windows end-to-end proof via the "
        "cross-OS cargo matrix; recorded Windows-proof-pending.",
        "",
    ])


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument(
        "--gate",
        action="store_true",
        help="exit non-zero unless the interpreter spawns a large-stack thread, "
        "routes through run_interpreter_inner, and the deep-recursion tests exist.",
    )
    args = parser.parse_args(list(argv) if argv is not None else None)

    r = read_status()
    print(render_markdown(r) if args.format == "md" else json.dumps(asdict(r), indent=2))

    if args.gate and not r.ok:
        print(f"interp-stack gate FAILED: {asdict(r)}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
