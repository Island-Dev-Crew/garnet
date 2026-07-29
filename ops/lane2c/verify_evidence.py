#!/usr/bin/env python3
"""Verify Lane 2C Callgrind evidence without using wall-clock thresholds."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
from pathlib import Path


LANE_ROOT = Path(__file__).resolve().parent
REPO_ROOT = LANE_ROOT.parents[1]
EVIDENCE_ROOT = LANE_ROOT / "evidence"
MEASUREMENT_PATH = EVIDENCE_ROOT / "measurement.json"
PROFILE_ROOT = EVIDENCE_ROOT / "callgrind"
STRESS_PATH = EVIDENCE_ROOT / "stress.txt"
MANIFEST_PATH = EVIDENCE_ROOT / "MANIFEST.sha256"


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def parse_profile(path: Path) -> dict[str, object]:
    header: dict[str, str] = {}
    with path.open("r", encoding="utf-8") as handle:
        for line in handle:
            if line == "\n":
                if "summary" in header:
                    break
                continue
            match = re.match(r"^(creator|cmd|events|summary):\s*(.*?)\s*$", line)
            if match:
                header[match.group(1)] = match.group(2)
    missing = {"creator", "cmd", "events", "summary"} - header.keys()
    if missing:
        raise ValueError(f"{path}: missing Callgrind fields {sorted(missing)}")
    return {
        "creator": header["creator"],
        "command": header["cmd"],
        "events": header["events"],
        "instructions": int(header["summary"]),
        "sha256": sha256(path),
    }


def verify_manifest(findings: list[str]) -> None:
    if not MANIFEST_PATH.is_file():
        findings.append("evidence manifest missing")
        return

    recorded: dict[Path, str] = {}
    for line_number, line in enumerate(
        MANIFEST_PATH.read_text(encoding="utf-8").splitlines(), start=1
    ):
        parts = line.split(None, 1)
        if len(parts) != 2 or not re.fullmatch(r"[0-9a-f]{64}", parts[0]):
            findings.append(f"evidence manifest line {line_number} is malformed")
            continue
        relative = Path(parts[1].strip())
        if relative.is_absolute() or ".." in relative.parts:
            findings.append(f"evidence manifest line {line_number} escapes evidence root")
            continue
        if relative in recorded:
            findings.append(f"evidence manifest duplicates {relative}")
            continue
        recorded[relative] = parts[0]

    actual = {
        path.relative_to(EVIDENCE_ROOT)
        for path in EVIDENCE_ROOT.rglob("*")
        if path.is_file() and path != MANIFEST_PATH
    }
    recorded_paths = set(recorded)
    if actual != recorded_paths:
        findings.append(
            "evidence manifest set mismatch: "
            f"missing={sorted(map(str, actual - recorded_paths))}, "
            f"extra={sorted(map(str, recorded_paths - actual))}"
        )
    for relative, expected in recorded.items():
        path = EVIDENCE_ROOT / relative
        if path.is_file() and sha256(path) != expected:
            findings.append(f"evidence manifest hash mismatch: {relative}")


def verify() -> dict[str, object]:
    measurement = json.loads(MEASUREMENT_PATH.read_text(encoding="utf-8"))
    findings: list[str] = []
    observed: dict[tuple[str, int, str], int] = {}

    verify_manifest(findings)

    harness_path = REPO_ROOT / measurement["harness"]["source_path"]
    if not harness_path.is_file():
        findings.append(f"shipped harness missing: {harness_path}")
    elif sha256(harness_path) != measurement["harness"]["source_sha256"]:
        findings.append("shipped harness SHA-256 mismatch")

    lockfile_path = REPO_ROOT / measurement["lockfile"]["root_path"]
    if not lockfile_path.is_file():
        findings.append(f"root lockfile missing: {lockfile_path}")
    elif sha256(lockfile_path) != measurement["lockfile"]["after_sha256"]:
        findings.append("root lockfile SHA-256 mismatch")
    if measurement["lockfile"]["before_sha256"] != measurement["lockfile"]["after_sha256"]:
        findings.append("measurement records root lockfile drift")

    active_ops_manifests = sorted(LANE_ROOT.rglob("Cargo.toml"))
    if active_ops_manifests:
        findings.append(
            "active build manifests under ops/lane2c: "
            f"{[str(path.relative_to(LANE_ROOT)) for path in active_ops_manifests]}"
        )
    if measurement["harness"]["active_manifests_under_ops"] != 0:
        findings.append("measurement does not record zero active ops manifests")

    if not STRESS_PATH.is_file():
        findings.append("stress output missing")
    elif "test result: ok. 4 passed; 0 failed" not in STRESS_PATH.read_text(
        encoding="utf-8"
    ):
        findings.append("stress output does not record the required 4/4 pass")

    expected_paths = {
        Path(row["path"])
        for row in measurement["profiles"]
    }
    actual_paths = {
        path.relative_to(LANE_ROOT)
        for path in PROFILE_ROOT.glob("*/*.callgrind")
    }
    if actual_paths != expected_paths:
        findings.append(
            "profile set mismatch: "
            f"missing={sorted(map(str, expected_paths - actual_paths))}, "
            f"extra={sorted(map(str, actual_paths - expected_paths))}"
        )

    for row in measurement["profiles"]:
        relative = Path(row["path"])
        path = LANE_ROOT / relative
        if not path.is_file():
            continue
        profile = parse_profile(path)
        key = (row["case"], row["size"], row["phase"])
        observed[key] = int(profile["instructions"])
        command_suffix = f" {row['case']} {row['size']}"
        checks = {
            "creator": profile["creator"] == "callgrind-3.22.0",
            "event": profile["events"] == "Ir",
            "command": str(profile["command"]).endswith(command_suffix),
            "instructions": profile["instructions"] == row["instructions"],
            "sha256": profile["sha256"] == row["sha256"],
        }
        for name, passed in checks.items():
            if not passed:
                findings.append(f"{relative}: {name} mismatch")

    curves: list[dict[str, object]] = []
    for case in measurement["cases"]:
        sizes = measurement["sizes"]
        before = [observed.get((case, size, "before")) for size in sizes]
        after = [observed.get((case, size, "after")) for size in sizes]
        if any(value is None for value in before + after):
            findings.append(f"{case}: incomplete before/after curve")
            continue
        before_ratios = [
            before[index + 1] / before[index] for index in range(len(sizes) - 1)
        ]
        after_ratios = [
            after[index + 1] / after[index] for index in range(len(sizes) - 1)
        ]
        if any(ratio < 3.5 for ratio in before_ratios):
            findings.append(f"{case}: base curve is not demonstrably superlinear")
        if any(ratio > 2.5 for ratio in after_ratios):
            findings.append(f"{case}: fixed curve exceeds the linear-count ceiling")
        if any(fixed >= base for fixed, base in zip(after, before, strict=True)):
            findings.append(f"{case}: fixed count did not improve every size")
        curves.append(
            {
                "case": case,
                "before": before,
                "after": after,
                "before_doubling_ratios": [
                    round(value, 3) for value in before_ratios
                ],
                "after_doubling_ratios": [
                    round(value, 3) for value in after_ratios
                ],
            }
        )

    return {
        "schema": "garnet.lane2c.teardown-evidence-verdict/v1",
        "ok": not findings,
        "counter": "Callgrind Ir",
        "base_head": measurement["base_head"],
        "product_head": measurement["product_head"],
        "repository_bindings": {
            "shipped_harness_sha256": measurement["harness"]["source_sha256"],
            "root_lockfile_sha256": measurement["lockfile"]["after_sha256"],
            "active_ops_manifests": len(active_ops_manifests),
            "stress": "4/4",
        },
        "curves": curves,
        "findings": findings,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--gate", action="store_true")
    args = parser.parse_args()
    result = verify()
    print(json.dumps(result, indent=2))
    return 1 if args.gate and not result["ok"] else 0


if __name__ == "__main__":
    raise SystemExit(main())
