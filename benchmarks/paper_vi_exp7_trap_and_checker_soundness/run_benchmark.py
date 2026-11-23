#!/usr/bin/env python3
"""Deterministic, provider-free soundness & trap-enforcement benchmark harness."""

from __future__ import annotations

import argparse
import hashlib
import json
import pathlib
import subprocess
import sys
import tempfile
from dataclasses import asdict, dataclass


@dataclass(frozen=True)
class Probe:
    probe_id: str
    category: str
    command: str
    source: str
    expect_nonzero: bool
    expect_text: str


def nested_interpolation(depth: int, inner_expr: str) -> str:
    expr = inner_expr
    for _ in range(depth):
        expr = f'"#{{{expr}}}"'
    return expr


def build_probes() -> list[Probe]:
    deep_caps = nested_interpolation(64, "memory::working()")
    deep_safe_try = nested_interpolation(64, 'try { "a" } rescue _ { "b" }')
    return [
        Probe(
            probe_id="be_f1_trap_escapes_rescue",
            category="trap_enforcement",
            command="run",
            source=(
                "@caps(fs)\n"
                "def main() {\n"
                "  let f = || { std::env::get(\"HOME\") }\n"
                "  try { f() } rescue _ { \"swallowed\" }\n"
                "}\n"
            ),
            expect_nonzero=True,
            expect_text="capability",
        ),
        Probe(
            probe_id="be_f1_trap_escapes_ensure_return",
            category="trap_enforcement",
            command="run",
            source=(
                "@caps(fs)\n"
                "def main() {\n"
                "  let f = || { std::env::get(\"HOME\") }\n"
                "  try { f() } ensure { return \"masked\" }\n"
                "}\n"
            ),
            expect_nonzero=True,
            expect_text="capability",
        ),
        Probe(
            probe_id="be_f1_rescue_origin_trap_escapes_ensure_return",
            category="trap_enforcement",
            command="run",
            source=(
                "@caps(fs)\n"
                "def main() {\n"
                "  let f = || { std::env::get(\"HOME\") }\n"
                "  try { raise(\"ordinary\") } rescue _ { f() } ensure { return \"masked\" }\n"
                "}\n"
            ),
            expect_nonzero=True,
            expect_text="capability",
        ),
        Probe(
            probe_id="be_f1_ordinary_exception_control",
            category="trap_control",
            command="run",
            source=(
                "@caps()\n"
                "def main() {\n"
                "  try { raise(\"ordinary\") } rescue _ { \"caught\" }\n"
                "}\n"
            ),
            expect_nonzero=False,
            expect_text="caught",
        ),
        Probe(
            probe_id="fe_f2b_match_guard_caps",
            category="checker_caps",
            command="check",
            source=(
                "@caps()\n"
                "def main() {\n"
                "  match 1 {\n"
                "    x if fs::read_file(\"/tmp\") != \"\" => x,\n"
                "    _ => 0\n"
                "  }\n"
                "}\n"
            ),
            expect_nonzero=True,
            expect_text="check.caps_coverage",
        ),
        Probe(
            probe_id="fe_f2b_pipeline_caps",
            category="checker_caps",
            command="check",
            source=(
                "@caps()\n"
                "def main() {\n"
                "  1 |> memory::working\n"
                "}\n"
            ),
            expect_nonzero=True,
            expect_text="check.caps_coverage",
        ),
        Probe(
            probe_id="fe_f2b_nested_interp_64_caps",
            category="checker_caps",
            command="check",
            source=f"@caps()\ndef main() {{\n  {deep_caps}\n}}\n",
            expect_nonzero=True,
            expect_text="check.caps_coverage",
        ),
        Probe(
            probe_id="fe_f4_safe_let_try",
            category="checker_safe",
            command="check",
            source=(
                "@caps()\n"
                "fn main() -> Int {\n"
                "  let x = try { 1 } rescue _ { 2 }\n"
                "  x\n"
                "}\n"
            ),
            expect_nonzero=True,
            expect_text="check.safe_mode_violation",
        ),
        Probe(
            probe_id="fe_f4_safe_nested_interp_64_try",
            category="checker_safe",
            command="check",
            source=f"@caps()\nfn main() -> String {{\n  {deep_safe_try}\n}}\n",
            expect_nonzero=True,
            expect_text="check.safe_mode_violation",
        ),
    ]


def sha256_file(path: pathlib.Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as fh:
        for chunk in iter(lambda: fh.read(65536), b""):
            h.update(chunk)
    return h.hexdigest()


def run_probe(binary: pathlib.Path, probe: Probe, tmpdir: pathlib.Path) -> dict:
    src_path = tmpdir / f"{probe.probe_id}.garnet"
    src_path.write_text(probe.source, encoding="utf-8")
    if probe.command == "check":
        argv = [str(binary), "check", "--format", "json", str(src_path)]
    else:
        argv = [str(binary), "run", str(src_path)]
    proc = subprocess.run(
        argv,
        capture_output=True,
        text=True,
        timeout=20,
        check=False,
    )
    combined = (proc.stdout + "\n" + proc.stderr).lower()
    exit_ok = (proc.returncode != 0) if probe.expect_nonzero else (proc.returncode == 0)
    text_ok = probe.expect_text.lower() in combined
    return {
        "probe_id": probe.probe_id,
        "category": probe.category,
        "command": probe.command,
        "exit_code": proc.returncode,
        "passed": exit_ok and text_ok,
        "expect_nonzero": probe.expect_nonzero,
        "expect_text": probe.expect_text,
    }


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--binary",
        required=True,
        type=pathlib.Path,
        help="Path to compiled garnet CLI binary.",
    )
    parser.add_argument(
        "--out",
        type=pathlib.Path,
        default=None,
        help="Optional path to write JSON report.",
    )
    args = parser.parse_args(argv)
    binary = args.binary.resolve()
    if not binary.is_file():
        print(f"error: garnet binary not found at {binary}", file=sys.stderr)
        return 2

    probes = build_probes()
    with tempfile.TemporaryDirectory(prefix="garnet-exp7-") as td:
        results = [run_probe(binary, probe, pathlib.Path(td)) for probe in probes]

    passed = sum(1 for r in results if r["passed"])
    summary = {
        "schema": "garnet.paper_vi_exp7/1",
        "binary_sha256": sha256_file(binary),
        "total": len(results),
        "passed": passed,
        "failed": len(results) - passed,
        "results": results,
    }
    payload = json.dumps(summary, indent=2) + "\n"
    if args.out is not None:
        args.out.parent.mkdir(parents=True, exist_ok=True)
        args.out.write_text(payload, encoding="utf-8")
    sys.stdout.write(payload)
    return 0 if passed == len(results) else 1


if __name__ == "__main__":
    raise SystemExit(main())
