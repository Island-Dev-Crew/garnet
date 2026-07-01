#!/usr/bin/env python3
"""Drift guard for the native Linux lane completion dossier (L4).

Pins the capstone dossier to reality: it must record every merged lane slice's
PR number, the three native-ARM64 status gates, the native/non-WSL framing, and
the load-bearing honesty anchors (seccomp generation + apply-proof; unsigned
packages; software-render launch; @caps enforced; research-grade). Also asserts
the three status-gate scripts the dossier cites actually exist.
"""
from __future__ import annotations

import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOSSIER = ROOT / "F_Project_Management" / "GARNET_LINUX_LANE_COMPLETION_DOSSIER_2026_07_01.html"
GATES = [
    "scripts/garnet_seccomp_apply_status.py",
    "scripts/garnet_native_debian_cli_install_status.py",
    "scripts/garnet_native_linux_studio_status.py",
]


def read() -> str:
    return DOSSIER.read_text(encoding="utf-8")


class LinuxLaneDossierTests(unittest.TestCase):
    def test_dossier_exists(self) -> None:
        self.assertTrue(DOSSIER.is_file(), "the L4 Linux lane dossier must exist")

    def test_records_every_slice_pr(self) -> None:
        text = read()
        for pr in ("#449", "#450", "#451"):
            self.assertIn(pr, text, f"the dossier must record PR {pr}")
        for slice_id in ("L1", "L2", "L3", "L4"):
            self.assertIn(f">{slice_id}<", text, f"the dossier must list slice {slice_id}")

    def test_cited_status_gates_exist(self) -> None:
        for gate in GATES:
            self.assertTrue((ROOT / gate).is_file(), f"cited gate missing: {gate}")
            self.assertIn(Path(gate).name, read(), f"dossier must cite {gate}")

    def test_native_non_wsl_framing_and_env(self) -> None:
        text = read()
        self.assertIn("non-WSL", text, "the native/non-WSL distinction is the whole point")
        self.assertIn("UTM Debian-12", text)
        self.assertIn("6.1.0-13-arm64", text, "the proven kernel must be recorded")
        self.assertIn("aarch64", text)

    def test_honesty_anchors_preserved(self) -> None:
        text = read()
        low = text.lower()
        self.assertIn("generation-only", low.replace("generation only", "generation-only"))
        self.assertIn("unsigned", low)
        self.assertIn("software render", low.replace("software-render", "software render"))
        self.assertIn("@caps", text)
        self.assertIn("research-grade prototype", text)
        self.assertNotIn("production-ready", text)
        # The integrity nuance: the deb-target promotion was NOT self-merged.
        self.assertIn("Jon-gated follow-up", text)
        self.assertIn("did not modify the gate it merges under".replace(
            "did not modify", "no PR modified"), text)


if __name__ == "__main__":
    unittest.main()
