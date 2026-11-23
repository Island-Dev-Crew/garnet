#!/usr/bin/env python3
"""Deterministic harness for Paper VI Experiment 4 (Kind-Aware Allocation & Mnemos Memory)."""

from __future__ import annotations

import argparse
import hashlib
import json
import pathlib
import resource
import subprocess
import sys
import time


WORKLOADS = [
    "examples/novel_01_capability_budgeted_memory_agent.garnet",
    "examples/novel_05_s22_stdlib_memory_pipeline.garnet",
    "examples/mvp_01_os_simulator.garnet",
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
    ap.add_argument("--repo-root", type=pathlib.Path, default=pathlib.Path("."))
    args = ap.parse_args(argv)
    binary = args.binary.resolve()
    repo = args.repo_root.resolve()
    if not binary.is_file():
        print(f"error: binary not found: {binary}", file=sys.stderr)
        return 2

    rows = []
    for rel in WORKLOADS:
        target = repo / rel
        if not target.is_file():
            rows.append({"workload": rel, "passed": False, "error": "missing file"})
            continue
        t0 = time.perf_counter()
        proc = subprocess.run(
            [str(binary), "run", str(target)],
            capture_output=True,
            text=True,
            timeout=30,
            check=False,
        )
        elapsed_ms = round((time.perf_counter() - t0) * 1000.0, 2)
        ru = resource.getrusage(resource.RUSAGE_CHILDREN)
        rows.append(
            {
                "workload": rel,
                "exit_code": proc.returncode,
                "elapsed_ms": elapsed_ms,
                "child_maxrss": ru.ru_maxrss,
                "passed": proc.returncode == 0,
            }
        )

    passed = sum(1 for r in rows if r["passed"])
    out = {
        "schema": "garnet.paper_vi_exp4/1",
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
