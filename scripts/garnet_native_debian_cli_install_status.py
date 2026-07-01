#!/usr/bin/env python3
"""Native ARM64 Debian CLI clean-install status (Linux lane L2).

Verifies the RECORDED proof that Garnet's `.deb` builds, installs as a system
package on a native (non-WSL) Debian ARM64 host, and that the installed binary
enforces the @caps kernel — undeclared `fs` use is flagged at check time
(check.caps_coverage) AND trapped at runtime. The existing Linux Studio/CLI
proofs are WSL-based and explicitly disclaim being a non-WSL desktop proof; this
gate covers exactly that gap. GitHub CI cannot drive the UTM VM, so — like the
seccomp-apply and WSL smokes — this is a status gate over recorded evidence.
"""
from __future__ import annotations

import argparse
import json
import sys
from dataclasses import asdict, dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PROOF = (
    ROOT / "proofs" / "linux" / "cli-install" / "utm-native-20260701"
    / "garnet-cli-native-debian-install.json"
)


@dataclass
class Status:
    schema_ok: bool
    package_installed: bool
    version_ok: bool
    clean_ok: bool
    static_trap_ok: bool
    runtime_trap_ok: bool
    honesty_ok: bool
    ok: bool


def evaluate() -> Status:
    data = json.loads(PROOF.read_text(encoding="utf-8")) if PROOF.is_file() else {}
    pkg = data.get("package", {})
    smokes = {s.get("id"): s for s in data.get("smokes", [])}
    honesty = data.get("honesty", "")

    schema_ok = data.get("schema") == "garnet.cli.native_debian_install.v1"
    package_installed = (
        pkg.get("name") == "garnet"
        and pkg.get("installed_path") == "/usr/bin/garnet"
        and pkg.get("dpkg_status") == "install ok installed"
        and str(pkg.get("version", "")).startswith("0.8.1")
    )
    version_ok = smokes.get("version", {}).get("exit") == 0
    clean_ok = (
        smokes.get("check_clean", {}).get("exit") == 0
        and smokes.get("run_clean", {}).get("exit") == 0
    )
    # The load-bearing part: undeclared authority is flagged statically AND trapped
    # at runtime by the installed binary.
    cv = smokes.get("check_violate", {})
    static_trap_ok = cv.get("exit") == 1 and cv.get("code") == "check.caps_coverage"
    rv = smokes.get("run_violate", {})
    runtime_trap_ok = rv.get("exit") == 1 and "requires @caps(fs)" in rv.get("trap", "")
    # Honesty anchors: native/non-WSL, enforced in-process, NOT signed, NOT a
    # seccomp OS-sandbox proof, research-grade.
    honesty_ok = (
        "non-WSL" in honesty
        and "enforced" in honesty.lower()
        and "not a seccomp os-sandbox proof" in honesty.lower()
        and "research-grade prototype" in honesty
    )
    ok = all(
        [
            schema_ok,
            package_installed,
            version_ok,
            clean_ok,
            static_trap_ok,
            runtime_trap_ok,
            honesty_ok,
        ]
    )
    return Status(
        schema_ok,
        package_installed,
        version_ok,
        clean_ok,
        static_trap_ok,
        runtime_trap_ok,
        honesty_ok,
        ok,
    )


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--gate", action="store_true", help="exit non-zero if the proof is incomplete")
    args = ap.parse_args()
    st = evaluate()
    print(json.dumps(asdict(st), indent=2))
    if args.gate and not st.ok:
        print("native-debian-cli-install gate FAILED", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
