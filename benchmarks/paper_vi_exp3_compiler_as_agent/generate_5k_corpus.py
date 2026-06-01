#!/usr/bin/env python3
"""Generate the deterministic S95 5K-LOC Paper VI Exp 3 corpus."""
from __future__ import annotations

import argparse
import hashlib
import json
import shutil
from pathlib import Path

SCHEMA = "garnet.paper_vi_exp3_5k_corpus/v1"


def _sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def _snapshot_lines(index: int, min_loc: int) -> list[str]:
    version = f"v{index:02d}"
    lines = [
        f"# {version}: generated 5K-LOC compiler-as-agent rerun snapshot.",
        "# This file is deterministic source-scale material, not a measured result.",
        f"# Evolution index: {index}.",
        "",
        "@caps()",
        f"def {version}_entry(seed) {{",
        f"  {version}_phase_0001(seed)",
        "}",
        "",
    ]

    case = 1
    while len(lines) < min_loc:
        next_case = case + 1
        lines.extend(
            [
                f"# {version} phase {case:04d}: bounded local rewrite proof seed.",
                f"def {version}_phase_{case:04d}(value) {{",
                f"  {version}_phase_{next_case:04d}(value)",
                "}",
                "",
            ]
        )
        case = next_case

    lines.extend(
        [
            f"def {version}_phase_{case:04d}(value) {{",
            "  value",
            "}",
            "",
            "@caps()",
            "def main() {",
            f"  {version}_entry({index})",
            "}",
        ]
    )
    return lines


def generate(output: Path, snapshots: int = 10, min_loc: int = 5000) -> dict:
    if output.exists():
        shutil.rmtree(output)
    output.mkdir(parents=True)

    manifest: dict = {
        "schema": SCHEMA,
        "snapshot_count": snapshots,
        "requested_min_loc": min_loc,
        "snapshots": [],
    }
    total_loc = 0
    min_snapshot_loc = 0

    for index in range(1, snapshots + 1):
        version = f"v{index:02d}"
        snapshot_dir = output / "codebase_versions" / version
        snapshot_dir.mkdir(parents=True)
        source = snapshot_dir / "main.garnet"
        source.write_text("\n".join(_snapshot_lines(index, min_loc)) + "\n", encoding="utf-8")
        loc = len(source.read_text(encoding="utf-8").splitlines())
        total_loc += loc
        min_snapshot_loc = loc if min_snapshot_loc == 0 else min(min_snapshot_loc, loc)
        manifest["snapshots"].append(
            {
                "id": version,
                "path": str(source.relative_to(output)).replace("\\", "/"),
                "loc": loc,
                "sha256": _sha256(source),
                "evolution_index": index,
            }
        )

    manifest["min_snapshot_loc"] = min_snapshot_loc
    manifest["total_loc"] = total_loc
    manifest_path = output / "manifest.json"
    manifest_path.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return manifest


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--snapshots", type=int, default=10)
    parser.add_argument("--min-loc", type=int, default=5000)
    args = parser.parse_args(argv)

    manifest = generate(args.output, snapshots=args.snapshots, min_loc=args.min_loc)
    print(json.dumps(manifest, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
