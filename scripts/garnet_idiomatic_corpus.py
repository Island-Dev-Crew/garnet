#!/usr/bin/env python3
"""Idiomatic Garnet corpus harness (S57).

A small, open corpus of *idiomatic* Garnet programs — ones that follow the
policies the hardening band established: named `@caps`, **typed** rescues (S42,
never a catch-all), exhaustive `match` over finite enums. The bar is high: each
must `garnet check` to **0 diagnostics** (fully clean — not even a non-fatal
advisory) and `garnet run` to its recorded output.

This complements `smoke_garnet_novel_compositions.py` (which fuses Paper-VI
contributions) and `smoke_garnet_studio_domain_matrix.py` (the 12 domains) — this
one is about *style*: what good Garnet looks like.

## Honest scope
A style/discipline corpus, not a performance or coverage claim. "Idiomatic" here
means: clean checker output + the hardening-band idioms, proven deterministically.
"""
from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
from dataclasses import asdict, dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8")

CLEAN_MARKER = "0 diagnostics"


@dataclass(frozen=True)
class IdiomaticCase:
    file: str
    idiom: str
    run_expect: tuple[str, ...]


CORPUS: tuple[IdiomaticCase, ...] = (
    IdiomaticCase(
        "examples/idiomatic/typed_errors.garnet",
        "typed rescue (S42 error policy) — names the exception type, never a catch-all",
        ("handled", "found: y"),
    ),
    IdiomaticCase(
        "examples/idiomatic/state_machine.garnet",
        "exhaustive match over a finite enum — no catch-all arm",
        ("after red: green",),
    ),
)


@dataclass
class CaseResult:
    file: str
    idiom: str
    check_clean: bool
    run_ok: bool
    passed: bool
    detail: str


def evaluate_case(case: IdiomaticCase, ccode: int, cout: str, rcode: int, rout: str) -> CaseResult:
    check_clean = ccode == 0 and CLEAN_MARKER in cout
    run_ok = rcode == 0 and all(s in rout for s in case.run_expect)
    detail = ""
    if not check_clean:
        detail = "check not clean (idiomatic code must be 0 diagnostics)"
    elif not run_ok:
        detail = "run output did not match expected"
    return CaseResult(
        file=case.file,
        idiom=case.idiom,
        check_clean=check_clean,
        run_ok=run_ok,
        passed=check_clean and run_ok,
        detail=detail or "ok",
    )


def resolve_garnet(explicit: str | None) -> list[str]:
    if explicit:
        return [explicit]
    env = os.environ.get("GARNET_CLI")
    if env:
        return [env]
    exe = "garnet.exe" if os.name == "nt" else "garnet"
    cands = [ROOT / "target" / p / exe for p in ("release", "debug")]
    cands = [c for c in cands if c.exists()]
    if cands:
        return [str(max(cands, key=lambda p: p.stat().st_mtime))]
    found = shutil.which("garnet")
    if found:
        return [found]
    raise FileNotFoundError("garnet CLI not found; build with `cargo build -p garnet-cli`")


def _run(argv: list[str]) -> tuple[int, str]:
    proc = subprocess.run(argv, cwd=ROOT, capture_output=True, text=True)
    return proc.returncode, proc.stdout + proc.stderr


def run_corpus(garnet: list[str]) -> list[CaseResult]:
    results = []
    for case in CORPUS:
        ccode, cout = _run([*garnet, "check", case.file])
        rcode, rout = _run([*garnet, "run", case.file])
        results.append(evaluate_case(case, ccode, cout, rcode, rout))
    return results


def render_markdown(results: list[CaseResult]) -> str:
    lines = [
        "# Garnet idiomatic corpus",
        "",
        "| file | idiom | check clean | run ok |",
        "|---|---|---|---|",
    ]
    for r in results:
        lines.append(
            f"| `{r.file}` | {r.idiom} | {'✅' if r.check_clean else '❌'} | {'✅' if r.run_ok else '❌'} |"
        )
    passed = sum(1 for r in results if r.passed)
    lines += ["", f"**{passed}/{len(results)} idiomatic programs clean + running.**", ""]
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--garnet", help="path to the garnet CLI")
    parser.add_argument("--format", choices=["json", "md"], default="md")
    args = parser.parse_args(list(argv) if argv is not None else None)

    garnet = resolve_garnet(args.garnet)
    results = run_corpus(garnet)
    if args.format == "md":
        print(render_markdown(results))
    else:
        print(json.dumps({"corpus": [asdict(r) for r in results]}, indent=2))
    return 0 if all(r.passed for r in results) else 1


if __name__ == "__main__":
    raise SystemExit(main())
