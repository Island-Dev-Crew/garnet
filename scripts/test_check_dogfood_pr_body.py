#!/usr/bin/env python3
"""Regression tests for the dogfood readiness PR body checker."""
from __future__ import annotations

import importlib.util
from pathlib import Path
import unittest

SCRIPT = Path(__file__).with_name("check_dogfood_pr_body.py")
SPEC = importlib.util.spec_from_file_location("check_dogfood_pr_body", SCRIPT)
assert SPEC is not None
checker = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(checker)

VALID_BODY = """
## Summary

Promotes one bounded readiness slice.

## Dogfood Readiness

### Current truth

- origin/main has been refreshed before this PR was opened.

### Local verification

- [x] `cargo fmt --all -- --check`
- [x] `cargo test -p garnet-cli --test conformance_phase_gates`

### Remote verification

- [x] Draft PR checks are expected to run before merge.

### Desktop dogfood bundle

- [x] `/Users/idc2.0/Desktop/dogfood/example-readiness-bundle`

### Deferred / out of scope

- Production allocator-integrated ARC remains deferred.
"""

# The `dogfood-readiness` skill template titles the evidence section
# "### Evidence bundle" and adds Merge confidence / Goal progress sections.
# The gate must accept this shape too (S31 reconciliation).
VALID_BODY_NEW_SKILL = """
## Summary

Adopts the dogfood-readiness skill body shape.

## Dogfood Readiness

### Current truth

- [x] Base/head refs and dirty state recorded.

### Local verification

- [x] `python3 -m unittest discover -s tests`

### Remote verification

- [x] CI matrix expected green before merge.

### Merge confidence

- [x] Fused band recorded (internal min external).

### Goal progress

- [x] goal-tracked from .dogfood/goal.json.

### Evidence bundle

- [x] Artifacts copied to a durable project folder.

### Deferred / out of scope

- This PR does not claim production readiness.
"""


class DogfoodPrBodyCheckerTests(unittest.TestCase):
    def test_ignores_non_sensitive_changes_without_body(self) -> None:
        result = checker.validate_body("", ["README.md"])
        self.assertEqual([], result.errors)

    def test_requires_dogfood_section_for_readiness_sensitive_changes(self) -> None:
        result = checker.validate_body("## Summary\n\nTiny update.\n", ["garnet-memory-v0.3/src/cycle.rs"])
        self.assertIn("missing required heading: ## Dogfood Readiness", result.errors)

    def test_accepts_complete_dogfood_evidence_body(self) -> None:
        result = checker.validate_body(VALID_BODY, ["garnet-memory-v0.3/tests/cycle.rs"])
        self.assertEqual([], result.errors)

    def test_accepts_new_skill_evidence_bundle_heading(self) -> None:
        result = checker.validate_body(VALID_BODY_NEW_SKILL, ["F_Project_Management/GARNET_v0_8_SLICE_DOGFOOD.md"])
        self.assertEqual([], result.errors)

    def test_requires_an_evidence_section_heading(self) -> None:
        body = VALID_BODY.replace("### Desktop dogfood bundle", "### Some other heading")
        result = checker.validate_body(body, ["F_Project_Management/GARNET_v0_8_SLICE_DOGFOOD.md"])
        self.assertIn("missing required heading: ### Evidence bundle (or ### Desktop dogfood bundle)", result.errors)

    def test_prose_mention_of_heading_does_not_satisfy_gate(self) -> None:
        # A heading named only inside prose/inline-code (not at line start) must not
        # count as the real section — regression for the substring-find false match.
        body = (
            "## Summary\n\n"
            "- gate now accepts `### Evidence bundle` or `### Desktop dogfood bundle`.\n\n"
            "## Dogfood Readiness\n\n"
            "### Current truth\n- recorded\n\n"
            "### Local verification\n- [x] `cmd`\n\n"
            "### Remote verification\n- [x] checks\n\n"
            "### Deferred / out of scope\n- nothing\n"
        )
        result = checker.validate_body(body, ["F_Project_Management/GARNET_v0_8_SLICE_DOGFOOD.md"])
        self.assertIn("missing required heading: ### Evidence bundle (or ### Desktop dogfood bundle)", result.errors)

    def test_real_heading_wins_over_prose_mention(self) -> None:
        # Prose mention earlier + a real evidence heading later → gate validates the real one.
        body = (
            "## Summary\n\n"
            "- gate now accepts `### Desktop dogfood bundle` (legacy).\n\n"
            "## Dogfood Readiness\n\n"
            "### Current truth\n- recorded\n\n"
            "### Local verification\n- [x] `cmd`\n\n"
            "### Remote verification\n- [x] checks\n\n"
            "### Evidence bundle\n- [x] bundle path\n\n"
            "### Deferred / out of scope\n- nothing\n"
        )
        result = checker.validate_body(body, ["F_Project_Management/GARNET_v0_8_SLICE_DOGFOOD.md"])
        self.assertEqual([], result.errors)

    def test_evidence_section_requires_a_checked_item(self) -> None:
        body = VALID_BODY_NEW_SKILL.replace(
            "- [x] Artifacts copied to a durable project folder.",
            "- [ ] Artifacts pending.",
        )
        result = checker.validate_body(body, ["F_Project_Management/GARNET_v0_8_SLICE_DOGFOOD.md"])
        self.assertIn("evidence bundle section must include at least one checked evidence item", result.errors)

    def test_rejects_unqualified_production_arc_completion_claim(self) -> None:
        body = VALID_BODY + "\nProduction ARC complete.\n"
        result = checker.validate_body(body, ["garnet-memory-v0.3/src/alloc.rs"])
        self.assertIn("unqualified production ARC completion claim", result.errors)

    def test_accepts_explicitly_negated_production_arc_completion_claim(self) -> None:
        body = VALID_BODY + "\nThis slice does not claim production ARC complete.\n"
        result = checker.validate_body(body, ["garnet-memory-v0.3/src/alloc.rs"])
        self.assertEqual([], result.errors)


if __name__ == "__main__":
    unittest.main()
