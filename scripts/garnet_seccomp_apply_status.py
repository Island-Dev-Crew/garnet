#!/usr/bin/env python3
"""OS-sandbox apply status (UTM seccomp slice).

S46 generates a seccomp policy but does not enforce it. This slice adds a reference
apply path and PROVES, on a real Linux kernel (the Mac's UTM Debian-12 ARM64 guest),
that the GENERATED policy is applied and deterministically traps a denied syscall
(`socket` under `@caps(fs)`), while `@caps(fs, net)` allows it (policy-driven). This
static gate asserts the harness, the reproduce script, the recorded proof, and the
honest scope stay in place.

## Honest scope (do not soften)
Linux seccomp only (macOS sandbox-exec / Windows AppContainer named-deferred). It
proves the GENERATED policy is enforceable, not that a program is "safe". The apply
path is a reference C harness (the proof VM has no Rust); a garnet-native Linux apply
path + applying to a spawned subprocess are follow-ups.
"""
from __future__ import annotations

import argparse
import json
import sys
from dataclasses import asdict, dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
HARNESS = ROOT / "tools" / "seccomp-apply" / "seccomp_apply.c"
PROVE = ROOT / "tools" / "seccomp-apply" / "prove.sh"
PROOF = ROOT / "tools" / "seccomp-apply" / "PROOF_utm_debian12_aarch64.txt"
DOC = ROOT / "C_Language_Specification" / "GARNET_SECCOMP_APPLY.md"


@dataclass
class SeccompApplyStatus:
    schema: str
    harness_present: bool
    applies_generated_policy: bool
    reproduce_script_present: bool
    proof_recorded: bool
    proof_deterministic: bool
    policy_driven: bool
    doc_present: bool
    honesty_anchor_present: bool
    ok: bool = False


def _read(p: Path) -> str:
    return p.read_text(encoding="utf-8") if p.is_file() else ""


def read_status() -> SeccompApplyStatus:
    h = _read(HARNESS)
    prove = _read(PROVE)
    proof = _read(PROOF)
    doc = _read(DOC)
    harness_present = bool(h)
    applies = (
        "SCMP_ACT_ERRNO" in h
        and "seccomp_load" in h
        and "socket(AF_INET" in h
    )
    reproduce = bool(prove) and "sandbox --format json" in prove and "seccomp_apply" in prove
    # The recorded proof must show the trap (BLOCKED + EPERM) and the PROVEN verdict.
    proof_recorded = (
        "PROVEN" in proof
        and "BLOCKED (trap)" in proof
        and "Operation not permitted" in proof
    )
    # Deterministic: 3 runs recorded (run 1/2/3 exit=0).
    proof_deterministic = (
        "run 1 exit=0" in proof
        and "run 2 exit=0" in proof
        and "run 3 exit=0" in proof
    )
    # Policy-driven: @caps(fs,net) allows socket (same harness, opposite result).
    policy_driven = "ALLOWED" in proof and "policy-driven" in proof
    doc_present = bool(doc)
    honesty = (
        "Linux seccomp only" in doc
        and "named-deferred" in doc
        and "not" in doc
        and 'safe' in doc
        and "generated, not enforced" in doc
    )
    ok = (
        harness_present
        and applies
        and reproduce
        and proof_recorded
        and proof_deterministic
        and policy_driven
        and doc_present
        and honesty
    )
    return SeccompApplyStatus(
        schema="garnet.seccomp_apply/v1",
        harness_present=harness_present,
        applies_generated_policy=applies,
        reproduce_script_present=reproduce,
        proof_recorded=proof_recorded,
        proof_deterministic=proof_deterministic,
        policy_driven=policy_driven,
        doc_present=doc_present,
        honesty_anchor_present=honesty,
        ok=ok,
    )


def render_markdown(r: SeccompApplyStatus) -> str:
    return "\n".join([
        "# Garnet OS-sandbox apply status (UTM seccomp)",
        "",
        f"_Schema {r.schema}._",
        "",
        f"- apply harness present (reads the generated policy): "
        f"{'yes' if r.harness_present and r.applies_generated_policy else 'NO'}",
        f"- reproduce script (`prove.sh`): {'yes' if r.reproduce_script_present else 'NO'}",
        f"- proof recorded on a real kernel (BLOCKED/EPERM/PROVEN): "
        f"{'yes' if r.proof_recorded else 'NO'}",
        f"- trap deterministic (3 runs): {'yes' if r.proof_deterministic else 'NO'}",
        f"- policy-driven (@caps(fs,net) allows socket): {'yes' if r.policy_driven else 'NO'}",
        f"- record doc + honesty anchors: "
        f"{'yes' if r.doc_present and r.honesty_anchor_present else 'NO'}",
        "",
        "S46 generated -> applied + trapped on a real Linux kernel (UTM Debian-12 "
        "ARM64). Linux seccomp only; macOS/Windows named-deferred; proves the "
        "generated policy is enforceable, not that a program is 'safe'.",
        "",
    ])


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument(
        "--gate",
        action="store_true",
        help="exit non-zero unless the apply harness, the reproduce script, the "
        "recorded deterministic + policy-driven proof, and the honest-scope doc "
        "are all present.",
    )
    args = parser.parse_args(list(argv) if argv is not None else None)

    r = read_status()
    print(render_markdown(r) if args.format == "md" else json.dumps(asdict(r), indent=2))

    if args.gate and not r.ok:
        print(f"seccomp-apply gate FAILED: {asdict(r)}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
