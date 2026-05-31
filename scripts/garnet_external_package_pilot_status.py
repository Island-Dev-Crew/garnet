#!/usr/bin/env python3
"""External package pilot status (S77).

Drives Garnet's external-package flow end-to-end against the filesystem registry
stub: publish → resolve by name+version → BLAKE3 content-address verify → refuse a
nonexistent dependency → flag a hallucinated near-miss (slopsquatting guard). The
runnable proof is `garnet-registry-stub/tests/external_package_pilot.rs`, which
runs in the `cargo test --workspace` matrix on every OS.

This reporter is a static anti-overclaim gate (the agent-contracts CI job builds
no compiler): it verifies the pilot test exists, the registry-stub infrastructure
(`build_index`/`resolve`/`verify_package` + the slopguard) is present, and the
honest-scope doc exists. The binary-backed proof is the cargo matrix.

## Honest scope (do not soften)
A LOCAL filesystem registry-stub pilot, NOT a live public ecosystem: no HTTP, no
publish/auth, no SemVer ranges, no signatures. The slopguard is a deterministic
heuristic ("prompt to verify"), not a security guarantee.
"""
from __future__ import annotations

import argparse
import json
import sys
from dataclasses import asdict, dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PILOT_TEST = ROOT / "garnet-registry-stub" / "tests" / "external_package_pilot.rs"
REGISTRY_LIB = ROOT / "garnet-registry-stub" / "src" / "lib.rs"
SLOPGUARD = ROOT / "garnet-registry-stub" / "src" / "slopguard.rs"
DOC = ROOT / "C_Language_Specification" / "GARNET_EXTERNAL_PACKAGE_PILOT.md"

# The pilot test must exercise these flow stages (substring markers).
PILOT_MARKERS = [
    "external_package_resolves_and_verifies",
    "tampered_external_package_fails_verification",
    "nonexistent_dependency_is_refused",
    "slopguard_flags_hallucinated_near_miss",
]


@dataclass
class PilotStatus:
    schema: str
    pilot_test_present: bool
    missing_markers: list[str]
    registry_infra_present: bool
    slopguard_present: bool
    doc_present: bool
    ok: bool = False
    groundings: list[str] = field(default_factory=list)


def _read(p: Path) -> str:
    return p.read_text(encoding="utf-8") if p.is_file() else ""


def read_status() -> PilotStatus:
    test = _read(PILOT_TEST)
    lib = _read(REGISTRY_LIB)
    slop = _read(SLOPGUARD)
    missing = [m for m in PILOT_MARKERS if m not in test]
    infra = all(s in lib for s in ("pub fn resolve", "pub fn verify_package", "pub fn build_index"))
    slopguard = "pub fn nearest" in slop
    test_present = bool(test) and not missing
    doc_present = DOC.is_file()
    ok = test_present and infra and slopguard and doc_present
    return PilotStatus(
        schema="garnet.external_package_pilot/v1",
        pilot_test_present=test_present,
        missing_markers=missing,
        registry_infra_present=infra,
        slopguard_present=slopguard,
        doc_present=doc_present,
        ok=ok,
        groundings=[
            f"registry resolve/verify/build in lib.rs: {infra}",
            f"slopguard nearest() present: {slopguard}",
        ],
    )


def render_markdown(r: PilotStatus) -> str:
    lines = [
        "# Garnet external package pilot status (S77)",
        "",
        f"_Schema {r.schema}._",
        "",
        f"- pilot test present + complete: {'yes' if r.pilot_test_present else 'NO'}"
        + (f" (missing: {r.missing_markers})" if r.missing_markers else ""),
        f"- registry infra (build_index/resolve/verify_package): {'yes' if r.registry_infra_present else 'NO'}",
        f"- slopguard (slopsquatting heuristic): {'yes' if r.slopguard_present else 'NO'}",
        f"- honest-scope doc present: {'yes' if r.doc_present else 'NO'}",
        "",
        "The pilot resolves an external package, BLAKE3-verifies it, detects "
        "tampering, refuses a nonexistent dependency, and flags a hallucinated "
        "near-miss (slopsquatting). Honest scope: a LOCAL filesystem registry-stub "
        "pilot, NOT a live public ecosystem; the slopguard is a heuristic, not a "
        "security guarantee.",
        "",
    ]
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument(
        "--gate",
        action="store_true",
        help="exit non-zero if the pilot test / registry infra / slopguard / doc "
        "is missing. The binary-backed run is the cargo test matrix.",
    )
    args = parser.parse_args(list(argv) if argv is not None else None)

    r = read_status()
    if args.format == "md":
        print(render_markdown(r))
    else:
        print(json.dumps(asdict(r), indent=2))

    if args.gate and not r.ok:
        print(
            "external-package-pilot gate FAILED: "
            f"test={r.pilot_test_present} missing={r.missing_markers} "
            f"infra={r.registry_infra_present} slopguard={r.slopguard_present} doc={r.doc_present}",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
