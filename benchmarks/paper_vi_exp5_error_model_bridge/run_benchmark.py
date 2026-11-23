#!/usr/bin/env python3
"""Deterministic harness for Paper VI Experiment 5 (Error-Model Bridging)."""

from __future__ import annotations

import argparse
import hashlib
import json
import pathlib
import subprocess
import sys
import tempfile


CASES = [
    (
        "managed_raise_caught_by_rescue",
        "@caps()\ndef main() { try { raise(\"oops\") } rescue _ { \"ok\" } }\n",
        0,
        "ok",
    ),
    (
        "safe_result_ok_and_err_match",
        (
            "@caps()\n"
            "fn parse_pos(n: Int) -> Result<Int, String> {\n"
            "  if n > 0 { Ok(n) } else { Err(\"neg\") }\n"
            "}\n"
            "@caps()\n"
            "fn main() -> Int {\n"
            "  match parse_pos(-1) { Ok(v) => v, Err(_) => 42 }\n"
            "}\n"
        ),
        0,
        "42",
    ),
    (
        "managed_calls_safe_result_boundary",
        (
            "@caps()\n"
            "fn safe_div(a: Int, b: Int) -> Result<Int, String> {\n"
            "  if b == 0 { Err(\"zero\") } else { Ok(a / b) }\n"
            "}\n"
            "@caps()\n"
            "def main() {\n"
            "  match safe_div(10, 2) { Ok(v) => v, Err(_) => -1 }\n"
            "}\n"
        ),
        0,
        "5",
    ),
    (
        "enforcement_trap_escapes_managed_rescue_bridge",
        (
            "@caps(fs)\n"
            "def main() {\n"
            "  let f = || { std::env::get(\"HOME\") }\n"
            "  try { f() } rescue _ { \"swallowed\" }\n"
            "}\n"
        ),
        1,
        "capability",
    ),
]


def sha256_file(path: pathlib.Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as fh:
        for chunk in iter(lambda: fh.read(65536), b""):
            h.update(chunk)
    return h.hexdigest()


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--binary", required=True, type=pathlib.Path)
    args = ap.parse_args(argv)
    binary = args.binary.resolve()
    if not binary.is_file():
        print(f"error: binary not found: {binary}", file=sys.stderr)
        return 2

    rows = []
    with tempfile.TemporaryDirectory(prefix="garnet-exp5-") as td:
        root = pathlib.Path(td)
        for name, src, expect_rc, expect_substr in CASES:
            p = root / f"{name}.garnet"
            p.write_text(src, encoding="utf-8")
            proc = subprocess.run(
                [str(binary), "run", str(p)],
                capture_output=True,
                text=True,
                timeout=20,
                check=False,
            )
            combined = (proc.stdout + "\n" + proc.stderr).lower()
            ok = (proc.returncode == expect_rc) and (expect_substr.lower() in combined)
            rows.append(
                {
                    "case": name,
                    "exit_code": proc.returncode,
                    "expect_exit": expect_rc,
                    "passed": ok,
                }
            )

    passed = sum(1 for r in rows if r["passed"])
    out = {
        "schema": "garnet.paper_vi_exp5/1",
        "binary_sha256": sha256_file(binary),
        "total": len(rows),
        "passed": passed,
        "failed": len(rows) - passed,
        "results": rows,
    }
    sys.stdout.write(json.dumps(out, indent=2) + "\n")
    return 0 if passed == len(rows) else 1


if __name__ == "__main__":
    raise SystemExit(main())
