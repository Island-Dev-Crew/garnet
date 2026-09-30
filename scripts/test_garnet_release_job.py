#!/usr/bin/env python3
"""Static contract for the tagged-release job in linux-packages.yml (T5b).

The job signs with a key only the `release` environment holds (C4-02),
publishes one curated body (C6-13), and publishes a release for the tag that
started it without creating or moving any tag (the v* tag ruleset).
"""
from __future__ import annotations

import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
WORKFLOW = ROOT / ".github" / "workflows" / "linux-packages.yml"
NOTES = "${{ github.ref_name }}.md"


def release_job() -> str:
    text = WORKFLOW.read_text(encoding="utf-8")
    start = text.index("\n  release:\n")
    following = re.search(r"\n  [a-z0-9-]+:\n", text[start + 1:])
    end = start + 1 + following.start() if following else len(text)
    return text[start:end]


def step(job: str, name: str) -> str:
    start = job.index(f"- name: {name}")
    following = job.find("\n      - ", start + 1)
    return job[start:following if following != -1 else len(job)]


class ReleaseJobTests(unittest.TestCase):
    def test_the_job_runs_in_the_release_environment(self) -> None:
        self.assertRegex(release_job(), r"\n    environment: release\n")

    def test_the_body_is_one_curated_file(self) -> None:
        job = release_job()
        self.assertNotIn("generate_release_notes", job)
        publish = step(job, "Publish release")
        self.assertIn(f"body_path: .github/release-notes/{NOTES}", publish)

    def test_missing_notes_fail_before_publishing(self) -> None:
        # softprops only warns when body_path cannot be read, then publishes
        # without the curated body, so the job checks the file itself.
        job = release_job()
        check = step(job, "Require the curated release notes")
        self.assertIn('notes=".github/release-notes/${GITHUB_REF_NAME}.md"', check)
        self.assertIn('if [ ! -s "${notes}" ]', check)
        self.assertIn("exit 1", check)
        self.assertLess(job.index(check), job.index("- name: Publish release"))

    def test_the_job_never_creates_or_moves_a_tag(self) -> None:
        job = release_job()
        self.assertNotIn("target_commitish", job)
        self.assertNotIn("tag_name", job)
        check = step(job, "Require the tag on origin")
        self.assertIn('git ls-remote --exit-code --tags origin "refs/tags/${GITHUB_REF_NAME}"', check)
        self.assertLess(job.index(check), job.index("- name: Publish release"))

    def test_the_job_runs_on_a_pinned_runner(self) -> None:
        self.assertRegex(release_job(), r"\n    runs-on: ubuntu-24\.04\n")


if __name__ == "__main__":
    unittest.main()
