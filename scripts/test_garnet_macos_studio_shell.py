#!/usr/bin/env python3
"""Contract tests for the macOS SwiftUI Studio shell.

The macOS port of the PR #391 Studio shell standard (see
F_Project_Management/GARNET_STUDIO_SUITE_HANDOFF_2026_06_12.md §1). This is the
macOS sibling of scripts/test_garnet_windows_linux_studio_shell.py: it ports
the *standard*, not the Tauri code, so the assertions are behavioral-parity
markers over the Swift sources rather than Tauri config checks.

Row 1 is the load-bearing gate: the Swift package cannot inherit the workspace
version, so the StudioVersion stamp == [workspace.package].version assertion in
this file IS the version-sync gate for the macOS app.
"""
from __future__ import annotations

import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
APP = ROOT / "apps" / "garnet-studio-macos"
SHELL = APP / "Sources" / "GarnetStudio" / "StudioShell.swift"
CHROME = APP / "Sources" / "GarnetStudio" / "StudioChrome.swift"
MAIN = APP / "Sources" / "GarnetStudio" / "GarnetStudioApp.swift"
TESTS = APP / "Tests" / "GarnetStudioTests" / "StudioShellTests.swift"
DIFFCAPS_CMD = APP / "Sources" / "GarnetStudio" / "DiffCapsCommand.swift"
VELOCITY_CMD = APP / "Sources" / "GarnetStudio" / "VelocityCheckCommand.swift"


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


class GarnetMacosStudioShellTests(unittest.TestCase):
    # ── Row 1 · Version truth ────────────────────────────────────────────

    def test_version_stamp_is_single_sourced_and_tracks_the_workspace(self) -> None:
        shell = read(SHELL)
        stamp = None
        for line in shell.splitlines():
            stripped = line.strip()
            if stripped.startswith('public static let release = "'):
                stamp = stripped.split('"')[1]
                break
        self.assertIsNotNone(stamp, "StudioVersion.release must declare the single stamp")
        self.assertNotEqual("0.1.0", stamp, "the 0.1.0 stamp drift must not return")

        workspace = read(ROOT / "Cargo.toml")
        workspace_version = None
        in_section = False
        for line in workspace.splitlines():
            stripped = line.strip()
            if stripped.startswith("["):
                in_section = stripped == "[workspace.package]"
                continue
            if in_section and stripped.startswith('version = "'):
                workspace_version = stripped.split('"')[1]
                break
        self.assertIsNotNone(workspace_version, "workspace.package version must exist")
        self.assertEqual(
            workspace_version,
            stamp,
            "the Swift package cannot inherit the workspace version, so this "
            "assertion is the version-sync gate between the macOS Studio stamp "
            "and the release version",
        )

        # No second hand-stamped version constant anywhere in the app sources.
        for source in (MAIN, CHROME):
            self.assertNotIn(
                'let appVersion = "',
                read(source),
                f"{source.name} must not reintroduce a second version stamp",
            )

    # ── Row 2 · Launch experience ────────────────────────────────────────

    def test_splash_holds_with_live_status_and_hard_ceiling(self) -> None:
        shell = read(SHELL)
        chrome = read(CHROME)
        self.assertIn("minimumMilliseconds = 700", shell)
        self.assertIn("ceilingMilliseconds = 25_000", shell)
        self.assertIn("SplashView", chrome)
        self.assertIn("statusMessage", chrome)
        self.assertIn("Rust Rigor. Ruby Velocity. One Coherent Language.", chrome)
        self.assertIn("reduceMotion", chrome)

    # ── Row 3 · Simple/Power modes ───────────────────────────────────────

    def test_interface_modes_are_persisted_and_power_only_sections_remain(self) -> None:
        main = read(MAIN)
        self.assertIn('@AppStorage("studio.interfaceMode")', main)
        self.assertIn('"simple"', main)
        self.assertIn('"power"', main)
        self.assertIn("// power-only", main)
        # The power-only sections stay compiled into the source — hidden by
        # mode, never removed (the macOS analog of "panels stay in the DOM").
        self.assertIn("Agentic Stress Matrix", main)
        self.assertIn("Release Evidence", main)

    # ── Row 4 · Validated settings ───────────────────────────────────────

    def test_settings_are_validated_clamped_and_boot_safe(self) -> None:
        shell = read(SHELL)
        self.assertIn("func normalized()", shell)
        self.assertIn("commandTimeoutSecs", shell)
        self.assertIn("matrixTimeoutSecs", shell)
        self.assertIn("decodeTolerantly", shell)
        self.assertIn("never block", shell.replace("\n", " "))
        tests = read(TESTS)
        self.assertIn("testCorruptSettingsFileNeverBlocksBoot", tests)

    # ── Row 5 · Process discipline ───────────────────────────────────────

    def test_every_spawn_goes_through_the_disciplined_runner(self) -> None:
        shell = read(SHELL)
        main = read(MAIN)
        self.assertIn("timedOut", shell)
        self.assertIn("uiPayloadByteCap", shell)
        self.assertIn("killProcessTree", shell)
        self.assertIn("SIGKILL", shell)
        self.assertIn("durationSeconds", shell)
        self.assertIn("output capped for UI display", shell)
        # No raw Process()+waitUntilExit spawn paths may remain in the app
        # layer; everything routes through StudioProcessRunner.
        self.assertNotIn("let process = Process()", main)
        self.assertIn("StudioProcessRunner.runBridged", main)
        # The agentic matrix gets the larger budget.
        self.assertIn("category: .matrix", main)
        tests = read(TESTS)
        self.assertIn("testRunnerReportsTimedOutAndKillsTheProcess", tests)
        self.assertIn("testRunnerDrainsLargeOutputWithoutDeadlockAndCapsUI", tests)

    # ── Row 6 · Truth surface ────────────────────────────────────────────

    def test_truth_tiles_replace_hand_written_release_stats(self) -> None:
        shell = read(SHELL)
        chrome = read(CHROME)
        main = read(MAIN)
        self.assertIn("docs/truth.json", shell)
        self.assertIn("unavailable", shell)
        self.assertIn("TruthTilesPanel", chrome)
        self.assertIn("Truth surface unavailable", chrome)
        self.assertIn("TruthTilesPanel(truth:", main)
        # Hand-written release statistics must not return.
        self.assertNotIn("87/87 slices complete", main)
        self.assertNotIn("v0.4.2 published", main)

    # ── Row 7 · Hover help everywhere ────────────────────────────────────

    def test_hover_help_covers_the_surface(self) -> None:
        total = sum(read(p).count(".help(") for p in (MAIN, CHROME, SHELL))
        self.assertGreaterEqual(
            total,
            30,
            "every control should explain itself with claim-boundary-honest copy",
        )

    # ── Row 8 · Evidence readers ─────────────────────────────────────────

    def test_evidence_readers_are_read_only_and_root_constrained(self) -> None:
        shell = read(SHELL)
        self.assertIn("resolveWithinEvidenceRoots", shell)
        self.assertIn("outside the Studio evidence roots", shell)
        self.assertIn("isSymlink", shell)
        self.assertIn("maxReadBytes", shell)
        self.assertIn("maxEntries", shell)
        tests = read(TESTS)
        self.assertIn("testEvidenceReaderRefusesPathsOutsideRootsAndSymlinks", tests)

    # ── Row 9 · Keyboard, status bar, themes, a11y ───────────────────────

    def test_keyboard_status_bar_themes_and_a11y_are_present(self) -> None:
        main = read(MAIN)
        chrome = read(CHROME)
        self.assertIn("keyboardShortcut", main)
        self.assertIn('CommandMenu("Go")', main)
        self.assertIn("StudioStatusBar", chrome)
        self.assertIn("preferredColorScheme", main)
        self.assertIn('@AppStorage("studio.theme")', main)
        self.assertIn("accessibilityReduceMotion", main)
        self.assertIn("accessibilityLabel", chrome)
        # Native Settings scene (⌘,) — the macOS-exceeding affordance.
        self.assertIn("Settings {", main)
        self.assertIn("StudioSettingsView", main)

    # ── Boundaries ───────────────────────────────────────────────────────

    def test_shell_boundaries_hold_no_provider_api_path(self) -> None:
        for path in (SHELL, CHROME, MAIN):
            source = read(path)
            self.assertNotIn(
                "URLSession",
                source,
                f"{path.name}: the Studio shell has no network call path; a "
                "provider/network surface needs a contract amendment first",
            )
            self.assertNotIn("api.openai.com", source)
            self.assertNotIn("api.anthropic.com", source)

    def test_honest_claim_boundaries_survive(self) -> None:
        main = read(MAIN)
        self.assertIn("provider-backed conversion is not active", main)
        self.assertIn("Deferred", main)
        self.assertIn("research-grade prototype", main)
        self.assertNotIn("production-ready", main)

    def test_diff_caps_review_panel_is_wired_power_only_and_verbatim(self) -> None:
        # M2: the Diff-Caps Review Gate is a power-only section that renders the
        # CLI's machine verdict verbatim (never recomputes the band).
        main = read(MAIN)
        self.assertIn('case diffCaps = "Diff-Caps Review"', main, "diff-caps section must be wired")
        self.assertIn("$0 != .diffCaps", main, "diff-caps must be a power-only section")
        cmd = read(DIFFCAPS_CMD)
        self.assertIn("diff-caps", cmd)
        self.assertIn("--machine", cmd, "must use the machine verdict, not human output")
        self.assertIn("never recomputed", cmd, "the band/verdict must be rendered verbatim, not recomputed")
        # The stdout/stderr-merge gotcha fix: the runner must slice the JSON object
        # out of the merged stream (episodic `note:` lines on stderr would otherwise
        # break the decode). Behavior is exercised by the M0b decoder tests; this
        # pins the slicing logic stays present.
        self.assertIn("extractJSONObject", cmd)
        self.assertIn('firstIndex(of: "{")', cmd, "must slice from the first JSON brace")

    def test_velocity_editor_panel_is_wired_power_only_and_cwd_isolated(self) -> None:
        # M3: the Velocity Editor is a power-only section that runs
        # `garnet check --format json` over the live buffer and isolates the
        # checker's .garnet-cache side-effect to a throwaway working directory.
        main = read(MAIN)
        self.assertIn('case velocity = "Velocity Editor"', main, "velocity section must be wired")
        self.assertIn("$0 != .velocity", main, "velocity must be a power-only section")
        cmd = read(VELOCITY_CMD)
        self.assertIn("check", cmd)
        self.assertIn("--format", cmd)
        self.assertIn("json", cmd, "must use the machine JSON diagnostics, not human output")
        # The .garnet-cache isolation: a unique temp dir is created and passed as
        # the process working directory, then removed.
        self.assertIn("workingDirectory", cmd, "the check must run in an isolated cwd")
        self.assertIn("temporaryDirectory", cmd)
        self.assertIn("removeItem", cmd, "the throwaway dir must be cleaned up")
        self.assertIn(".garnet-cache", cmd, "the cwd-isolation rationale must be documented")
        # The JSON-slicing gotcha (merged stdout+stderr) must be present.
        self.assertIn("extractJSONObject", cmd)
        self.assertIn('firstIndex(of: "{")', cmd, "must slice from the first JSON brace")

    def test_converter_help_makes_no_os_sandbox_overclaim(self) -> None:
        # M1 honesty-cleanup: the Convert action help once claimed "sandboxed
        # output" while the converter writes plain files to the local evidence
        # dir with NO OS sandbox. The overclaim must not return. (A future M4
        # "declared-not-enforced" deferred-row label is a different surface; this
        # negative contract targets the "sandboxed"/"sandbox" overclaim wording.)
        main = read(MAIN)
        self.assertNotIn("sandboxed", main, "the 'sandboxed output' converter overclaim must not return")
        self.assertNotIn("sandbox", main, "no Studio copy may claim an OS sandbox the converter does not provide")
        self.assertIn("Active conversion", main, "Convert help must still describe active conversion honestly")


if __name__ == "__main__":
    unittest.main()
