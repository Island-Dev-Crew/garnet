#!/usr/bin/env python3
"""Deterministic harness for Paper VI Experiment 2 (Progressive Type-Disclosure)."""

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
        "managed_unannotated_pure",
        "@caps()\ndef add(a, b) { a + b }\n@caps()\ndef main() { add(2, 3) }\n",
        True,
    ),
    (
        "safe_typed_pure",
        "@caps()\nfn add(a: Int, b: Int) -> Int { a + b }\n@caps()\nfn main() -> Int { add(2, 3) }\n",
        True,
    ),
    (
        "managed_to_safe_boundary_valid",
        "@caps()\nfn core(x: Int) -> Int { x * 2 }\n@caps()\ndef main() { core(21) }\n",
        True,
    ),
    (
        "safe_caps_wildcard_rejected",
        "@caps(*)\nfn main() -> Int { 1 }\n",
        False,
    ),
    (
        "safe_unbounded_try_rejected",
        "@caps()\nfn main() -> Int { try { 1 } rescue _ { 2 } }\n",
        False,
    ),
    (
        "safe_undeclared_cap_rejected",
        "@caps()\nfn main() -> String { fs::read_file(\"/tmp/x\") }\n",
        False,
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
    with tempfile.TemporaryDirectory(prefix="garnet-exp2-") as td:
        root = pathlib.Path(td)
        for name, src, expect_ok in CASES:
            p = root / f"{name}.garnet"
            p.write_text(src, encoding="utf-8")
            proc = subprocess.run(
                [str(binary), "check", "--format", "json", str(p)],
                capture_output=True,
                text=True,
                timeout=20,
                check=False,
            )
            ok = (proc.returncode == 0) == expect_ok
            rows.append(
                {
                    "case": name,
                    "expect_ok": expect_ok,
                    "exit_code": proc.returncode,
                    "passed": ok,
                }
            )

    passed = sum(1 for r in rows if r["passed"])
    out = {
        "schema": "garnet.paper_vi_exp2/1",
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
