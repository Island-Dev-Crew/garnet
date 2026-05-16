#!/usr/bin/env python3
"""Regression tests for the agentic dogfood matrix inventory."""
from __future__ import annotations

import importlib.util
import sys
import tempfile
import unittest
from collections import Counter
from pathlib import Path
from textwrap import dedent

SCRIPT = Path(__file__).with_name("run_agentic_dogfood_matrix.py")
SPEC = importlib.util.spec_from_file_location("run_agentic_dogfood_matrix", SCRIPT)
assert SPEC is not None
matrix = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules["run_agentic_dogfood_matrix"] = matrix
SPEC.loader.exec_module(matrix)


class AgenticDogfoodMatrixTests(unittest.TestCase):
    def _fake_result(self, probe: object) -> object:
        return matrix.ProbeResult(
            probe=probe,
            status="passed",
            exit_code=0,
            duration_ms=1,
            stdout_log="/tmp/stdout.log",
            stderr_log="/tmp/stderr.log",
            stdout_excerpt="",
            stderr_excerpt="",
        )

    def _inventory_results(self, probes: list[object]) -> list[object]:
        results = []
        for probe in probes:
            if isinstance(probe, matrix.Probe):
                results.append(self._fake_result(probe))
            else:
                results.append(probe())
        return results

    def test_probe_inventory_includes_agent_recovery_domain(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(Path("/usr/bin/true"), work, fixtures, include_app_workbench=False)
            results = self._inventory_results(probes)

        ids = {result.probe.id for result in results}
        domains = Counter(result.probe.domain for result in results)

        self.assertEqual(domains["agent recovery and diagnostics"], 4)
        self.assertIn("report-converter-adoption-status", ids)
        self.assertIn("check-malformed-agent-source", ids)
        self.assertIn("check-missing-agent-source", ids)
        self.assertIn("eval-unknown-agent-symbol", ids)
        self.assertIn("verify-missing-release-manifest", ids)

    def test_converter_status_probe_guards_intelligent_assist_contract(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(Path("/usr/bin/true"), work, fixtures, include_app_workbench=False)

        probe = next(
            probe
            for probe in probes
            if isinstance(probe, matrix.Probe)
            and probe.id == "report-converter-adoption-status"
        )

        self.assertIn("planned-contract", probe.expected_stdout)
        self.assertIn("CapCaps/capability boundaries", probe.expected_stdout)
        self.assertIn("provider_required", probe.expected_stdout)

    def test_probe_inventory_includes_mit_readiness_accounting(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(Path("/usr/bin/true"), work, fixtures, include_app_workbench=False)
            concrete_probes = [probe for probe in probes if isinstance(probe, matrix.Probe)]

        ids = {probe.id for probe in concrete_probes}
        domains = Counter(probe.domain for probe in concrete_probes)

        self.assertEqual(domains["MIT readiness accounting"], 3)
        self.assertIn("report-mit-readiness-plan-complete", ids)
        self.assertIn("report-mit-readiness-open-productization", ids)
        self.assertIn("report-mit-readiness-assist-and-frontends", ids)

    def test_probe_inventory_includes_mac_side_continuation_accounting(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(Path("/usr/bin/true"), work, fixtures, include_app_workbench=False)
            concrete_probes = [probe for probe in probes if isinstance(probe, matrix.Probe)]

        ids = {probe.id for probe in concrete_probes}
        domains = Counter(probe.domain for probe in concrete_probes)

        self.assertEqual(domains["Mac-side continuation accounting"], 1)
        self.assertIn("report-mac-side-continuation-boundaries", ids)

    def test_probe_inventory_includes_mit_demo_route(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(Path("/usr/bin/true"), work, fixtures, include_app_workbench=False)
            results = self._inventory_results(probes)

        ids = {result.probe.id for result in results}
        domains = Counter(result.probe.domain for result in results)

        self.assertEqual(domains["MIT demo route"], 3)
        self.assertIn("report-mit-demo-route-current-truth", ids)
        self.assertIn("report-mit-demo-route-blocked-gates", ids)
        self.assertIn("report-mit-demo-route-output-manifest", ids)

    def test_probe_inventory_includes_mit_deck_outline(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(Path("/usr/bin/true"), work, fixtures, include_app_workbench=False)
            results = self._inventory_results(probes)

        ids = {result.probe.id for result in results}
        domains = Counter(result.probe.domain for result in results)

        self.assertEqual(domains["MIT deck outline"], 3)
        self.assertIn("report-mit-deck-outline-current-truth", ids)
        self.assertIn("report-mit-deck-outline-blocked-gates", ids)
        self.assertIn("report-mit-deck-outline-output-manifest", ids)

    def test_probe_inventory_includes_promo_video_readiness_contract(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(Path("/usr/bin/true"), work, fixtures, include_app_workbench=False)
            results = self._inventory_results(probes)

        ids = {result.probe.id for result in results}
        domains = Counter(result.probe.domain for result in results)

        self.assertEqual(domains["promo video readiness"], 9)
        self.assertIn("report-promo-video-current-truth", ids)
        self.assertIn("report-promo-video-required-gates", ids)
        self.assertIn("report-promo-video-source-lock", ids)
        self.assertIn("report-promo-video-composition-source", ids)
        self.assertIn("report-promo-video-render-harness-contract", ids)
        self.assertIn("report-promo-video-visual-qa-harness-contract", ids)
        self.assertIn("report-promo-video-website-export-harness-contract", ids)
        self.assertIn("report-promo-video-site-sync-harness-contract", ids)
        self.assertIn("report-promo-video-output-manifest", ids)

    def test_probe_inventory_includes_repo_site_adoption_surface(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(Path("/usr/bin/true"), work, fixtures, include_app_workbench=False)
            concrete_probes = [probe for probe in probes if isinstance(probe, matrix.Probe)]

        ids = {probe.id for probe in concrete_probes}
        domains = Counter(probe.domain for probe in concrete_probes)

        self.assertEqual(domains["repo/site adoption surface"], 3)
        self.assertIn("report-adoption-surface-active-truth", ids)
        self.assertIn("report-adoption-surface-planned-frontends", ids)
        self.assertIn("report-adoption-surface-use-cases", ids)

    def test_probe_inventory_includes_assist_context_pack(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(Path("/usr/bin/true"), work, fixtures, include_app_workbench=False)
            concrete_probes = [probe for probe in probes if isinstance(probe, matrix.Probe)]

        ids = {probe.id for probe in concrete_probes}
        domains = Counter(probe.domain for probe in concrete_probes)

        self.assertEqual(domains["converter intelligent assist"], 4)
        self.assertIn("report-assist-context-current-truth", ids)
        self.assertIn("report-assist-context-required-gates", ids)
        self.assertIn("report-assist-context-documents", ids)
        self.assertIn("report-assist-context-prompt-pack", ids)

    def test_probe_inventory_includes_converter_assist_plan(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(Path("/usr/bin/true"), work, fixtures, include_app_workbench=False)
            results = self._inventory_results(probes)

        ids = {result.probe.id for result in results}
        domains = Counter(result.probe.domain for result in results)

        self.assertEqual(domains["converter assist planning"], 10)
        self.assertIn("report-assist-plan-typescript-current-truth", ids)
        self.assertIn("report-assist-plan-typescript-risks", ids)
        self.assertIn("report-assist-plan-javascript-risks", ids)
        self.assertIn("report-assist-plan-swift-risks", ids)
        self.assertIn("report-assist-plan-java-risks", ids)
        self.assertIn("report-assist-plan-c-risks", ids)
        self.assertIn("report-assist-plan-cpp-risks", ids)
        self.assertIn("report-assist-plan-csharp-risks", ids)
        self.assertIn("report-assist-plan-perl-risks", ids)
        self.assertIn("report-assist-plan-output-manifest", ids)

    def test_probe_inventory_includes_converter_llm_feasibility_gate(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(Path("/usr/bin/true"), work, fixtures, include_app_workbench=False)
            results = self._inventory_results(probes)

        ids = {result.probe.id for result in results}
        domains = Counter(result.probe.domain for result in results)

        self.assertEqual(domains["converter LLM feasibility"], 4)
        self.assertIn("report-converter-llm-feasibility-current-truth", ids)
        self.assertIn("report-converter-llm-feasibility-language-coverage", ids)
        self.assertIn("report-converter-llm-feasibility-blockers", ids)
        self.assertIn("report-converter-llm-feasibility-output-manifest", ids)

    def test_probe_inventory_includes_converter_advisory_bundle(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(Path("/usr/bin/true"), work, fixtures, include_app_workbench=False)
            results = self._inventory_results(probes)

        ids = {result.probe.id for result in results}
        domains = Counter(result.probe.domain for result in results)

        self.assertEqual(domains["converter advisory bundle"], 4)
        self.assertIn("report-converter-advisory-bundle-current-truth", ids)
        self.assertIn("report-converter-advisory-bundle-omits-source", ids)
        self.assertIn("report-converter-advisory-bundle-include-source-gate", ids)
        self.assertIn("report-converter-advisory-bundle-output-manifest", ids)

    def test_probe_inventory_includes_converter_advisory_bundle_studio_ux(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(Path("/usr/bin/true"), work, fixtures, include_app_workbench=False)
            concrete_probes = [probe for probe in probes if isinstance(probe, matrix.Probe)]

        ids = {probe.id for probe in concrete_probes}
        domains = Counter(probe.domain for probe in concrete_probes)

        self.assertEqual(domains["converter advisory bundle UX"], 3)
        self.assertIn("report-studio-advisory-bundle-action", ids)
        self.assertIn("report-studio-advisory-bundle-runner", ids)
        self.assertIn("report-studio-advisory-bundle-default-privacy", ids)

    def test_probe_inventory_includes_converter_advisory_review_studio_ux(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(Path("/usr/bin/true"), work, fixtures, include_app_workbench=False)
            concrete_probes = [probe for probe in probes if isinstance(probe, matrix.Probe)]

        ids = {probe.id for probe in concrete_probes}
        domains = Counter(probe.domain for probe in concrete_probes)

        self.assertEqual(domains["converter advisory review UX"], 3)
        self.assertIn("report-studio-advisory-review-action", ids)
        self.assertIn("report-studio-advisory-review-runner", ids)
        self.assertIn("report-studio-advisory-review-desktop-evidence", ids)

    def test_probe_inventory_includes_converter_advisory_handoff_studio_ux(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(Path("/usr/bin/true"), work, fixtures, include_app_workbench=False)
            concrete_probes = [probe for probe in probes if isinstance(probe, matrix.Probe)]

        ids = {probe.id for probe in concrete_probes}
        domains = Counter(probe.domain for probe in concrete_probes)

        self.assertEqual(domains["converter advisory handoff UX"], 3)
        self.assertIn("report-studio-advisory-handoff-action", ids)
        self.assertIn("report-studio-advisory-handoff-runner", ids)
        self.assertIn("report-studio-advisory-handoff-desktop-evidence", ids)

    def test_probe_inventory_includes_studio_objective_pulse_gate(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(Path("/usr/bin/true"), work, fixtures, include_app_workbench=False)
            concrete_probes = [probe for probe in probes if isinstance(probe, matrix.Probe)]

        ids = {probe.id for probe in concrete_probes}
        domains = Counter(probe.domain for probe in concrete_probes)

        self.assertEqual(domains["MIT objective pulse UX"], 3)
        self.assertIn("report-studio-objective-pulse-action", ids)
        self.assertIn("report-studio-objective-pulse-runner", ids)
        self.assertIn("report-studio-objective-pulse-truth-copy", ids)

    def test_probe_inventory_includes_studio_mac_continuation_pulse_gate(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(Path("/usr/bin/true"), work, fixtures, include_app_workbench=False)
            concrete_probes = [probe for probe in probes if isinstance(probe, matrix.Probe)]

        ids = {probe.id for probe in concrete_probes}
        domains = Counter(probe.domain for probe in concrete_probes)

        self.assertEqual(domains["Mac continuation pulse UX"], 3)
        self.assertIn("report-studio-mac-continuation-pulse-action", ids)
        self.assertIn("report-studio-mac-continuation-pulse-runner", ids)
        self.assertIn("report-studio-mac-continuation-pulse-truth-copy", ids)

    def test_probe_inventory_includes_studio_mit_demo_route_gate(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(Path("/usr/bin/true"), work, fixtures, include_app_workbench=False)
            concrete_probes = [probe for probe in probes if isinstance(probe, matrix.Probe)]

        ids = {probe.id for probe in concrete_probes}
        domains = Counter(probe.domain for probe in concrete_probes)

        self.assertEqual(domains["MIT demo route UX"], 3)
        self.assertIn("report-studio-mit-demo-route-action", ids)
        self.assertIn("report-studio-mit-demo-route-runner", ids)
        self.assertIn("report-studio-mit-demo-route-desktop-evidence", ids)

    def test_probe_inventory_includes_converter_advisory_review_gate(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(Path("/usr/bin/true"), work, fixtures, include_app_workbench=False)
            results = self._inventory_results(probes)

        ids = {result.probe.id for result in results}
        domains = Counter(result.probe.domain for result in results)

        self.assertEqual(domains["converter advisory review"], 3)
        self.assertIn("report-converter-advisory-review-current-truth", ids)
        self.assertIn("report-converter-advisory-review-source-included-block", ids)
        self.assertIn("report-converter-advisory-review-output-manifest", ids)

    def test_probe_inventory_includes_converter_advisory_handoff_gate(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(Path("/usr/bin/true"), work, fixtures, include_app_workbench=False)
            results = self._inventory_results(probes)

        ids = {result.probe.id for result in results}
        domains = Counter(result.probe.domain for result in results)

        self.assertEqual(domains["converter advisory handoff"], 3)
        self.assertIn("report-converter-advisory-handoff-current-truth", ids)
        self.assertIn("report-converter-advisory-handoff-source-included-block", ids)
        self.assertIn("report-converter-advisory-handoff-output-manifest", ids)

    def test_probe_inventory_includes_web_pwa_offline_gate(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(Path("/usr/bin/true"), work, fixtures, include_app_workbench=False)
            concrete_probes = [probe for probe in probes if isinstance(probe, matrix.Probe)]

        ids = {probe.id for probe in concrete_probes}
        domains = Counter(probe.domain for probe in concrete_probes)

        self.assertEqual(domains["web/PWA productization"], 3)
        self.assertIn("smoke-web-pwa-offline-handler", ids)
        self.assertIn("smoke-web-pwa-local-readiness", ids)
        self.assertIn("smoke-web-pwa-browser-offline", ids)

    def test_probe_inventory_includes_signed_release_provenance(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(Path("/usr/bin/true"), work, fixtures, include_app_workbench=False)
            results = self._inventory_results(probes)

        ids = {result.probe.id for result in results}
        domains = Counter(result.probe.domain for result in results)

        self.assertEqual(domains["signed release provenance"], 3)
        self.assertIn("release-keygen-build-verify-signature", ids)
        self.assertIn("release-unsigned-manifest-requires-signature", ids)
        self.assertIn("release-signed-manifest-tamper-detection", ids)

    def test_probe_inventory_includes_macos_notarization_readiness(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(Path("/usr/bin/true"), work, fixtures, include_app_workbench=False)
            concrete_probes = [probe for probe in probes if isinstance(probe, matrix.Probe)]

        ids = {probe.id for probe in concrete_probes}
        domains = Counter(probe.domain for probe in concrete_probes)

        self.assertEqual(domains["macOS notarization readiness"], 3)
        self.assertIn("report-notarization-status-blockers", ids)
        self.assertIn("report-notarization-status-redaction", ids)
        self.assertIn("report-notarization-status-missing-bundle", ids)

    def test_signed_release_probe_does_not_persist_generated_private_keys(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fake_garnet = work / "fake-garnet"
            fake_garnet.write_text(
                dedent(
                    """\
                    #!/usr/bin/env sh
                    case "$1" in
                      keygen)
                        printf 'private test key\\n' > "$2"
                        echo 'generated Ed25519 signing keypair'
                        ;;
                      build)
                        source="${5:-$3}"
                        manifest="${source}.manifest.json"
                        printf '{"signature":"test","signer_pubkey":"test"}\\n' > "$manifest"
                        echo 'signed_by test'
                        ;;
                      verify)
                        echo 'signature valid'
                        ;;
                    esac
                    """
                ),
                encoding="utf-8",
            )
            fake_garnet.chmod(0o755)
            fixtures = matrix.prepare_fixtures(work)

            result = matrix.build_signed_release_probe(fake_garnet, work, fixtures["build_source"])

            self.assertTrue(result.passed)
            self.assertEqual([], list((work / "signed-release").glob("*.key")))

    def test_probe_inventory_covers_developer_experience_repair(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(Path("/usr/bin/true"), work, fixtures, include_app_workbench=False)
            concrete_probes = [probe for probe in probes if isinstance(probe, matrix.Probe)]

        ids = {probe.id for probe in concrete_probes}
        domains = Counter(probe.domain for probe in concrete_probes)

        self.assertEqual(domains["developer experience"], 3)
        self.assertIn("fmt-repair-dirty-agent", ids)

    def test_probe_inventory_covers_memory_declaration_analysis(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(Path("/usr/bin/true"), work, fixtures, include_app_workbench=False)
            concrete_probes = [probe for probe in probes if isinstance(probe, matrix.Probe)]

        ids = {probe.id for probe in concrete_probes}
        domains = Counter(probe.domain for probe in concrete_probes)

        self.assertEqual(domains["agent memory and analysis"], 3)
        self.assertIn("parse-advertised-log-analyzer-memory", ids)

    def test_probe_inventory_covers_signed_memory_persistence_integrity(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(Path("/usr/bin/true"), work, fixtures, include_app_workbench=False)
            concrete_probes = [probe for probe in probes if isinstance(probe, matrix.Probe)]

        ids = {probe.id for probe in concrete_probes}
        domains = Counter(probe.domain for probe in concrete_probes)

        self.assertEqual(domains["memory persistence integrity"], 3)
        self.assertIn("memory-signed-cache-roundtrip", ids)
        self.assertIn("memory-signed-cache-tamper-rejection", ids)
        self.assertIn("memory-signed-cache-foreign-key-rejection", ids)

    def test_packaged_matrix_skips_source_workspace_memory_integrity_probes(self) -> None:
        original_root = matrix.ROOT
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp) / "dogfood"
            packaged_resources = Path(temp) / "Garnet Studio.app" / "Contents" / "Resources"
            packaged_resources.mkdir(parents=True)
            matrix.ROOT = packaged_resources
            try:
                probes = matrix.memory_persistence_integrity_probes(work)
                results = [probe() for probe in probes]
            finally:
                matrix.ROOT = original_root

        self.assertEqual([result.status for result in results], ["skipped", "skipped", "skipped"])
        self.assertTrue(all(result.passed for result in results))
        self.assertEqual(matrix.score(results)["skipped"], 3)
        self.assertIn("Cargo.toml", results[0].stdout_excerpt)
        self.assertIn("source workspace", results[0].probe.notes)

    def test_probe_inventory_covers_agent_toolbelt_examples(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(Path("/usr/bin/true"), work, fixtures, include_app_workbench=False)
            concrete_probes = [probe for probe in probes if isinstance(probe, matrix.Probe)]

        ids = {probe.id for probe in concrete_probes}
        domains = Counter(probe.domain for probe in concrete_probes)

        self.assertEqual(domains["agent toolbelt examples"], 5)
        self.assertIn("run-agent-toolbelt-triage-router", ids)
        self.assertIn("run-agent-toolbelt-capability-budget", ids)
        self.assertIn("run-agent-toolbelt-memory-recall", ids)
        self.assertIn("run-agent-toolbelt-release-gate", ids)
        self.assertIn("run-agent-toolbelt-repair-planner", ids)

    def test_probe_inventory_covers_agent_adversarial_boundaries(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(Path("/usr/bin/true"), work, fixtures, include_app_workbench=False)
            concrete_probes = [probe for probe in probes if isinstance(probe, matrix.Probe)]

        ids = {probe.id for probe in concrete_probes}
        domains = Counter(probe.domain for probe in concrete_probes)

        self.assertEqual(domains["agent adversarial boundaries"], 3)
        self.assertIn("reject-agent-depth-budget-bomb", ids)
        self.assertIn("reject-agent-main-without-caps", ids)
        self.assertIn("reject-agent-safe-var-mutation", ids)

    def test_probe_inventory_covers_source_app_workbench_smoke(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            garnet = Path("/tmp/garnet-target/debug/garnet")
            probes = matrix.probe_set(garnet, work, fixtures, include_app_workbench=True)
            concrete_probes = [probe for probe in probes if isinstance(probe, matrix.Probe)]

        ids = {probe.id for probe in concrete_probes}
        domains = Counter(probe.domain for probe in concrete_probes)

        self.assertEqual(domains["macOS app workbench"], 3)
        self.assertIn("app-self-test", ids)
        self.assertIn("app-xctest", ids)
        self.assertIn("app-smoke-test", ids)

    def test_probe_inventory_covers_canonical_project_templates(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(Path("/usr/bin/true"), work, fixtures, include_app_workbench=False)
            results = self._inventory_results(probes)

        ids = {result.probe.id for result in results}
        domains = Counter(result.probe.domain for result in results)

        self.assertEqual(domains["project scaffolding"], 3)
        self.assertIn("template-cli-run-and-test", ids)
        self.assertIn("template-web-api-run-and-test", ids)
        self.assertIn("template-agent-orchestrator-run-and-test", ids)

    def test_domain_coverage_marks_undercovered_agentic_surfaces(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(Path("/usr/bin/true"), work, fixtures, include_app_workbench=False)
            results = self._inventory_results(probes)

        coverage = {item["domain"]: item for item in matrix.domain_coverage(results)}

        self.assertEqual(coverage["web/PWA productization"]["probe_count"], 3)
        self.assertEqual(coverage["web/PWA productization"]["target_probe_count"], 3)
        self.assertEqual(coverage["web/PWA productization"]["status"], "adequate")
        self.assertEqual(coverage["project scaffolding"]["status"], "adequate")
        self.assertEqual(coverage["developer experience"]["status"], "adequate")
        self.assertEqual(coverage["agent toolbelt examples"]["status"], "adequate")
        self.assertEqual(coverage["agent memory and analysis"]["status"], "adequate")
        self.assertEqual(coverage["memory persistence integrity"]["status"], "adequate")
        self.assertEqual(coverage["agent recovery and diagnostics"]["status"], "adequate")
        self.assertEqual(coverage["agent adversarial boundaries"]["status"], "adequate")
        self.assertEqual(coverage["MIT readiness accounting"]["status"], "adequate")
        self.assertEqual(coverage["converter intelligent assist"]["status"], "adequate")
        self.assertEqual(coverage["converter assist planning"]["status"], "adequate")
        self.assertEqual(coverage["converter LLM feasibility"]["status"], "adequate")
        self.assertEqual(coverage["converter advisory bundle"]["status"], "adequate")
        self.assertEqual(coverage["converter advisory review"]["status"], "adequate")
        self.assertEqual(coverage["converter advisory handoff"]["status"], "adequate")
        self.assertEqual(coverage["converter advisory bundle UX"]["status"], "adequate")
        self.assertEqual(coverage["converter advisory review UX"]["status"], "adequate")
        self.assertEqual(coverage["converter advisory handoff UX"]["status"], "adequate")
        self.assertEqual(coverage["MIT objective pulse UX"]["status"], "adequate")
        self.assertEqual(coverage["MIT demo route UX"]["status"], "adequate")
        self.assertEqual(coverage["MIT deck outline"]["status"], "adequate")
        self.assertEqual(coverage["signed release provenance"]["status"], "adequate")
        self.assertEqual(coverage["macOS notarization readiness"]["status"], "adequate")

    def test_write_outputs_persists_domain_coverage(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            probe = matrix.Probe(
                "one-web-probe",
                "web/PWA productization",
                "one probe should still expose coverage debt",
                ["/bin/true"],
                True,
            )
            result = self._fake_result(probe)
            matrix.write_outputs(
                work,
                [result],
                {
                    "repo": "/tmp/repo",
                    "head": "abc123",
                    "branch": "test",
                    "garnet": "/bin/garnet",
                    "app_workbench": "skipped",
                    "artifact_dir": str(work),
                },
            )

            data = (work / "dogfood-readiness-data.json").read_text(encoding="utf-8")
            report = (work / "dogfood-readiness-report.md").read_text(encoding="utf-8")

        self.assertIn('"domain_coverage"', data)
        self.assertIn('"needs-expansion"', data)
        self.assertIn("## Domain Coverage Adequacy", report)
        self.assertIn("web/PWA productization", report)


if __name__ == "__main__":
    unittest.main()
