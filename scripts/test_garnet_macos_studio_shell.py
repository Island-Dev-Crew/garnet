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
LEGEND_BRIDGE = APP / "Sources" / "GarnetStudio" / "EnforcementLegendBridge.swift"
AGENTLOOP_BRIDGE = APP / "Sources" / "GarnetStudio" / "AgentLoopBridge.swift"
AGENTLOOP_CMD = APP / "Sources" / "GarnetStudio" / "AgentLoopCommand.swift"
BOOTSTRAP_BRIDGE = APP / "Sources" / "GarnetStudio" / "BootstrapBridge.swift"
BOOTSTRAP_CMD = APP / "Sources" / "GarnetStudio" / "BootstrapCommand.swift"
DISTRIBUTION_BRIDGE = APP / "Sources" / "GarnetStudio" / "DistributionBridge.swift"


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

    def test_enforced_declared_legend_boundary_is_not_widened(self) -> None:
        # M4: the legend is a load-bearing honesty surface. Pin the
        # enforced-vs-declared boundary so a future edit cannot silently widen it:
        # ONLY @caps + @max_depth are enforced; @bounded/@mailbox/memory/time are
        # declared; the OS sandbox is deferred and seccomp is Linux-only.
        main = read(MAIN)
        self.assertIn('case legend = "Enforced / Declared"', main, "legend section must be wired")
        self.assertIn("$0 != .legend", main, "legend must be a power-only section")
        bridge = read(LEGEND_BRIDGE)
        # The two enforced fences each carry status .enforced.
        for enforced in ('name: "@caps"', 'name: "@max_depth"'):
            self.assertIn(enforced, bridge)
        self.assertEqual(
            bridge.count("status: .enforced"),
            2,
            "exactly two fences may be .enforced (@caps + @max_depth) — boundary not widened",
        )
        # The named-deferred fences stay declared, never enforced.
        for declared in ('name: "@bounded"', 'name: "@mailbox"', 'name: "memory"', 'name: "time"'):
            self.assertIn(declared, bridge)
        # The OS sandbox is deferred and seccomp is Linux-only.
        self.assertIn("status: .deferred", bridge)
        self.assertIn("Linux seccomp only", bridge)
        self.assertIn("do not apply an OS sandbox", bridge)
        # Confirmed-live only when the probe reproduced (no faked green).
        self.assertIn("confirmed live this run", bridge)

    def test_agent_loop_console_is_wired_power_only_and_verdict_verbatim(self) -> None:
        # M5: the agent-loop console renders an existing --record-dir as a 4-gate
        # pipeline. The verdict is read verbatim from decision.md (never recomputed);
        # acceptance is "on capability + depth evidence" only; seal provenance is
        # autonomous acceptance, NOT a human approval.
        main = read(MAIN)
        self.assertIn('case agentLoop = "Agent-Loop Console"', main, "agent-loop section must be wired")
        self.assertIn("$0 != .agentLoop", main, "agent-loop must be a power-only section")
        bridge = read(AGENTLOOP_BRIDGE)
        cmd = read(AGENTLOOP_CMD)
        # The reader pulls the record-dir artifacts (the 4-gate pipeline inputs).
        for artifact in ("decision.md", "diff_caps.txt", "capability_manifest.json",
                         "seal.json", "transparency_log.jsonl", "run_trap.txt"):
            self.assertIn(artifact, cmd, f"the console must read {artifact}")
        # The four gates, in order.
        for gate in ('case check', 'case diffCaps', 'case run', 'case seal'):
            self.assertIn(gate, bridge)
        # Honesty anchors: verdict from decision.md, capability+depth only, seal != approval.
        self.assertIn("never recomputed", bridge.lower(), "the verdict-verbatim rule must be documented")
        self.assertIn("@caps + @max_depth", bridge, "the enforced kernel pair must be named")
        # The seal gate must only pass when a seal was actually parsed.
        self.assertIn("sealPresent", bridge)

    def test_bootstrap_is_generate_only_never_spawns_and_is_allowlisted(self) -> None:
        # M6 (descoped): the macOS bootstrap GENERATES allowlisted bash/zsh scripts
        # for operator-run only. It must never spawn/execute, never use sudo, and
        # never edit a shell profile.
        main = read(MAIN)
        self.assertIn('case bootstrap = "Bootstrap"', main, "bootstrap section must be wired")
        self.assertIn("$0 != .bootstrap", main, "bootstrap must be a power-only section")
        cmd = read(BOOTSTRAP_CMD)
        # The writer must NOT spawn or execute anything.
        for spawn in ("Process(", "StudioProcessRunner", "waitUntilExit", "/bin/sh", "execv"):
            self.assertNotIn(spawn, cmd, f"the bootstrap writer must not {spawn} — generate only")
        bridge = read(BOOTSTRAP_BRIDGE)
        # The allowlist guard must forbid privileged tokens (and not be empty).
        # (That every GENERATED script is actually clean is proved by the Swift
        # unit test testEveryGeneratedScriptIsAllowlistClean.)
        self.assertIn("forbiddenTokens", bridge)
        self.assertIn('"sudo"', bridge, "the allowlist must forbid sudo")
        self.assertIn("| sh", bridge, "the allowlist must forbid remote-pipe execution")
        self.assertIn("rm -rf", bridge, "the allowlist must forbid destructive removals")
        # The honesty copy: generate-only, never runs, no sudo, no profile edit.
        section = read(APP / "Sources" / "GarnetStudio" / "BootstrapSection.swift")
        self.assertIn("never runs them", section)
        self.assertIn("never uses sudo", section)
        self.assertIn("generation only", section)

    def test_distribution_reporter_does_not_overclaim_signing_or_notarization(self) -> None:
        # M7: the macOS .app is unsigned + un-notarized. The reporter must say so;
        # signing and notarization stay deferred and the headline never claims
        # distribution-readiness.
        main = read(MAIN)
        self.assertIn('case distribution = "Distribution"', main, "distribution section must be wired")
        self.assertIn("$0 != .distribution", main, "distribution must be a power-only section")
        bridge = read(DISTRIBUTION_BRIDGE)
        self.assertIn("unsigned and un-notarized", bridge, "the honest posture must be stated")
        self.assertIn("Notarization (Apple notary)", bridge)
        self.assertIn("Code signing (Developer ID)", bridge)
        self.assertIn("Gatekeeper", bridge)
        # Signing/notarization/gatekeeper rows are catalog .deferred (never ready).
        self.assertGreaterEqual(
            bridge.count("catalog: .deferred"), 3,
            "signing + notarization + gatekeeper must be deferred, not ready",
        )

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
