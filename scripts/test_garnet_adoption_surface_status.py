#!/usr/bin/env python3
"""Regression tests for the repo/site adoption surface status reporter."""
from __future__ import annotations

import importlib.util
import sys
import unittest
from dataclasses import asdict
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_adoption_surface_status.py")
SPEC = importlib.util.spec_from_file_location("garnet_adoption_surface_status", SCRIPT)
assert SPEC is not None
status_script = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules["garnet_adoption_surface_status"] = status_script
SPEC.loader.exec_module(status_script)


class GarnetAdoptionSurfaceStatusTests(unittest.TestCase):
    def test_json_separates_active_planned_and_llm_assist_truth(self) -> None:
        data = asdict(status_script.read_surface())

        self.assertEqual(["Rust", "Ruby", "Python", "Go"], data["active_converter_languages"])
        self.assertEqual(
            {"JavaScript", "TypeScript", "Swift", "Java", "C", "C++", "C#", "Perl"},
            set(data["planned_converter_languages"]),
        )
        self.assertEqual("active-partial", data["llm_assist_status"])
        self.assertIn("converter LLM feasibility is advisory-feasible", data["llm_assist_truth"])
        self.assertIn(
            "advisory planning is feasible, autonomous LLM conversion is not feasible yet",
            data["llm_assist_truth"],
        )
        self.assertIn("provider-neutral advisory bundle is active", data["llm_assist_truth"])
        self.assertIn("provider-backed conversion is not active", data["llm_assist_truth"])
        self.assertIn("deterministic converter output remains authoritative", data["llm_assist_truth"])

    def test_json_inventory_keeps_public_hooks_evidence_backed(self) -> None:
        data = asdict(status_script.read_surface())

        use_case_ids = {item["id"] for item in data["verified_use_cases"]}
        self.assertEqual(
            {
                "dual-mode-programming",
                "agent-toolbelt",
                "migration-assistant",
                "agentic-dogfood-matrix",
                "macos-workbench",
            },
            use_case_ids,
        )
        contract = "\n".join(data["repo_site_contract"])
        self.assertIn("Rust, Ruby, Python, and Go only", contract)
        self.assertIn("planned lanes", contract)
        self.assertIn("not active conversion", contract)

    def test_macos_workbench_hook_reflects_current_studio_workflows(self) -> None:
        data = asdict(status_script.read_surface())
        workbench = next(item for item in data["verified_use_cases"] if item["id"] == "macos-workbench")

        self.assertIn("Assist Plan", workbench["hook"])
        self.assertIn("Codex Run", workbench["hook"])
        self.assertIn("dist/Garnet Studio.app", workbench["hook"])
        self.assertIn("script/build_and_run.sh", workbench["evidence"])
        self.assertIn(".codex/environments/environment.toml", workbench["evidence"])
        self.assertIn("scripts/test_garnet_studio_run_button.py", workbench["evidence"])

    def test_json_keeps_productization_gates_open(self) -> None:
        gates = set(asdict(status_script.read_surface())["open_gates"])

        self.assertGreaterEqual(
            gates,
            {
                "Developer ID notarization",
                "mobile distribution",
                "promo video",
                "provider-backed LLM assist",
                "broad deterministic converter frontends",
            },
        )

    def test_markdown_contains_site_safe_summary(self) -> None:
        markdown = status_script.render_markdown(status_script.read_surface())

        self.assertIn("Rust rigor, Ruby velocity, agent-native dogfood evidence.", markdown)
        self.assertIn("Active deterministic lanes: Rust, Ruby, Python, Go.", markdown)
        self.assertIn("Planned lanes only: JavaScript, TypeScript, Swift", markdown)
        self.assertIn("provider-backed conversion is not active", markdown)
        self.assertIn("It is not yet a notarized product", markdown)
        self.assertIn("Dual-mode programming", markdown)
        self.assertIn("Agent toolbelt examples", markdown)
        self.assertIn("Migration assistant", markdown)
        self.assertIn("Repo/Site Contract", markdown)

    def test_repo_and_site_point_to_the_adoption_surface_reporter(self) -> None:
        root = SCRIPT.parents[1]
        readme = (root / "README.md").read_text(encoding="utf-8")
        site = (root / "docs" / "index.html").read_text(encoding="utf-8")

        self.assertIn("scripts/garnet_adoption_surface_status.py", readme)
        self.assertIn("scripts/garnet_adoption_surface_status.py", site)
        self.assertIn("Adoption surface", site)
        self.assertIn("Garnet Studio workbench", site)
        self.assertIn("Codex Run", site)
        self.assertIn("dist/Garnet Studio.app", site)
        self.assertIn("Assist Plan", site)
        self.assertIn("Advisory Bundle", site)
        self.assertIn("provider-neutral prompt pack", site)
        self.assertIn("feasibility + context + prompt + assist plan + advisory bundle active", site)


if __name__ == "__main__":
    unittest.main()
