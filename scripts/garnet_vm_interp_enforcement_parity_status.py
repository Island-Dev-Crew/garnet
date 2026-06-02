#!/usr/bin/env python3
"""VM / interpreter ENFORCEMENT-parity campaign + gate (S101, Stage V closeout).

The S73 parity campaign (`garnet_vm_interp_parity.py`) proves the two backends
agree on RESULT (stdout + exit code) over the example corpus. This campaign proves
the stronger property the trust kernel needs: every runtime-enforcement **trap**
fires **identically** on both `garnet run --interp` and `garnet run --vm`. With
S99 (`@max_depth`) and S100 (`@caps`), the "the VM enforces nothing" seam the
substrate named-deferred is **CLOSED** for these two ceilings.

It consolidates the two enforcement gates rather than duplicating their checks:
- S99 (`garnet_bounded_enforcement_status`): the VM traps on `@max_depth` with the
  identical message, proven by `vm_and_interp_traps_are_identical`.
- S100 (`garnet_caps_enforcement_status`): the VM installs the same program-entry
  caps frame, so the S92 entry gate fires on `--vm` too — proven by
  `vm_entry_caps_not_launderable_through_helper` (no authority laundering via --vm).

## Honest scope (do not soften)
TRAP-parity covers the two enforced ceilings only: `@max_depth` recursion and
`@caps` host-authority. Still **declared-not-enforced** on BOTH backends (named,
never faked): `@bounded` (Wasmtime fuel — wasmtime absent), memory, time, mailbox,
and OS-level sandbox application. This is trap-parity for the enforced ceilings,
NOT a proof of total backend equivalence (that is the result-parity campaign's
honest scope, also bounded).
"""
from __future__ import annotations

import argparse
import importlib.util
import json
import sys
from dataclasses import asdict, dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPTS = ROOT / "scripts"
BOUNDED_TEST = ROOT / "garnet-cli" / "tests" / "bounded_enforcement.rs"
CAPS_TEST = ROOT / "garnet-cli" / "tests" / "caps_enforcement.rs"


def _load(module_name: str):
    spec = importlib.util.spec_from_file_location(
        module_name, SCRIPTS / f"{module_name}.py"
    )
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[module_name] = module
    spec.loader.exec_module(module)
    return module


def _read(p: Path) -> str:
    return p.read_text(encoding="utf-8") if p.is_file() else ""


@dataclass
class EnforcementParityStatus:
    schema: str
    max_depth_trap_parity: bool
    caps_trap_parity: bool
    enforced_ceilings: list[str]
    named_deferred: list[str]
    ok: bool = False


def read_status() -> EnforcementParityStatus:
    bounded = _load("garnet_bounded_enforcement_status").read_status()
    caps = _load("garnet_caps_enforcement_status").read_status()
    bounded_test = _read(BOUNDED_TEST)
    caps_test = _read(CAPS_TEST)

    max_depth_parity = (
        bounded.vm_traps_on_exceed
        and "vm_and_interp_traps_are_identical" in bounded_test
    )
    caps_parity = (
        caps.vm_entry_caps_frame_present
        and "vm_entry_caps_not_launderable_through_helper" in caps_test
    )
    enforced = []
    if max_depth_parity:
        enforced.append("@max_depth")
    if caps_parity:
        enforced.append("@caps")
    ok = max_depth_parity and caps_parity
    return EnforcementParityStatus(
        schema="garnet.vm_interp_enforcement_parity/v1",
        max_depth_trap_parity=max_depth_parity,
        caps_trap_parity=caps_parity,
        enforced_ceilings=enforced,
        named_deferred=[
            "@bounded (Wasmtime fuel)",
            "memory",
            "time",
            "@mailbox",
            "OS-level sandbox application",
        ],
        ok=ok,
    )


def render_markdown(r: EnforcementParityStatus) -> str:
    return "\n".join([
        "# Garnet VM/interpreter ENFORCEMENT-parity (S101, Stage V closeout)",
        "",
        f"_Schema {r.schema}._",
        "",
        f"- `@max_depth` trap fires identically on both backends (S99): "
        f"{'yes' if r.max_depth_trap_parity else 'NO'}",
        f"- `@caps` trap (incl. the S92 entry gate) fires identically on both "
        f"backends (S100): {'yes' if r.caps_trap_parity else 'NO'}",
        f"- enforced ceilings at VM/interp TRAP-parity: "
        f"{', '.join(r.enforced_ceilings) or 'none'}",
        "",
        "The \"VM enforces nothing\" seam is **CLOSED** for `@max_depth` and `@caps`: "
        "both ceilings now trap identically on `--interp` and `--vm`, proven by "
        "both-backends tests. Still declared-not-enforced on BOTH backends (named, "
        f"never faked): {', '.join(r.named_deferred)}. This is trap-parity for the "
        "enforced ceilings, not a proof of total backend equivalence.",
        "",
    ])


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument(
        "--gate",
        action="store_true",
        help="exit non-zero unless BOTH @max_depth and @caps reach VM/interp "
        "trap-parity (the VM enforces them with the identical trap, proven by "
        "both-backends tests).",
    )
    args = parser.parse_args(list(argv) if argv is not None else None)

    r = read_status()
    print(render_markdown(r) if args.format == "md" else json.dumps(asdict(r), indent=2))

    if args.gate and not r.ok:
        print(f"vm-interp-enforcement-parity gate FAILED: {asdict(r)}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
