#!/usr/bin/env python3
"""M8 — drift guard for the Foundation + macOS Studio completion dossier.

The capstone dossier records the M0–M8 arc. This pins it to reality so it cannot
silently drift: it must record every merged slice's PR number, the seven
multi-pass catches, the green ladder, and the load-bearing honesty anchors
(enforced = @caps + @max_depth; the .app is unsigned/un-notarized; etc.).
"""
from __future__ import annotations

import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOSSIER = ROOT / "F_Project_Management" / "GARNET_STUDIO_TRACK_B_COMPLETION_DOSSIER_2026_06_28.html"


def read() -> str:
    return DOSSIER.read_text(encoding="utf-8")


class CompletionDossierTests(unittest.TestCase):
    def test_dossier_exists(self) -> None:
        self.assertTrue(DOSSIER.is_file(), "the M8 completion dossier must exist")

    def test_records_every_merged_slice_pr(self) -> None:
        text = read()
        for pr in ("#435", "#436", "#437", "#438", "#439", "#440", "#441",
                   "#442", "#443", "#444", "#445", "#446", "#447"):
            self.assertIn(pr, text, f"the dossier must record PR {pr}")
        for slice_id in ("A1", "A2", "A5", "A6", "M0a", "M0b", "M1", "M2",
                         "M3", "M4", "M5", "M6", "M7", "M8"):
            self.assertIn(f">{slice_id}<", text, f"the dossier must list slice {slice_id}")

    def test_records_the_seven_catches(self) -> None:
        text = read()
        # The dossier numbers each catch 1..7 inside an <h3>.
        for n in range(1, 8):
            self.assertIn(f"{n} ·", text, f"catch {n} must be recorded")
        # The headline catch — the real CI OOM root cause — must be named.
        self.assertIn("ancestorRoots", text)
        self.assertIn("26.6", text, "the OOM footprint evidence must be recorded")
        self.assertIn("Darwin 24", text, "the macOS-version divergence must be recorded")

    def test_honesty_anchors_are_preserved(self) -> None:
        text = read()
        self.assertIn("@caps", text)
        self.assertIn("@max_depth", text)
        self.assertIn("seccomp Linux-only", text)
        self.assertIn("declared-not-enforced", text)
        self.assertIn("unsigned", text)
        self.assertIn("un-notarized", text)
        self.assertIn("research-grade prototype", text)
        self.assertNotIn("production-ready", text)

    def test_integrity_rules_and_jon_owned_tag_recorded(self) -> None:
        text = read()
        self.assertIn("release tag", text.lower().replace("release tag remains jon-owned", "release tag"))
        self.assertIn("Jon", text, "the tag/ownership boundary must be stated")
        self.assertIn("agent/model/gate-version", text)


if __name__ == "__main__":
    unittest.main()
