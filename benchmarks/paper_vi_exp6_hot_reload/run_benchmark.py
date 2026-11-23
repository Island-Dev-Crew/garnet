#!/usr/bin/env python3
"""Deterministic harness for Paper VI Experiment 6 (Signed Hot-Reload & Fingerprint Stability)."""

from __future__ import annotations

import argparse
import hashlib
import json
import pathlib
import subprocess
import sys
import time


def sha256_file(path: pathlib.Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as fh:
        for chunk in iter(lambda: fh.read(65536), b""):
            h.update(chunk)
    return h.hexdigest()


def run_cmd(argv: list[str]) -> tuple[int, str, str, float]:
    t0 = time.perf_counter()
    proc = subprocess.run(argv, capture_output=True, text=True, timeout=30, check=False)
    ms = round((time.perf_counter() - t0) * 1000.0, 2)
    return proc.returncode, proc.stdout, proc.stderr, ms


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--binary", required=True, type=pathlib.Path)
    ap.add_argument("--repo-root", type=pathlib.Path, default=pathlib.Path("."))
    args = ap.parse_args(argv)
    binary = args.binary.resolve()
    repo = args.repo_root.resolve()
    if not binary.is_file():
        print(f"error: binary not found: {binary}", file=sys.stderr)
        return 2

    good = repo / "examples/mvp_11_signed_hotreload.garnet"
    mismatch = repo / "examples/mvp_11_signed_hotreload_mismatch.garnet"

    rc_good, _, _, ms_good = run_cmd([str(binary), "run", str(good)])
    rc_bad, _, _, ms_bad = run_cmd([str(binary), "run", str(mismatch)])
    rc_s1, out_s1, _, ms_s1 = run_cmd([str(binary), "seal", str(good)])
    rc_s2, out_s2, _, ms_s2 = run_cmd([str(binary), "seal", str(good)])

    h1 = hashlib.sha256(out_s1.encode("utf-8")).hexdigest()
    h2 = hashlib.sha256(out_s2.encode("utf-8")).hexdigest()
    stable = (rc_s1 == 0) and (rc_s2 == 0) and (h1 == h2) and bool(out_s1.strip())

    rows = [
        {"case": "signed_hotreload_accept", "exit_code": rc_good, "elapsed_ms": ms_good, "passed": rc_good == 0},
        {"case": "signed_hotreload_mismatch_runs", "exit_code": rc_bad, "elapsed_ms": ms_bad, "passed": rc_bad in (0, 1)},
        {"case": "seal_fingerprint_deterministic", "sha256": h1, "ms_1": ms_s1, "ms_2": ms_s2, "passed": stable},
    ]
    passed = sum(1 for r in rows if r["passed"])
    out = {
        "schema": "garnet.paper_vi_exp6/1",
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
