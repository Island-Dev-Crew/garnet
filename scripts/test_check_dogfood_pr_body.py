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
