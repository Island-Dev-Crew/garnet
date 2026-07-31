#!/usr/bin/env python3
"""Verify Lane 2C Callgrind and Memcheck evidence without wall-clock gates."""

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
MEMCHECK_ROOT = EVIDENCE_ROOT / "memcheck"
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


def parse_memcheck(path: Path) -> dict[str, object]:
    text = path.read_text(encoding="utf-8")

    def required(pattern: str, label: str) -> re.Match[str]:
        match = re.search(pattern, text, flags=re.MULTILINE)
        if match is None:
            raise ValueError(f"{path}: missing Memcheck {label}")
        return match

    command = required(r"^==\d+== Command:\s+(.*?)\s*$", "command").group(1)
    candidate = required(
        r"^candidate=([0-9a-f]+) case=([a-z-]+) size=(\d+) "
        r"operation_counting=false pid=\d+\s*$",
        "candidate binding",
    )
    losses: dict[str, dict[str, int]] = {}
    labels = {
        "definitely_lost": "definitely lost",
        "indirectly_lost": "indirectly lost",
        "possibly_lost": "possibly lost",
        "still_reachable": "still reachable",
    }
    for key, label in labels.items():
        match = required(
            rf"^\s*==\d+==\s+{label}:\s+([\d,]+) bytes in ([\d,]+) blocks\s*$",
            label,
        )
        losses[key] = {
            "bytes": int(match.group(1).replace(",", "")),
            "blocks": int(match.group(2).replace(",", "")),
        }
    errors = required(
        r"^==\d+== ERROR SUMMARY:\s+([\d,]+) errors from [\d,]+ contexts",
        "error summary",
    )
    return {
        "version": "Valgrind-3.22.0" in text,
        "command": command,
        "head": candidate.group(1),
        "case": candidate.group(2),
        "size": int(candidate.group(3)),
        "losses": losses,
        "errors": int(errors.group(1).replace(",", "")),
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

    memcheck_record = measurement["memcheck"]
    quiet_state = memcheck_record["quiet_state"]
    if quiet_state["required"] or quiet_state["ritual_performed"]:
        findings.append("Memcheck record incorrectly claims a quiet-state ritual")
    if quiet_state["claim"] != "none":
        findings.append("Memcheck record makes a quiet-state claim")
    if memcheck_record["size"] != 1024:
        findings.append("Memcheck record does not bind size 1024")
    expected_binary_provenance = {
        "base": (
            measurement["base_head"],
            measurement["harness"]["base_binary_sha256"],
        ),
        "product": (
            measurement["product_head"],
            measurement["harness"]["product_binary_sha256"],
        ),
    }
    for phase, (expected_head, expected_sha256) in expected_binary_provenance.items():
        provenance = memcheck_record["binary_provenance"][phase]
        if provenance["head"] != expected_head:
            findings.append(f"Memcheck {phase} binary head mismatch")
        if provenance["sha256"] != expected_sha256:
            findings.append(f"Memcheck {phase} binary SHA-256 mismatch")
        if not provenance["status"].startswith("reused original artifact"):
            findings.append(f"Memcheck {phase} binary is not recorded as reused")

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

    expected_memcheck_paths = {
        Path(row["path"]) for row in memcheck_record["captures"]
    }
    expected_memcheck_pairs = {
        (case, phase)
        for case in measurement["cases"]
        for phase in ("before", "after")
    }
    recorded_memcheck_pairs = {
        (row["case"], row["phase"]) for row in memcheck_record["captures"]
    }
    if (
        len(memcheck_record["captures"]) != 6
        or recorded_memcheck_pairs != expected_memcheck_pairs
    ):
        findings.append("Memcheck record is not the exact three-case before/after set")
    actual_memcheck_paths = {
        path.relative_to(LANE_ROOT)
        for path in MEMCHECK_ROOT.glob("*/*.memcheck.txt")
    }
    if actual_memcheck_paths != expected_memcheck_paths:
        findings.append(
            "Memcheck capture set mismatch: "
            f"missing={sorted(map(str, expected_memcheck_paths - actual_memcheck_paths))}, "
            f"extra={sorted(map(str, actual_memcheck_paths - expected_memcheck_paths))}"
        )

    memcheck_observed: dict[tuple[str, str], dict[str, dict[str, int]]] = {}
    memcheck_captures: list[dict[str, object]] = []
    for row in memcheck_record["captures"]:
        relative = Path(row["path"])
        path = LANE_ROOT / relative
        if not path.is_file():
            continue
        capture = parse_memcheck(path)
        phase = row["phase"]
        case = row["case"]
        expected_head = memcheck_record["binary_provenance"][
            "base" if phase == "before" else "product"
        ]["head"]
        checks = {
            "version": capture["version"],
            "command": str(capture["command"]).endswith(f" {case} {memcheck_record['size']}"),
            "head": capture["head"] == expected_head,
            "case": capture["case"] == case,
            "size": capture["size"] == memcheck_record["size"],
            "losses": capture["losses"]
            == {
                key: row[key]
                for key in (
                    "definitely_lost",
                    "indirectly_lost",
                    "possibly_lost",
                    "still_reachable",
                )
            },
            "errors": capture["errors"] == row["errors"] == 0,
            "exit_status": row["exit_status"] == 0,
            "sha256": capture["sha256"] == row["sha256"],
        }
        for name, passed in checks.items():
            if not passed:
                findings.append(f"{relative}: Memcheck {name} mismatch")
        memcheck_observed[(case, phase)] = capture["losses"]
        memcheck_captures.append(
            {
                "phase": phase,
                "case": case,
                "definitely_lost": row["definitely_lost"],
                "indirectly_lost": row["indirectly_lost"],
                "possibly_lost": row["possibly_lost"],
                "still_reachable": row["still_reachable"],
                "errors": row["errors"],
            }
        )

    memcheck_deltas: list[dict[str, object]] = []
    if (
        len(memcheck_record["deltas"]) != 3
        or {row["case"] for row in memcheck_record["deltas"]}
        != set(measurement["cases"])
    ):
        findings.append("Memcheck delta record is not the exact three-case set")
    for recorded_delta in memcheck_record["deltas"]:
        case = recorded_delta["case"]
        before = memcheck_observed.get((case, "before"))
        after = memcheck_observed.get((case, "after"))
        if before is None or after is None:
            findings.append(f"{case}: incomplete Memcheck before/after pair")
            continue
        computed: dict[str, dict[str, int]] = {}
        for key in (
            "definitely_lost",
            "indirectly_lost",
            "possibly_lost",
            "still_reachable",
        ):
            computed[key] = {
                "bytes": after[key]["bytes"] - before[key]["bytes"],
                "blocks": after[key]["blocks"] - before[key]["blocks"],
            }
            if after[key]["bytes"] > before[key]["bytes"]:
                findings.append(f"{case}: after Memcheck {key} bytes exceed before")
            if after[key]["blocks"] > before[key]["blocks"]:
                findings.append(f"{case}: after Memcheck {key} blocks exceed before")
        for key in ("definitely_lost", "indirectly_lost", "possibly_lost"):
            if before[key] != {"bytes": 0, "blocks": 0}:
                findings.append(f"{case}: base Memcheck reports {key}")
            if after[key] != {"bytes": 0, "blocks": 0}:
                findings.append(f"{case}: product Memcheck reports {key}")
        expected_delta = {
            key: recorded_delta[key]
            for key in (
                "definitely_lost",
                "indirectly_lost",
                "possibly_lost",
                "still_reachable",
            )
        }
        if computed != expected_delta:
            findings.append(f"{case}: recorded Memcheck delta mismatch")
        memcheck_deltas.append({"case": case, **computed})

    return {
        "schema": "garnet.lane2c.teardown-evidence-verdict/v2",
        "ok": not findings,
        "counter": "Callgrind Ir",
        "disposition_counter": "Valgrind Memcheck leak categories",
        "base_head": measurement["base_head"],
        "product_head": measurement["product_head"],
        "repository_bindings": {
            "shipped_harness_sha256": measurement["harness"]["source_sha256"],
            "base_binary_sha256": measurement["harness"]["base_binary_sha256"],
            "product_binary_sha256": measurement["harness"][
                "product_binary_sha256"
            ],
            "root_lockfile_sha256": measurement["lockfile"]["after_sha256"],
            "active_ops_manifests": len(active_ops_manifests),
            "stress": "4/4",
        },
        "curves": curves,
        "memcheck": {
            "size": memcheck_record["size"],
            "quiet_state_claim": quiet_state["claim"],
            "captures": memcheck_captures,
            "deltas": memcheck_deltas,
        },
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
