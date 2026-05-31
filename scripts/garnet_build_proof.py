#!/usr/bin/env python3
"""Windows / Linux / macOS build-proof reporter + cross-OS coverage gate (S47).

The reconciliation's Windows-propriety question is: *does every attribute behave
correctly cross-platform, or is it just packaging?* This reporter answers it from
the CI matrix, distinguishing two axes per OS:

- **behaves** — the OS is in the `cargo test --workspace` matrix in
  `.github/workflows/ci.yml`, i.e. the toolchain compiles **and** the full test
  suite passes on that OS (the substantive proof, not just a build).
- **distributes** — a packaging/artifact job exists for that OS family (CLI
  tarball, deb/rpm). This is the "just packaging" axis; its absence is reported,
  not gated.

**Honest scope:** this is a single-OS checkout. The script does **not** run
Windows/Linux builds — it verifies the *CI matrix* covers them and gates against
silent regression of that coverage. The actual cross-OS execution is attested by
CI, not by this script. `--gate` exits non-zero if any target OS lacks the
"behaves" proof.
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Sequence

ROOT = Path(__file__).resolve().parents[1]

TARGET_OSES = ["ubuntu-latest", "windows-latest", "macos-latest"]
OS_LABELS = {
    "ubuntu-latest": "Linux",
    "windows-latest": "Windows",
    "macos-latest": "macOS",
}
WORKSPACE_TEST = "cargo test --workspace"


@dataclass
class OsProof:
    os: str
    label: str
    behaves: bool
    distributes: bool
    distribution_evidence: str


@dataclass
class BuildProof:
    schema: str
    workspace_test_command: str
    oses: list[OsProof]
    all_behaves: bool
    notes: list[str]


def _read(rel: str) -> str:
    path = ROOT / rel
    return path.read_text(encoding="utf-8") if path.is_file() else ""


def _matrix_oses(ci_yaml: str) -> set[str]:
    """Union of every `os: [ ... ]` array declared in ci.yml (the test job's
    matrix is the one that lists all three target OSes)."""
    found: set[str] = set()
    for match in re.finditer(r"os:\s*\[([^\]]*)\]", ci_yaml):
        for token in match.group(1).split(","):
            name = token.strip().strip("\"'")
            if name:
                found.add(name)
    return found


def _distribution_evidence(os_name: str) -> tuple[bool, str]:
    """Packaging/artifact evidence per OS family (reported, not gated)."""
    # The cross-OS packaging jobs all live in linux-packages.yml (deb/rpm smoke
    # plus the macOS CLI tarballs), despite the file name.
    pkg = _read(".github/workflows/linux-packages.yml")
    if os_name == "macos-latest":
        present = "macos-cli-tarballs" in pkg or "apple-darwin" in pkg
        return present, "linux-packages.yml (macos-cli-tarballs)" if present else "none"
    if os_name == "ubuntu-latest":
        present = "smoke-deb" in pkg or "smoke-rpm" in pkg
        return present, "linux-packages.yml (deb/rpm smoke)" if present else "none"
    if os_name == "windows-latest":
        # The Windows *Studio* installer is tracked separately; there is no
        # Windows CLI distribution artifact today. Report honestly.
        present = False
        return present, "none (Windows CLI packaging not yet wired; Studio installer is separate)"
    return False, "none"


def read_build_proof() -> BuildProof:
    ci = _read(".github/workflows/ci.yml")
    matrix = _matrix_oses(ci)
    has_workspace_test = WORKSPACE_TEST in ci

    oses: list[OsProof] = []
    for os_name in TARGET_OSES:
        behaves = (os_name in matrix) and has_workspace_test
        distributes, evidence = _distribution_evidence(os_name)
        oses.append(
            OsProof(
                os=os_name,
                label=OS_LABELS[os_name],
                behaves=behaves,
                distributes=distributes,
                distribution_evidence=evidence,
            )
        )

    notes = [
        "CI-attested: this reporter verifies the CI matrix; it does not run "
        "Windows/Linux builds locally (single-OS checkout).",
        "'behaves' = compiles + `cargo test --workspace` passes on that OS in ci.yml.",
        "The S46 sandbox policy is Linux-syscall-shaped (seccomp); it is generated "
        "on every OS but only meaningful for a Linux enforcement host.",
        "Determinism is independently gated by determinism.yml (Cross-OS determinism).",
    ]
    return BuildProof(
        schema="garnet.build_proof/v1",
        workspace_test_command=WORKSPACE_TEST,
        oses=oses,
        all_behaves=all(p.behaves for p in oses),
        notes=notes,
    )


def render_markdown(proof: BuildProof) -> str:
    lines = [
        "# Garnet build proof (Windows / Linux / macOS)",
        "",
        f"_Schema {proof.schema}. CI-attested; not locally re-run._",
        "",
        "| OS | behaves (cargo test --workspace) | distributes |",
        "|---|---|---|",
    ]
    for p in proof.oses:
        behaves = "✅" if p.behaves else "❌"
        dist = "✅" if p.distributes else "—"
        lines.append(f"| {p.label} | {behaves} | {dist} ({p.distribution_evidence}) |")
    lines.append("")
    lines.append(f"**All target OSes behave: {'yes' if proof.all_behaves else 'NO'}.**")
    lines.append("")
    for note in proof.notes:
        lines.append(f"- {note}")
    lines.append("")
    return "\n".join(lines)


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument(
        "--gate",
        action="store_true",
        help="exit non-zero if any target OS lacks the 'behaves' proof",
    )
    args = parser.parse_args(list(argv) if argv is not None else None)

    proof = read_build_proof()
    if args.format == "md":
        print(render_markdown(proof))
    else:
        print(json.dumps(asdict(proof), indent=2))

    if args.gate and not proof.all_behaves:
        missing = [p.label for p in proof.oses if not p.behaves]
        print(
            f"build-proof gate FAILED: missing cross-OS test coverage for {', '.join(missing)}",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
