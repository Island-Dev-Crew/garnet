#!/usr/bin/env python3
"""Regression tests for Windows Studio clean-VM installer proof accounting."""
from __future__ import annotations

import importlib.util
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

SCRIPT = Path(__file__).with_name("garnet_windows_clean_vm_installer_status.py")
SPEC = importlib.util.spec_from_file_location("garnet_windows_clean_vm_installer_status", SCRIPT)
assert SPEC is not None
status_mod = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules["garnet_windows_clean_vm_installer_status"] = status_mod
SPEC.loader.exec_module(status_mod)


class GarnetWindowsCleanVmInstallerStatusTests(unittest.TestCase):
    def test_target_matrix_names_architectures_without_overclaiming(self) -> None:
        status = status_mod.read_status(Path("missing-root"))
        targets = {target.id: target for target in status.package_targets}

        self.assertFalse(status.clean_vm_verified)
        self.assertEqual("x86_64-pc-windows-msvc", targets["studio-windows-x64-nsis"].rust_target)
        self.assertEqual("aarch64-pc-windows-msvc", targets["studio-windows-arm64-nsis"].rust_target)
        self.assertEqual("i686-pc-windows-msvc", targets["studio-windows-x86-nsis"].rust_target)
        self.assertEqual("first-clean-vm-target", targets["studio-windows-x64-nsis"].status)
        self.assertEqual("planned-after-x64-proof", targets["studio-windows-arm64-nsis"].status)
        self.assertEqual("deferred-until-user-demand", targets["studio-windows-x86-nsis"].status)
        self.assertIn("clean Windows VM guest identity", status.blocked_by)
        self.assertIn("winget install path is verified", status.forbidden_claims)

    def test_current_host_record_is_not_clean_vm_verified(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            installer = root / "Garnet-Studio-setup.exe"
            install_log = root / "install.log"
            smoke = root / "studio-smoke.json"
            screenshot = root / "launch.png"
            installer.write_bytes(b"fake installer")
            install_log.write_text("exit_code=0\n", encoding="utf-8")
            smoke.write_text(
                json.dumps(
                    {
                        "status": "passed",
                        "source_included": False,
                        "provider_api_called": False,
                    }
                ),
                encoding="utf-8",
            )
            screenshot.write_bytes(b"fake image")

            record = status_mod.build_proof_record(
                mode="current-host",
                installer=installer,
                vm_name="devbox",
                guest_os="Windows 11",
                guest_arch="x64",
                install_log=install_log,
                studio_smoke_json=smoke,
                screenshot=screenshot,
            )

            self.assertFalse(record.verified)
            gates = {gate.id: gate for gate in record.gates}
            self.assertEqual("blocked", gates["fresh-guest"].status)
            self.assertEqual("pass", gates["studio-smoke"].status)

            status_mod.write_proof(record, root / "current-host-proof")
            status = status_mod.read_status(root)

            self.assertFalse(status.clean_vm_verified)
            self.assertEqual(["clean Windows VM guest identity"], status.blocked_by)

    def test_clean_vm_record_writes_manifest_and_verifies_all_gates(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            bundle = root / "bundle"
            installer = root / "Garnet-Studio-setup.exe"
            install_log = root / "install.log"
            smoke = root / "studio-smoke.json"
            screenshot = root / "launch.png"
            installer.write_bytes(b"fake installer")
            install_log.write_text("exit_code=0\n", encoding="utf-8")
            smoke.write_text(
                json.dumps(
                    {
                        "status": "passed",
                        "source_included": False,
                        "provider_api_called": False,
                    }
                ),
                encoding="utf-8",
            )
            screenshot.write_bytes(b"fake image")

            record = status_mod.build_proof_record(
                mode="clean-vm",
                installer=installer,
                vm_name="garnet-win11-clean",
                guest_os="Windows 11 23H2",
                guest_arch="x64",
                install_log=install_log,
                studio_smoke_json=smoke,
                screenshot=screenshot,
            )
            path = status_mod.write_proof(record, bundle)

            self.assertTrue(record.verified)
            self.assertTrue(path.exists())
            self.assertTrue((bundle / "MANIFEST.sha256").exists())
            status = status_mod.read_status(root)
            self.assertTrue(status.clean_vm_verified)
            self.assertEqual("clean-vm-proof-verified", status.status)
            self.assertFalse(status.blocked_by)

    def test_cli_json_and_markdown_are_honest_without_evidence(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            json_output = subprocess.check_output(
                [
                    sys.executable,
                    str(SCRIPT),
                    "--format",
                    "json",
                    "--evidence-root",
                    temp,
                ],
                text=True,
            )
            data = json.loads(json_output)
            self.assertFalse(data["clean_vm_verified"])
            self.assertIn("proof-contract-ready-clean-vm-open", data["status"])
            self.assertIn("signed Windows MSI is available", data["forbidden_claims"])

            markdown = subprocess.check_output(
                [sys.executable, str(SCRIPT), "--evidence-root", temp],
                text=True,
            )
            self.assertIn("Clean VM verified: `false`", markdown)
            self.assertIn("Windows 32-bit remains deferred", markdown)


BUNDLES_REL = Path("proofs/windows/studio-clean-vm")


def _record_committed_bundle(
    repo: Path,
    name: str,
    *,
    guest_arch: str = "x64",
    log_inside_bundle: bool = True,
) -> Path:
    """Record a bundle the way the W2 handoff does: from the repo root, with
    repo-relative paths, into proofs/windows/studio-clean-vm/<name>/."""
    bundle_rel = BUNDLES_REL / name
    bundle = repo / bundle_rel
    bundle.mkdir(parents=True)
    (repo / "target").mkdir(exist_ok=True)
    (repo / "target" / "Garnet-Studio-setup.exe").write_bytes(b"fake installer")
    (bundle / "commands.txt").write_text("recorded by the test\n", encoding="utf-8")
    log_rel = bundle_rel / "install.log" if log_inside_bundle else Path("install.log")
    (repo / log_rel).write_text("exit_code=0\n", encoding="utf-8")
    (bundle / "studio-smoke.json").write_text(
        json.dumps({"status": "passed", "source_included": False, "provider_api_called": False}),
        encoding="utf-8",
    )
    (bundle / "launch.png").write_bytes(b"fake image")
    previous = Path.cwd()
    os.chdir(repo)
    try:
        record = status_mod.build_proof_record(
            mode="clean-vm",
            installer=Path("target/Garnet-Studio-setup.exe"),
            vm_name="Windows Sandbox",
            guest_os="Windows 11 Pro 26100",
            guest_arch=guest_arch,
            install_log=log_rel,
            studio_smoke_json=bundle_rel / "studio-smoke.json",
            screenshot=bundle_rel / "launch.png",
        )
        status_mod.write_proof(record, bundle_rel)
    finally:
        os.chdir(previous)
    return bundle


class _CommittedRepoCase(unittest.TestCase):
    """A temporary repo as the reporter's ROOT, with no local Desktop proof."""

    def setUp(self) -> None:
        self._temp = tempfile.TemporaryDirectory()
        self.addCleanup(self._temp.cleanup)
        self.repo = Path(self._temp.name) / "repo"
        self.repo.mkdir()
        empty_home = Path(self._temp.name) / "home"
        for patcher in (
            mock.patch.object(status_mod, "ROOT", self.repo),
            mock.patch.object(
                status_mod,
                "default_evidence_root",
                lambda home=None: empty_home / "Desktop" / "dogfood" / "garnet-studio-windows-clean-vm",
            ),
        ):
            patcher.start()
            self.addCleanup(patcher.stop)


class CommittedCleanVmBundleTests(_CommittedRepoCase):
    """T5a, Jon's path (a): a clean-VM proof committed to the repo is read on any
    host, and only after its manifest and gates check out."""

    def test_committed_bundle_is_read_when_no_root_is_given(self) -> None:
        _record_committed_bundle(self.repo, "20260924-1200-nuc")

        status = status_mod.read_status()

        self.assertTrue(status.clean_vm_verified)
        self.assertEqual("clean-vm-proof-verified", status.status)
        self.assertEqual("committed:proofs/windows/studio-clean-vm/20260924-1200-nuc", status.proof_source)
        self.assertFalse(status.blocked_by)

    def test_tampered_committed_bundle_is_not_verified(self) -> None:
        bundle = _record_committed_bundle(self.repo, "20260924-1200-nuc")
        (bundle / "install.log").write_text("exit_code=1\n", encoding="utf-8")

        status = status_mod.read_status()

        self.assertFalse(status.clean_vm_verified)
        self.assertIn("committed clean-VM bundle integrity", status.blocked_by)

    def test_committed_bundle_evidence_must_sit_inside_the_bundle(self) -> None:
        _record_committed_bundle(self.repo, "20260924-1200-nuc", log_inside_bundle=False)

        status = status_mod.read_status()

        self.assertFalse(status.clean_vm_verified)
        self.assertIn("committed clean-VM bundle integrity", status.blocked_by)

    def test_committed_bundle_must_be_an_x64_guest(self) -> None:
        _record_committed_bundle(self.repo, "20260924-1200-nuc", guest_arch="arm64")

        status = status_mod.read_status()

        self.assertFalse(status.clean_vm_verified)

    def test_newest_committed_bundle_decides_without_falling_back(self) -> None:
        _record_committed_bundle(self.repo, "20260924-1200-nuc")
        newer = _record_committed_bundle(self.repo, "20260925-0900-nuc")
        (newer / "launch.png").write_bytes(b"swapped image")

        status = status_mod.read_status()

        self.assertFalse(status.clean_vm_verified)
        self.assertEqual("committed:proofs/windows/studio-clean-vm/20260925-0900-nuc", status.proof_source)

    def test_unreadable_committed_proof_is_not_verified(self) -> None:
        bundle = _record_committed_bundle(self.repo, "20260924-1200-nuc")
        (bundle / "windows-clean-vm-installer-proof.json").write_text("{not json", encoding="utf-8")

        status = status_mod.read_status()

        self.assertFalse(status.clean_vm_verified)
        self.assertIn("committed clean-VM bundle integrity", status.blocked_by)

    def test_explicit_evidence_root_still_wins(self) -> None:
        _record_committed_bundle(self.repo, "20260924-1200-nuc")

        status = status_mod.read_status(Path(self._temp.name) / "missing-root")

        self.assertFalse(status.clean_vm_verified)
        self.assertEqual("none", status.proof_source)



class CommittedCleanVmBundleAdversarialTests(_CommittedRepoCase):
    """T5a (Codex review of #598): a committed bundle that is malformed, relinked
    or stripped of its guest identity never counts as verified, and never
    raises. Each case refreshes the manifest, so only the named defect remains."""

    def setUp(self) -> None:
        super().setUp()
        self.bundle = _record_committed_bundle(self.repo, "20260924-1200-nuc")

    def _edit_proof(self, **fields: object) -> None:
        path = self.bundle / status_mod.PROOF_FILE
        data = json.loads(path.read_text(encoding="utf-8"))
        data.update(fields)
        path.write_text(json.dumps(data), encoding="utf-8")
        status_mod._write_manifest(self.bundle)

    def _assert_unverified(self) -> None:
        status = status_mod.read_status()
        self.assertFalse(status.clean_vm_verified, status.proof_source)
        self.assertIn("committed clean-VM bundle integrity", status.blocked_by)

    def test_the_unedited_fixture_verifies(self) -> None:
        self.assertTrue(status_mod.read_status().clean_vm_verified)

    def test_verified_must_be_the_literal_true(self) -> None:
        for value in ("false", [False], 1, "true"):
            with self.subTest(value=value):
                self._edit_proof(verified=value)
                self._assert_unverified()

    def test_a_missing_newest_proof_cannot_bring_back_an_older_bundle(self) -> None:
        newer = _record_committed_bundle(self.repo, "20260925-0900-nuc")
        (newer / status_mod.PROOF_FILE).unlink()
        self._assert_unverified()

    def test_a_symlinked_bundle_outside_the_repo_does_not_count(self) -> None:
        outside = Path(self._temp.name) / "outside"
        self.bundle.rename(outside)
        self.bundle.symlink_to(outside, target_is_directory=True)
        self._assert_unverified()

    def test_a_symlinked_proof_root_does_not_count(self) -> None:
        root = self.repo / BUNDLES_REL
        outside = Path(self._temp.name) / "outside-root"
        root.rename(outside)
        root.symlink_to(outside, target_is_directory=True)
        self._assert_unverified()

    def test_the_fresh_guest_identity_is_checked_not_its_gate_label(self) -> None:
        self._edit_proof(guest_os="", vm_name="")
        self._assert_unverified()

    def test_the_claim_boundary_cannot_be_removed(self) -> None:
        self._edit_proof(forbidden_claims=[])
        self._assert_unverified()

    def test_a_gate_listed_twice_does_not_count(self) -> None:
        data = json.loads((self.bundle / status_mod.PROOF_FILE).read_text(encoding="utf-8"))
        self._edit_proof(gates=[*data["gates"], data["gates"][0]])
        self._assert_unverified()

    def test_a_non_string_architecture_is_unverified_not_an_exception(self) -> None:
        self._edit_proof(guest_arch=64)
        self._assert_unverified()

    def test_an_array_smoke_record_is_unverified_not_an_exception(self) -> None:
        (self.bundle / "studio-smoke.json").write_text("[]", encoding="utf-8")
        status_mod._write_manifest(self.bundle)
        self._assert_unverified()

    def test_windows_separators_in_recorded_paths_still_verify(self) -> None:
        data = json.loads((self.bundle / status_mod.PROOF_FILE).read_text(encoding="utf-8"))
        self._edit_proof(
            **{field: data[field].replace("/", "\\") for field in ("install_log", "studio_smoke_json", "screenshot")}
        )
        self.assertTrue(status_mod.read_status().clean_vm_verified)


if __name__ == "__main__":
    unittest.main()
