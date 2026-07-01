#!/usr/bin/env python3
"""Native ARM64 Linux Studio (Tauri) build/install/launch status (Linux lane L3).

Verifies the RECORDED proof that the Tauri Garnet Studio builds a .deb, installs
as a system package, passes its non-GUI --studio-smoke, and launches (headless
under xvfb) on a NATIVE (non-WSL) desktop Debian ARM64 host. The pre-existing
Linux Studio smokes are WSL/WSLg and explicitly disclaim being a non-WSL desktop
proof; this gate covers that gap. GitHub CI cannot drive the UTM VM, so — like
the seccomp-apply and WSL smokes — this is a status gate over recorded evidence.
"""
from __future__ import annotations

import argparse
import json
import sys
from dataclasses import asdict, dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PROOF = (
    ROOT / "proofs" / "linux" / "studio" / "utm-native-20260701"
    / "garnet-studio-native-linux.json"
)


@dataclass
class Status:
    schema_ok: bool
    build_ok: bool
    install_ok: bool
    smoke_ok: bool
    launch_ok: bool
    honesty_ok: bool
    ok: bool


def evaluate() -> Status:
    data = json.loads(PROOF.read_text(encoding="utf-8")) if PROOF.is_file() else {}
    build = data.get("build", {})
    inst = data.get("install", {})
    smoke = data.get("smoke", {})
    launch = data.get("gui_launch", {})
    honesty = data.get("honesty", "")

    schema_ok = data.get("schema") == "garnet.studio.native_linux.v1"
    build_ok = build.get("tauri_rc") == 0 and str(build.get("deb", "")).endswith(".deb")
    install_ok = (
        inst.get("package") == "garnet-studio"
        and inst.get("installed_bin") == "/usr/bin/garnet-studio"
        and inst.get("dpkg_status") == "install ok installed"
        and ".desktop" in str(inst.get("desktop_entry", ""))
    )
    smoke_ok = (
        smoke.get("exit") == 0
        and smoke.get("status") == "passed"
        and smoke.get("platform") == "linux"
        and smoke.get("provider_api_called") is False
    )
    # A successful headless launch: the app ran until the timeout (exit 124), i.e.
    # it did NOT crash on GTK/WebKit init.
    launch_ok = launch.get("exit") == 124
    # Honesty anchors: native/non-WSL, closes the WSL gap, software-render caveat,
    # not signed, research-grade.
    hl = honesty.lower()
    honesty_ok = (
        "non-wsl" in hl
        and "wsl" in hl
        and "software rendering" in hl
        and "not signed" in hl
        and "research-grade prototype" in honesty
    )
    ok = all([schema_ok, build_ok, install_ok, smoke_ok, launch_ok, honesty_ok])
    return Status(schema_ok, build_ok, install_ok, smoke_ok, launch_ok, honesty_ok, ok)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--gate", action="store_true", help="exit non-zero if the proof is incomplete")
    args = ap.parse_args()
    st = evaluate()
    print(json.dumps(asdict(st), indent=2))
    if args.gate and not st.ok:
        print("native-linux-studio gate FAILED", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
