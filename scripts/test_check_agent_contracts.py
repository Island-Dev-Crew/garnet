#!/usr/bin/env python3
"""Regression tests for the AGENTS.md documentation contract checker."""
from __future__ import annotations

import importlib.util
from pathlib import Path
import shutil
import tempfile
import unittest

SCRIPT = Path(__file__).with_name("check-agent-contracts.py")
SPEC = importlib.util.spec_from_file_location("check_agent_contracts", SCRIPT)
assert SPEC is not None
checker = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(checker)


def copy_contract_tree(target: Path) -> None:
    for rel in checker.REQUIRED:
        source = checker.ROOT / rel
        destination = target / rel
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, destination)

    for manifest in checker.ROOT.rglob("Cargo.toml"):
        if checker.skip_path(manifest):
            continue
        destination = target / manifest.relative_to(checker.ROOT)
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(manifest, destination)


class AgentContractCheckerTests(unittest.TestCase):
    def with_tree(self, callback) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            copy_contract_tree(root)
            callback(root)

    def test_repo_snapshot_validates(self) -> None:
        checker.validate(checker.ROOT)

    def test_rejects_placeholder_contract_text(self) -> None:
        def case(root: Path) -> None:
            (root / "examples/AGENTS.md").write_text(
                "# AGENTS.md - Examples Contract\n"
                "\n"
                "## Scope\n"
                "\n"
                "TODO placeholder text for the examples folder.\n"
                "\n"
                "## Stable Contracts\n"
                "\n"
                "- production readiness parser/interpreter/checker MVP demonstration.\n"
                "\n"
                "## Required Checks\n"
                "\n"
                "Run tests.\n",
                encoding="utf-8",
            )
            with self.assertRaises(checker.ContractError):
                checker.validate(root)

        self.with_tree(case)

    def test_rejects_generic_contract_missing_local_terms(self) -> None:
        def case(root: Path) -> None:
            (root / "garnet-memory-v0.3/AGENTS.md").write_text(
                "# AGENTS.md - Memory Core Contract\n"
                "\n"
                "## Scope\n"
                "\n"
                "Owns this folder.\n"
                "\n"
                "## Stable Contracts\n"
                "\n"
                "- Keep the implementation correct.\n"
                "\n"
                "## Required Checks\n"
                "\n"
                "```sh\n"
                "cargo test -p garnet-memory\n"
                "```\n",
                encoding="utf-8",
            )
            with self.assertRaises(checker.ContractError):
                checker.validate(root)

        self.with_tree(case)

    def test_rejects_crate_without_local_contract(self) -> None:
        def case(root: Path) -> None:
            new_crate = root / "garnet-new-surface"
            new_crate.mkdir()
            (new_crate / "Cargo.toml").write_text(
                '[package]\nname = "garnet-new-surface"\nversion = "0.1.0"\n',
                encoding="utf-8",
            )
            with self.assertRaises(checker.ContractError):
                checker.validate(root)

        self.with_tree(case)

    def test_rejects_broken_internal_links(self) -> None:
        def case(root: Path) -> None:
            path = root / "C_Language_Specification/AGENTS.md"
            path.write_text(
                path.read_text(encoding="utf-8")
                + "\n[missing contract](DOES_NOT_EXIST.md)\n",
                encoding="utf-8",
            )
            with self.assertRaises(checker.ContractError):
                checker.validate(root)

        self.with_tree(case)


if __name__ == "__main__":
    unittest.main()
