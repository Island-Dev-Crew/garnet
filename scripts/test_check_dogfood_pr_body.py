#!/usr/bin/env python3
"""Regression tests for the dogfood readiness PR body checker."""
from __future__ import annotations

import contextlib
import importlib.util
import io
from pathlib import Path
import subprocess
import unittest
from unittest import mock

SCRIPT = Path(__file__).with_name("check_dogfood_pr_body.py")
SPEC = importlib.util.spec_from_file_location("check_dogfood_pr_body", SCRIPT)
assert SPEC is not None
checker = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(checker)

FIXTURES = Path(__file__).with_name("fixtures") / "dogfood_pr_bodies"
SENSITIVE = ["F_Project_Management/GARNET_v0_8_SLICE_DOGFOOD.md"]


def read_fixture(name: str) -> str:
    return (FIXTURES / name).read_text(encoding="utf-8")


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

VALID_BODY_EVIDENCE_THEN_DESKTOP = VALID_BODY_NEW_SKILL.replace(
    "### Deferred / out of scope",
    """### Desktop dogfood bundle

- [ ] Legacy bundle intentionally left unchecked.

### Deferred / out of scope""",
)

VALID_BODY_DESKTOP_THEN_EVIDENCE = VALID_BODY.replace(
    "### Deferred / out of scope",
    """### Evidence bundle

- [ ] Alternate bundle intentionally left unchecked.

### Deferred / out of scope""",
)


class SensitivePathClassificationTests(unittest.TestCase):
    """Path-normalization regressions: `lstrip("./")` treats '.' and '/' as a
    character SET, so ".github/workflows/ci.yml" normalized to
    "github/workflows/ci.yml" and workflow-only PRs silently skipped the
    dogfood evidence requirement (found 2026-07-13)."""

    def test_github_workflow_path_is_sensitive(self) -> None:
        self.assertTrue(checker.is_sensitive_path(".github/workflows/ci.yml"))

    def test_pull_request_template_is_sensitive(self) -> None:
        # Same bug hid the SENSITIVE_FILES entry that starts with a dot.
        self.assertTrue(checker.is_sensitive_path(".github/PULL_REQUEST_TEMPLATE.md"))

    def test_leading_dot_slash_prefix_is_stripped(self) -> None:
        self.assertTrue(checker.is_sensitive_path("./.github/workflows/ci.yml"))
        self.assertTrue(checker.is_sensitive_path("./F_Project_Management/LAUNCH/LAUNCH_READINESS.md"))

    def test_windows_separators_normalize(self) -> None:
        self.assertTrue(checker.is_sensitive_path(".github\\workflows\\ci.yml"))

    def test_lookalike_paths_stay_non_sensitive(self) -> None:
        self.assertFalse(checker.is_sensitive_path("github/workflows/ci.yml"))
        self.assertFalse(checker.is_sensitive_path("scripts/render_garnet_promo_video.mjs"))

    def test_the_body_checker_guards_itself(self) -> None:
        # H3-02: this script produces the required "PR dogfood evidence" context, so a
        # change to it must carry that evidence too — this asserted assertFalse before.
        self.assertTrue(checker.is_sensitive_path("scripts/check_dogfood_pr_body.py"))
        self.assertTrue(checker.is_sensitive_path("scripts/test_check_dogfood_pr_body.py"))

    def test_workflow_only_change_requires_dogfood_body(self) -> None:
        result = checker.validate_body("## Summary\n\nCI tweak.\n", [".github/workflows/ci.yml"])
        self.assertTrue(result.sensitive)
        self.assertIn("missing required heading: ## Dogfood Readiness", result.errors)


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

    def test_validates_first_evidence_heading_when_evidence_precedes_desktop(self) -> None:
        self.assertLess(
            checker.heading_line_pos(VALID_BODY_EVIDENCE_THEN_DESKTOP, "### Evidence bundle"),
            checker.heading_line_pos(VALID_BODY_EVIDENCE_THEN_DESKTOP, "### Desktop dogfood bundle"),
        )
        result = checker.validate_body(
            VALID_BODY_EVIDENCE_THEN_DESKTOP,
            ["F_Project_Management/GARNET_v0_8_SLICE_DOGFOOD.md"],
        )
        self.assertEqual([], result.errors)

    def test_validates_first_evidence_heading_when_desktop_precedes_evidence(self) -> None:
        self.assertLess(
            checker.heading_line_pos(VALID_BODY_DESKTOP_THEN_EVIDENCE, "### Desktop dogfood bundle"),
            checker.heading_line_pos(VALID_BODY_DESKTOP_THEN_EVIDENCE, "### Evidence bundle"),
        )
        result = checker.validate_body(
            VALID_BODY_DESKTOP_THEN_EVIDENCE,
            ["F_Project_Management/GARNET_v0_8_SLICE_DOGFOOD.md"],
        )
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
            "### Local verification\n- [x] `cargo fmt --all -- --check`\n\n"
            "### Remote verification\n- [x] checks\n\n"
            "### Deferred / out of scope\n- nothing\n"
        )
        result = checker.validate_body(body, ["F_Project_Management/GARNET_v0_8_SLICE_DOGFOOD.md"])
        self.assertIn("missing required heading: ### Evidence bundle (or ### Desktop dogfood bundle)", result.errors)

    def test_real_heading_wins_over_prose_mention(self) -> None:
        # Prose mention earlier + a real evidence heading later → gate validates the real one.
        # The evidence item is a real command so this test stays about headings, not
        # about the evidence-token rules exercised in EvidenceTokenTests. It read
        # `cmd` until the review v2 cure, which rejects a structureless span.
        body = (
            "## Summary\n\n"
            "- gate now accepts `### Desktop dogfood bundle` (legacy).\n\n"
            "## Dogfood Readiness\n\n"
            "### Current truth\n- recorded\n\n"
            "### Local verification\n- [x] `cargo fmt --all -- --check`\n\n"
            "### Remote verification\n- [x] Draft PR checks are expected to run before merge.\n\n"
            "### Evidence bundle\n- [x] `/Users/idc2.0/Desktop/dogfood/example-readiness-bundle`\n\n"
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


class SectionBoundaryTests(unittest.TestCase):
    """Crown D-1: a section ends at the next heading of the same or higher level.
    A `## ` heading closes an open `### ` section in Markdown; the checker used
    to scan only for the next literal `### `, so a checked item under a later,
    unrelated `## ` section satisfied the evidence contract."""

    OUTSIDE_ITEM_BODY = (
        "## Dogfood Readiness\n\n"
        "### Current truth\n\n- recorded\n\n"
        "### Deferred / out of scope\n\n- nothing deferred\n\n"
        "### Local verification\n\n- [x] `cargo test -p garnet-cli`\n\n"
        "### Remote verification\n\n- [x] Draft PR checks are expected to run before merge.\n\n"
        "### Evidence bundle\n\n- bundle not yet copied\n\n"
        "## Rigor checklist\n\n"
        "- [x] `garnet-stdlib/src/registry.rs` carries the new primitive entry.\n"
    )

    def test_checked_item_under_later_h2_does_not_satisfy_evidence_section(self) -> None:
        result = checker.validate_body(self.OUTSIDE_ITEM_BODY, SENSITIVE)
        self.assertIn("evidence bundle section must include at least one checked evidence item", result.errors)

    def test_section_text_stops_at_same_or_higher_level_heading(self) -> None:
        section = checker.section_text(self.OUTSIDE_ITEM_BODY, "### Evidence bundle")
        self.assertIsNotNone(section)
        self.assertIn("bundle not yet copied", section)
        self.assertNotIn("Rigor checklist", section)
        self.assertNotIn("registry.rs", section)

    def test_deeper_heading_stays_inside_section(self) -> None:
        body = VALID_BODY.replace(
            "- [x] `/Users/idc2.0/Desktop/dogfood/example-readiness-bundle`",
            "#### Bundle\n\n- [x] `/Users/idc2.0/Desktop/dogfood/example-readiness-bundle`",
        )
        result = checker.validate_body(body, SENSITIVE)
        self.assertEqual([], result.errors)


class ExactHeadingTests(unittest.TestCase):
    """Hardening H3-01 (headings): a required heading matches only when the line's
    heading text equals the contract heading exactly after trailing-whitespace
    normalization. `### Current truth — none stated` is a different heading."""

    def test_suffixed_required_heading_does_not_match(self) -> None:
        body = VALID_BODY.replace("### Current truth", "### Current truth — none stated")
        result = checker.validate_body(body, SENSITIVE)
        self.assertIn("missing required heading: ### Current truth", result.errors)

    def test_suffixed_evidence_heading_does_not_match(self) -> None:
        body = VALID_BODY_NEW_SKILL.replace("### Evidence bundle", "### Evidence bundle — no artifact")
        result = checker.validate_body(body, SENSITIVE)
        self.assertIn("missing required heading: ### Evidence bundle (or ### Desktop dogfood bundle)", result.errors)

    def test_trailing_whitespace_is_normalized(self) -> None:
        body = VALID_BODY.replace("### Local verification\n", "### Local verification   \n")
        result = checker.validate_body(body, SENSITIVE)
        self.assertEqual([], result.errors)

    def test_h3_vacuous_body_is_rejected(self) -> None:
        # Verbatim reproduction body from the hardening scope (H3-vacuous-body.md):
        # suffix prose on every heading and only `- [x] x` beneath Local, Remote,
        # and Evidence passed the gate at beeb5e7b.
        body = (
            "## Dogfood Readiness — no facts\n"
            "### Current truth — none stated\n"
            "### Local verification — no command or result\n- [x] x\n"
            "### Remote verification — no run or URL\n- [x] x\n"
            "### Evidence bundle — no artifact\n- [x] x\n"
            "### Deferred / out of scope — none stated\n"
        )
        result = checker.validate_body(body, [".github/workflows/ci.yml"])
        for heading in checker.REQUIRED_HEADINGS:
            self.assertIn(f"missing required heading: {heading}", result.errors)
        self.assertIn("missing required heading: ### Evidence bundle (or ### Desktop dogfood bundle)", result.errors)


class EvidenceTokenTests(unittest.TestCase):
    """Hardening H3-01 (evidence): a checked item counts only when it carries an
    evidence token. A bare `- [x] x` or `- [x] done` is a placeholder, not evidence,
    and the problem names the section."""

    def test_bare_x_in_local_verification_is_rejected(self) -> None:
        body = VALID_BODY.replace(
            "- [x] `cargo fmt --all -- --check`\n- [x] `cargo test -p garnet-cli --test conformance_phase_gates`",
            "- [x] x",
        )
        result = checker.validate_body(body, SENSITIVE)
        self.assertTrue(any(e.startswith("local verification section") for e in result.errors), result.errors)

    def test_bare_done_in_remote_verification_is_rejected(self) -> None:
        body = VALID_BODY.replace("- [x] Draft PR checks are expected to run before merge.", "- [x] done")
        result = checker.validate_body(body, SENSITIVE)
        self.assertTrue(any(e.startswith("remote verification section") for e in result.errors), result.errors)

    def test_bare_done_in_evidence_bundle_is_rejected(self) -> None:
        body = VALID_BODY_NEW_SKILL.replace("- [x] Artifacts copied to a durable project folder.", "- [x] done")
        result = checker.validate_body(body, SENSITIVE)
        self.assertTrue(any(e.startswith("evidence bundle section") for e in result.errors), result.errors)

    def test_one_evidentiary_item_satisfies_the_section(self) -> None:
        # The rule is at-least-one, not every: a placeholder next to a real item is fine.
        body = VALID_BODY.replace("- [x] `cargo fmt --all -- --check`", "- [x] x\n- [x] `cargo fmt --all -- --check`")
        result = checker.validate_body(body, SENSITIVE)
        self.assertEqual([], result.errors)

    def test_hard_evidence_tokens(self) -> None:
        accepted = (
            "`cargo test --workspace` green",
            "the diff touches scripts/check_dogfood_pr_body.py only",
            "AGENTS.md/CHANGELOG.md re-read at docs/why.html",
            "bound at 68317ae",
            "bound at beeb5e7b23e892521da439da67d44a37f23b5584",
            "run https://github.com/Island-Dev-Crew/garnet/actions/runs/1",
            "review record merged with #542",
            "conformance 6/6",
            "Ran 8 tests",
            "exit 0",
            "ok: true",
            "3 tests pass",
            "both targets answer 200",
            "census 67 → 73",
        )
        rejected = ("x", "done", "verified", "all good", "checks", "bundle", "defaced")
        for text in accepted:
            self.assertTrue(checker.item_carries_evidence(text, "local"), text)
        for text in rejected:
            self.assertFalse(checker.item_carries_evidence(text, "local"), text)

    def test_remote_section_accepts_a_named_check_with_its_status(self) -> None:
        ritual = "Fresh PR checks are required to settle before handoff; no CI conclusion is claimed in advance."
        self.assertTrue(checker.item_carries_evidence(ritual, "remote"))
        self.assertTrue(checker.item_carries_evidence("CI matrix expected green before merge.", "remote"))
        self.assertFalse(checker.item_carries_evidence("checks", "remote"))
        self.assertFalse(checker.item_carries_evidence(ritual, "local"))

    def test_evidence_section_accepts_a_named_artifact_with_its_location(self) -> None:
        self.assertTrue(checker.item_carries_evidence("The one-line diff and the cross-family record named above.", "evidence"))
        self.assertTrue(checker.item_carries_evidence("Artifacts copied to a durable project folder.", "evidence"))
        self.assertFalse(checker.item_carries_evidence("bundle", "evidence"))
        self.assertFalse(checker.item_carries_evidence("The one-line diff and the cross-family record named above.", "local"))


class WidenedAlternativeBypassTests(unittest.TestCase):
    """Review v1 (Codex, cross-family, bound to 250c9748): H3-01 evidence
    hardening remained bypassable. `widened-vacuous-pass.md` passed the gate at
    the candidate head — `dogfood-pr-body: ok (1 changed files checked)` — with a
    whitespace-only code span under Local, a generic negated `No CI run.` under
    Remote, and a generic negated `No report was recorded.` under Evidence. None
    of those is a recomputable token or the named check/artifact fact AGENTS.md
    requires. The cure: a code span must carry content; the two widened
    alternatives must be positive, unnegated, and substantive; and a negated
    clause satisfies no alternative at all."""

    SECTION_MESSAGE = "{label} section has 1 checked item(s) but none carries evidence"

    def assert_only_problem(self, body: str, label: str) -> None:
        result = checker.validate_body(body, SENSITIVE)
        self.assertEqual(1, len(result.errors), result.errors)
        self.assertTrue(
            result.errors[0].startswith(self.SECTION_MESSAGE.format(label=label)),
            result.errors[0],
        )

    def test_reviewer_bypass_body_is_now_rejected_in_all_three_sections(self) -> None:
        result = checker.validate_body(read_fixture("widened-vacuous-pass.md"), [".github/workflows/ci.yml"])
        self.assertTrue(result.sensitive)
        for label in ("local verification", "remote verification", "evidence bundle"):
            self.assertTrue(
                any(e.startswith(self.SECTION_MESSAGE.format(label=label)) for e in result.errors),
                (label, result.errors),
            )

    def test_whitespace_only_code_span_is_not_evidence(self) -> None:
        body = VALID_BODY.replace(
            "- [x] `cargo fmt --all -- --check`\n- [x] `cargo test -p garnet-cli --test conformance_phase_gates`",
            "- [x] ` `",
        )
        self.assert_only_problem(body, "local verification")
        self.assertFalse(checker.code_span_carries_content(" "))
        self.assertFalse(checker.code_span_carries_content("-"))  # one punctuation char
        self.assertFalse(checker.code_span_carries_content("--"))  # pure punctuation
        self.assertTrue(checker.code_span_carries_content("ok"))
        self.assertTrue(checker.code_span_carries_content("cargo fmt --all -- --check"))

    def test_negated_remote_claim_is_not_evidence(self) -> None:
        body = VALID_BODY.replace("- [x] Draft PR checks are expected to run before merge.", "- [x] No CI run.")
        self.assert_only_problem(body, "remote verification")
        self.assertFalse(checker.item_carries_evidence("No CI run.", "remote"))
        self.assertFalse(checker.item_carries_evidence("No CI run was required.", "remote"))
        self.assertFalse(checker.item_carries_evidence("The CI run was not required.", "remote"))
        # A negated clause cannot be laundered by a hard token either.
        self.assertFalse(checker.item_carries_evidence("No `cargo test` was run.", "remote"))
        # The ritual every merged body uses is a positive claim plus a disclaimer
        # and must keep counting — the negation is judged per clause, not per item.
        self.assertTrue(
            checker.item_carries_evidence(
                "Fresh PR checks are required to settle before handoff; "
                "no CI conclusion is claimed in advance.",
                "remote",
            )
        )

    def test_negated_evidence_claim_is_not_evidence(self) -> None:
        body = VALID_BODY_NEW_SKILL.replace(
            "- [x] Artifacts copied to a durable project folder.",
            "- [x] No report was recorded.",
        )
        self.assert_only_problem(body, "evidence bundle")
        self.assertFalse(checker.item_carries_evidence("No report was recorded.", "evidence"))
        self.assertFalse(checker.item_carries_evidence("The bundle is above.", "evidence"))
        self.assertFalse(checker.item_carries_evidence("Bundle recorded.", "evidence"))
        self.assertTrue(
            checker.item_carries_evidence("The one-line diff and the cross-family record named above.", "evidence")
        )


class RealPrBodyFixtureTests(unittest.TestCase):
    """Positive calibration: the merged bodies of #545 and #546 (fetched 2026-09-02
    with `gh pr view N --json body`) must keep passing the tightened contract."""

    def test_pr_545_front_door_body_passes(self) -> None:
        result = checker.validate_body(read_fixture("pr-545.md"), ["docs/index.html", *SENSITIVE])
        self.assertEqual([], result.errors)

    def test_pr_546_register_sweep_body_passes(self) -> None:
        result = checker.validate_body(read_fixture("pr-546.md"), SENSITIVE)
        self.assertEqual([], result.errors)


class GitSubprocessBoundTests(unittest.TestCase):
    """Crown D-N4: the one subprocess the gate runs is bounded and fails closed."""

    def test_git_diff_is_bounded_by_a_30s_timeout(self) -> None:
        seen: dict[str, object] = {}

        def fake_check_output(cmd, **kwargs):
            seen.update(kwargs)
            return "deny.toml\n"

        with mock.patch.object(checker.subprocess, "check_output", fake_check_output):
            paths = checker.read_changed_paths("base", "head")
        self.assertEqual(["deny.toml"], paths)
        self.assertEqual(30, checker.GIT_TIMEOUT_SECONDS)
        self.assertEqual(checker.GIT_TIMEOUT_SECONDS, seen.get("timeout"))

    def test_git_diff_timeout_fails_closed_with_an_explicit_problem(self) -> None:
        def fake_check_output(cmd, **kwargs):
            raise subprocess.TimeoutExpired(cmd, kwargs.get("timeout", 0))

        out = io.StringIO()
        with mock.patch.object(checker.subprocess, "check_output", fake_check_output):
            with contextlib.redirect_stdout(out):
                code = checker.main(["--base", "base", "--head", "head"])
        self.assertEqual(1, code)
        self.assertIn("::error::", out.getvalue())
        self.assertIn("timed out", out.getvalue())


class BareStatusSpanTests(unittest.TestCase):
    """A code span that is only a verdict word is a claim, not evidence."""

    @staticmethod
    def _body(item: str) -> str:
        sections = [f"{h}\n- [x] {item}\n" for h in checker.REQUIRED_HEADINGS]
        sections.append(f"{checker.EVIDENCE_HEADINGS[0]}\n- [x] {item}\n")
        return "\n".join(sections)

    def _errors(self, item: str) -> list[str]:
        return checker.validate_body(self._body(item), [".github/workflows/ci.yml"]).errors

    def test_bare_status_words_are_rejected(self):
        for word in ("ok", "done", "PASSED", "green", "clean", "n/a", "verified"):
            with self.subTest(word=word):
                self.assertTrue(
                    self._errors(f"`{word}`"),
                    f"a bare `{word}` span must not satisfy an evidence section",
                )

    def test_real_evidence_still_passes(self):
        for item in (
            "`cargo test -p garnet-cli` \u2014 `16 passed; 0 failed`",
            "`16 passed`",
            "`scripts/check_dogfood_pr_body.py`",
            "`--gate`",
            "`a81672a5`",
        ):
            with self.subTest(item=item):
                self.assertEqual(
                    self._errors(item), [], f"{item} is recomputable evidence and must pass"
                )

    def test_status_word_inside_a_real_command_still_counts(self):
        self.assertEqual(self._errors("`python3 -I scripts/x.py` printed `ok` for 3 files"), [])


if __name__ == "__main__":
    unittest.main()