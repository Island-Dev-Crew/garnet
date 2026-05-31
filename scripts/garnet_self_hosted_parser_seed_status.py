#!/usr/bin/env python3
"""Self-hosted parser SEED status (S72).

`examples/self_hosted_parser_seed.garnet` is a Garnet program that parses a
subset of Garnet's OWN surface syntax — `def name(params) { ... }` declarations
from an embedded source string — counting arity and detecting the `@caps(...)`
managed-function annotation, using only Stable, no-caps `str::` primitives.

This reporter inventories the seed (present + well-formed) and, when the `garnet`
binary is built, RUNS the proof: `garnet check` (expects 0 diagnostics) and
`garnet run` (expects the deterministic parse output). When the binary is absent
(e.g. the python-only CI agent-contracts job), the dynamic proof is skipped and
the gate falls back to static well-formedness — the binary-backed proof runs in
the canonical-examples CI job, which builds the compiler.

## Honest scope (do not soften)
A SEED toward self-hosting, NOT the production parser (`garnet-parser-v0.3`). It
recognizes def headers + @caps lines; it does not build a full AST or handle the
whole grammar (nested braces, expressions, types, comments). Full self-hosting
remains roadmap work.
"""
from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from dataclasses import asdict, dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SEED = ROOT / "examples" / "self_hosted_parser_seed.garnet"

# Markers that prove the seed is a real Garnet-parsing program, not a stub.
REQUIRED_MARKERS = [
    "def parse_name(",
    "def parse_arity(",
    'str::split(',
    'str::starts_with(t, "def ")',
]

# Deterministic `garnet run` output lines the seed must emit.
EXPECTED_RUN_LINES = [
    "def main arity 0 caps yes",
    "def add arity 2 caps no",
    "def greet arity 1 caps no",
    "parsed defs: 3 managed: 1",
]


def _garnet_binary() -> Path | None:
    exe = "garnet.exe" if os.name == "nt" else "garnet"
    for profile in ("release", "debug"):
        cand = ROOT / "target" / profile / exe
        if cand.exists():
            return cand
    return None


@dataclass
class SeedStatus:
    schema: str
    seed_present: bool
    missing_markers: list[str]
    binary_available: bool
    check_clean: bool
    run_matches: bool
    missing_run_lines: list[str] = field(default_factory=list)
    ok: bool = False


def read_status(run_binary: bool = True) -> SeedStatus:
    src = SEED.read_text(encoding="utf-8") if SEED.is_file() else ""
    missing = [m for m in REQUIRED_MARKERS if m not in src]
    well_formed = bool(src) and not missing

    binary = _garnet_binary() if run_binary else None
    check_clean = False
    run_matches = False
    missing_lines: list[str] = list(EXPECTED_RUN_LINES)
    if binary is not None and SEED.is_file():
        chk = subprocess.run(
            [str(binary), "check", str(SEED)], capture_output=True, text=True, timeout=120
        )
        check_clean = chk.returncode == 0 and "0 diagnostics" in chk.stdout
        run = subprocess.run(
            [str(binary), "run", str(SEED)], capture_output=True, text=True, timeout=120
        )
        out = run.stdout
        missing_lines = [ln for ln in EXPECTED_RUN_LINES if ln not in out]
        run_matches = run.returncode == 0 and not missing_lines

    # Gate: well-formed always; if the binary is present it must also pass.
    if binary is None:
        ok = well_formed
    else:
        ok = well_formed and check_clean and run_matches

    return SeedStatus(
        schema="garnet.self_hosted_parser_seed/v1",
        seed_present=well_formed,
        missing_markers=missing,
        binary_available=binary is not None,
        check_clean=check_clean,
        run_matches=run_matches,
        missing_run_lines=missing_lines,
        ok=ok,
    )


def render_markdown(r: SeedStatus) -> str:
    lines = [
        "# Garnet self-hosted parser seed status (S72)",
        "",
        f"_Schema {r.schema}._",
        "",
        f"- seed present + well-formed: {'yes' if r.seed_present else 'NO'}"
        + (f" (missing markers: {r.missing_markers})" if r.missing_markers else ""),
        f"- garnet binary available: {'yes' if r.binary_available else 'no (dynamic proof skipped)'}",
        f"- `garnet check` clean (0 diagnostics): {'yes' if r.check_clean else ('n/a' if not r.binary_available else 'NO')}",
        f"- `garnet run` matches expected output: {'yes' if r.run_matches else ('n/a' if not r.binary_available else f'NO {r.missing_run_lines}')}",
        "",
        "The seed parses a subset of Garnet's OWN `def` syntax (name + arity + "
        "@caps managed flag) using Stable `str::` primitives. Honest scope: a SEED, "
        "NOT the production parser; no full AST / grammar. Full self-hosting is roadmap.",
        "",
    ]
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument(
        "--gate",
        action="store_true",
        help="exit non-zero if the seed is missing/malformed (or, when the binary "
        "is present, if check/run fail). Dynamic proof is skipped if the binary is absent.",
    )
    parser.add_argument("--no-run", action="store_true", help="skip running the binary")
    args = parser.parse_args(list(argv) if argv is not None else None)

    r = read_status(run_binary=not args.no_run)
    if args.format == "md":
        print(render_markdown(r))
    else:
        print(json.dumps(asdict(r), indent=2))

    if args.gate and not r.ok:
        print(
            "self-hosted-parser-seed gate FAILED: "
            f"seed_present={r.seed_present} missing_markers={r.missing_markers} "
            f"binary={r.binary_available} check_clean={r.check_clean} "
            f"run_matches={r.run_matches} missing_run_lines={r.missing_run_lines}",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
