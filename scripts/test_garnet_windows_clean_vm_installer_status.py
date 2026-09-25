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
import zlib
from pathlib import Path
from unittest import mock

SCRIPT = Path(__file__).with_name("garnet_windows_clean_vm_installer_status.py")
SPEC = importlib.util.spec_from_file_location("garnet_windows_clean_vm_installer_status", SCRIPT)
assert SPEC is not None
status_mod = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules["garnet_windows_clean_vm_installer_status"] = status_mod
SPEC.loader.exec_module(status_mod)


# A real 1x1 PNG, so screenshot fixtures pass the reader's PNG check.
TINY_PNG = bytes.fromhex(
    "89504e470d0a1a0a0000000d49484452000000010000000108060000001f15c4890000000a494441"
    "54789c63000100000500010d0a2db40000000049454e44ae426082"
)


GAMMA_2_2 = (45455).to_bytes(4, "big")  # a valid gAMA body (1/2.2)


def _png(width: int, height: int, colour: int, depth: int, raw: bytes, extra_chunks: tuple = ()) -> bytes:
    """A PNG with correct CRCs around the given filtered image data."""
    def chunk(kind: bytes, body: bytes) -> bytes:
        return len(body).to_bytes(4, "big") + kind + body + (zlib.crc32(kind + body) & 0xFFFFFFFF).to_bytes(4, "big")

    header = width.to_bytes(4, "big") + height.to_bytes(4, "big") + bytes([depth, colour, 0, 0, 0])
    body = b"".join(chunk(kind, data) for kind, data in extra_chunks)
    return (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", header)
        + body
        + chunk(b"IDAT", zlib.compress(raw))
        + chunk(b"IEND", b"")
    )


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
            screenshot.write_bytes(TINY_PNG)

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
            screenshot.write_bytes(TINY_PNG)

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

    def test_the_recorder_blocks_a_non_windows_guest(self) -> None:
        # Codex round 8: recorder and reader apply one guest-OS rule, so a wrong
        # name shows as a blocked fresh-guest gate at record time.
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            for name, data in (("setup.exe", b"x"), ("install.log", b"ok"), ("launch.png", TINY_PNG)):
                (root / name).write_bytes(data)
            (root / "studio-smoke.json").write_text(
                json.dumps({"status": "passed", "source_included": False, "provider_api_called": False}),
                encoding="utf-8",
            )
            for guest_os in ("Ubuntu 24.04 LTS", "Windows Subsystem for Android", "windows11_64Guest"):
                with self.subTest(guest_os=guest_os):
                    record = status_mod.build_proof_record(
                        mode="clean-vm",
                        installer=root / "setup.exe",
                        vm_name="vm",
                        guest_os=guest_os,
                        guest_arch="x64",
                        install_log=root / "install.log",
                        studio_smoke_json=root / "studio-smoke.json",
                        screenshot=root / "launch.png",
                    )
                    gates = {gate.id: gate for gate in record.gates}
                    self.assertEqual("blocked", gates["fresh-guest"].status)
                    self.assertFalse(record.verified)

    def test_the_recorder_applies_the_readers_guest_identity_checks(self) -> None:
        # Codex round 10 (LOW): the recorder passed an empty VM name or an
        # unknown architecture that committed replay rejects.
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / "setup.exe").write_bytes(b"x")
            (root / "install.log").write_bytes(b"ok")
            (root / "launch.png").write_bytes(TINY_PNG)
            (root / "studio-smoke.json").write_text(
                json.dumps({"status": "passed", "source_included": False, "provider_api_called": False}),
                encoding="utf-8",
            )
            for vm_name, guest_arch in (("", "x64"), ("Windows Sandbox", "garbage")):
                with self.subTest(vm_name=vm_name, guest_arch=guest_arch):
                    record = status_mod.build_proof_record(
                        mode="clean-vm",
                        installer=root / "setup.exe",
                        vm_name=vm_name,
                        guest_os="Windows 11 Pro 26100",
                        guest_arch=guest_arch,
                        install_log=root / "install.log",
                        studio_smoke_json=root / "studio-smoke.json",
                        screenshot=root / "launch.png",
                    )
                    gates = {gate.id: gate for gate in record.gates}
                    self.assertEqual("blocked", gates["fresh-guest"].status)
                    self.assertFalse(record.verified)

    def test_the_recorder_blocks_an_empty_log_or_a_non_png_screenshot(self) -> None:
        # Codex round 9: the recorder's gates check the evidence, not only that
        # the files exist.
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / "setup.exe").write_bytes(b"x")
            (root / "studio-smoke.json").write_text(
                json.dumps({"status": "passed", "source_included": False, "provider_api_called": False}),
                encoding="utf-8",
            )
            for log, shot, blocked in ((b"", TINY_PNG, "install-log"), (b"ok", b"not an image", "launch-screenshot")):
                with self.subTest(blocked=blocked):
                    (root / "install.log").write_bytes(log)
                    (root / "launch.png").write_bytes(shot)
                    record = status_mod.build_proof_record(
                        mode="clean-vm",
                        installer=root / "setup.exe",
                        vm_name="Windows Sandbox",
                        guest_os="Windows 11 Pro 26100",
                        guest_arch="x64",
                        install_log=root / "install.log",
                        studio_smoke_json=root / "studio-smoke.json",
                        screenshot=root / "launch.png",
                    )
                    gates = {gate.id: gate for gate in record.gates}
                    self.assertEqual("blocked", gates[blocked].status)
                    self.assertFalse(record.verified)

    def test_the_installer_gate_label_claims_only_what_is_recorded(self) -> None:
        # Codex round 11: committed replay cannot see the .exe, so the label
        # must not say the installer "exists".
        label = {gate.id: gate.label for gate in status_mod.required_gates()}["installer-artifact"]
        self.assertNotIn("exists", label)
        self.assertIn("recorded", label)

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
    (bundle / "launch.png").write_bytes(TINY_PNG)
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
        self.local_root = empty_home / "Desktop" / "dogfood" / "garnet-studio-windows-clean-vm"
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
        (newer / "launch.png").write_bytes(TINY_PNG + b"swapped")

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

    def _link_dir(self, link: Path, target: Path) -> None:
        # Windows refuses symlinks without Developer Mode or the privilege;
        # a skip says so instead of failing for a reason that is not the reader.
        try:
            link.symlink_to(target, target_is_directory=True)
        except OSError as error:
            self.skipTest(f"this host cannot create a directory symlink: {error}")

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
        self._link_dir(self.bundle, outside)
        self._assert_unverified()

    def test_a_symlinked_proof_root_does_not_count(self) -> None:
        root = self.repo / BUNDLES_REL
        outside = Path(self._temp.name) / "outside-root"
        root.rename(outside)
        self._link_dir(root, outside)
        self._assert_unverified()

    def test_a_recorded_path_must_name_the_bundle_directly(self) -> None:
        # Round 2: a recorded path through a link (or a detour) must not count,
        # even when it resolves back into the bundle.
        bundle_rel = (BUNDLES_REL / self.bundle.name).as_posix()
        self._edit_proof(install_log=f"{bundle_rel}/../{self.bundle.name}/install.log")
        self._assert_unverified()

    def test_a_link_inside_a_recorded_path_does_not_count(self) -> None:
        self._link_dir(self.repo / "evidence-alias", self.bundle)
        self._edit_proof(install_log="evidence-alias/install.log")
        self._assert_unverified()

    def _write_verified_local_proof(self) -> None:
        """A verified proof in the local Desktop root: any fallback would show."""
        local = self.local_root / "local-bundle"
        local.mkdir(parents=True)
        for name, data in (("setup.exe", b"x"), ("install.log", b"ok"), ("launch.png", TINY_PNG)):
            (local / name).write_bytes(data)
        (local / "studio-smoke.json").write_text(
            json.dumps({"status": "passed", "source_included": False, "provider_api_called": False}),
            encoding="utf-8",
        )
        record = status_mod.build_proof_record(
            mode="clean-vm",
            installer=local / "setup.exe",
            vm_name="Windows Sandbox",
            guest_os="Windows 11",
            guest_arch="x64",
            install_log=local / "install.log",
            studio_smoke_json=local / "studio-smoke.json",
            screenshot=local / "launch.png",
        )
        status_mod.write_proof(record, local)

    def test_a_newest_entry_that_is_not_a_directory_does_not_bring_back_an_older_bundle(self) -> None:
        (self.repo / BUNDLES_REL / "20260925-0900-nuc").write_text("not a bundle", encoding="utf-8")
        self._assert_unverified()

    def test_a_dangling_link_on_the_proof_path_does_not_fall_back_to_local_evidence(self) -> None:
        self._write_verified_local_proof()
        proofs = self.repo / "proofs"
        proofs.rename(Path(self._temp.name) / "moved-proofs")
        self._link_dir(proofs, Path(self._temp.name) / "missing-target")
        self._assert_unverified()

    def test_duplicate_keys_in_the_record_do_not_count(self) -> None:
        path = self.bundle / status_mod.PROOF_FILE
        data = json.loads(path.read_text(encoding="utf-8"))
        data["verified"] = False
        path.write_text(json.dumps(data)[:-1] + ', "verified": true}', encoding="utf-8")
        status_mod._write_manifest(self.bundle)
        self._assert_unverified()

    def test_duplicate_keys_in_the_smoke_record_do_not_count(self) -> None:
        (self.bundle / "studio-smoke.json").write_text(
            '{"status": "failed", "status": "passed", "source_included": false, "provider_api_called": false}',
            encoding="utf-8",
        )
        status_mod._write_manifest(self.bundle)
        self._assert_unverified()

    def test_a_file_listed_twice_in_the_manifest_does_not_count(self) -> None:
        manifest = self.bundle / "MANIFEST.sha256"
        manifest.write_text(f"{'0' * 64}  install.log\n" + manifest.read_text(encoding="utf-8"), encoding="utf-8")
        self._assert_unverified()

    def test_invalid_utf8_evidence_is_unverified_not_an_exception(self) -> None:
        (self.bundle / "studio-smoke.json").write_bytes(b"\xff")
        status_mod._write_manifest(self.bundle)
        self._assert_unverified()

    def test_an_invalid_utf8_manifest_is_unverified_not_an_exception(self) -> None:
        (self.bundle / "MANIFEST.sha256").write_bytes(b"\xff\n")
        self._assert_unverified()

    def test_a_proof_path_ancestor_that_is_a_file_does_not_fall_back(self) -> None:
        # Codex round 3: a regular file in place of proofs/ or proofs/windows/
        # made the root look absent and let local evidence verify.
        self._write_verified_local_proof()
        for ancestor in ("proofs/windows", "proofs"):
            with self.subTest(ancestor=ancestor):
                target = self.repo / ancestor
                moved = Path(self._temp.name) / f"moved-{ancestor.replace('/', '-')}"
                target.rename(moved)
                target.write_text("not a directory", encoding="utf-8")
                self._assert_unverified()
                target.unlink()
                moved.rename(target)

    def test_an_unreadable_proof_path_does_not_fall_back(self) -> None:
        if os.name != "posix" or os.geteuid() == 0:
            self.skipTest("needs a non-root POSIX host to make a directory unreadable")
        self._write_verified_local_proof()
        proofs = self.repo / "proofs"
        proofs.chmod(0)
        self.addCleanup(proofs.chmod, 0o755)
        self._assert_unverified()

    def test_a_search_only_proof_path_does_not_count(self) -> None:
        # Codex round 4: a directory that can be traversed but not listed (mode
        # 0100) still let the bundle verify; every ancestor must be readable.
        if os.name != "posix" or os.geteuid() == 0:
            self.skipTest("needs a non-root POSIX host to make a directory search-only")
        for ancestor in ("proofs", "proofs/windows"):
            with self.subTest(ancestor=ancestor):
                directory = self.repo / ancestor
                directory.chmod(0o100)
                try:
                    self._assert_unverified()
                finally:
                    directory.chmod(0o755)

    def test_an_evidence_name_must_be_a_verified_file_of_the_bundle(self) -> None:
        # Codex round 3: on NTFS, 'studio-smoke.json:passed' names a stream the
        # manifest never hashes. Case and alias variants must not count either.
        bundle_rel = (BUNDLES_REL / self.bundle.name).as_posix()
        for variant in (f"{bundle_rel}/studio-smoke.json:passed", f"{bundle_rel}/STUDIO-SMOKE.JSON"):
            with self.subTest(variant=variant):
                stream_like = self.repo / variant
                if variant.endswith(":passed"):
                    stream_like.write_text(
                        json.dumps({"status": "passed", "source_included": False, "provider_api_called": False}),
                        encoding="utf-8",
                    )
                self._edit_proof(studio_smoke_json=variant)
                self._assert_unverified()
                if variant.endswith(":passed"):
                    stream_like.unlink()
                    status_mod._write_manifest(self.bundle)

    def test_a_file_added_after_manifest_verification_does_not_count(self) -> None:
        # Codex round 4: evidence must come from the manifest-verified read, not
        # from a later directory scan. The hook adds a passing smoke file right
        # after verification returns, which a later scan would pick up.
        bundle_rel = (BUNDLES_REL / self.bundle.name).as_posix()
        self._edit_proof(studio_smoke_json=f"{bundle_rel}/unlisted-smoke.json")
        name = "_verified_manifest" if hasattr(status_mod, "_verified_manifest") else "_manifest_problem"
        real = getattr(status_mod, name)

        def verify_then_add(bundle: Path, *args: object, **kwargs: object) -> object:
            result = real(bundle, *args, **kwargs)
            (bundle / "unlisted-smoke.json").write_text(
                json.dumps({"status": "passed", "source_included": False, "provider_api_called": False}),
                encoding="utf-8",
            )
            return result

        with mock.patch.object(status_mod, name, verify_then_add):
            self._assert_unverified()

    def _swap_before_first_evidence_read(self, names: set[str], swap: object) -> None:
        """Run `swap` once, just before the reader first reads one of `names`:
        after every metadata check, before the bytes are taken."""
        done: list[bool] = []
        bundle = self.bundle

        def maybe_swap(target: object) -> None:
            path = Path(os.fsdecode(target))
            if not done and path.parent == bundle and path.name in names:
                done.append(True)
                swap()

        real_read_bytes = Path.read_bytes
        real_open = os.open

        def read_bytes(path_self: Path) -> bytes:
            maybe_swap(path_self)
            return real_read_bytes(path_self)

        def os_open(target: object, *args: object, **kwargs: object) -> int:
            maybe_swap(target)
            return real_open(target, *args, **kwargs)

        for patcher in (mock.patch.object(Path, "read_bytes", read_bytes), mock.patch("os.open", os_open)):
            patcher.start()
            self.addCleanup(patcher.stop)

    def test_evidence_swapped_for_hard_links_after_the_checks_does_not_count(self) -> None:
        # Codex round 5: three distinct same-content files pass the metadata
        # checks, then become hard links to one file before the reads.
        smoke_bytes = (self.bundle / "studio-smoke.json").read_bytes()
        for name in ("install.log", "launch.png"):
            (self.bundle / name).write_bytes(smoke_bytes)
        status_mod._write_manifest(self.bundle)

        def swap() -> None:
            for name in ("install.log", "launch.png"):
                (self.bundle / name).unlink()
                os.link(self.bundle / "studio-smoke.json", self.bundle / name)

        self._swap_before_first_evidence_read({"install.log", "launch.png", "studio-smoke.json"}, swap)
        self._assert_unverified()

    def test_a_record_swapped_for_an_outside_link_after_the_checks_does_not_count(self) -> None:
        record = self.bundle / status_mod.PROOF_FILE
        outside = Path(self._temp.name) / "outside-record.json"
        outside.write_bytes(record.read_bytes())

        def swap() -> None:
            record.unlink()
            record.symlink_to(outside)

        self._swap_before_first_evidence_read({status_mod.PROOF_FILE}, swap)
        self._assert_unverified()

    def test_a_filesystem_without_stable_file_identity_fails_closed(self) -> None:
        # Codex round 6: Python promises inode uniqueness only when st_ino is
        # nonzero. With st_ino 0 the identity checks prove nothing, so the
        # reader must report the bundle unverified, not skip them.
        real_lstat, real_fstat = os.lstat, os.fstat

        def zero_ino(result: os.stat_result) -> os.stat_result:
            fields = list(result[:10])
            fields[1] = 0
            return os.stat_result(fields)

        with mock.patch("os.lstat", lambda *a, **k: zero_ino(real_lstat(*a, **k))), mock.patch(
            "os.fstat", lambda *a, **k: zero_ino(real_fstat(*a, **k))
        ):
            self._assert_unverified()

    def test_evidence_files_must_not_be_hard_links(self) -> None:
        # Codex round 4: two names for one file let one smoke record fill every
        # role. Git never checks out hard links, so a bundle file has one link.
        smoke = self.bundle / "studio-smoke.json"
        for name in ("install.log", "launch.png"):
            (self.bundle / name).unlink()
            try:
                os.link(smoke, self.bundle / name)
            except OSError as error:
                self.skipTest(f"this host cannot create a hard link: {error}")
        status_mod._write_manifest(self.bundle)
        self._assert_unverified()

    def test_the_three_evidence_files_are_distinct(self) -> None:
        data = json.loads((self.bundle / status_mod.PROOF_FILE).read_text(encoding="utf-8"))
        self._edit_proof(screenshot=data["install_log"])
        self._assert_unverified()

    def test_the_newest_bundle_must_be_named_timestamp_dash_host(self) -> None:
        # Codex round 3: a name with only the timestamp, or a malformed tail,
        # is reported (fail closed), not accepted and not skipped.
        for name in ("20260925-0900", "20260925-0900malformed"):
            with self.subTest(name=name):
                bundle = _record_committed_bundle(self.repo, name)
                self._assert_unverified()
                for path in bundle.iterdir():
                    path.unlink()
                bundle.rmdir()

    def test_a_bundle_holds_only_regular_files(self) -> None:
        # A link or a subdirectory inside the bundle, even one no field names,
        # breaks "no link anywhere from the repository down".
        self._link_dir(self.bundle / "extra", Path(self._temp.name))
        self._assert_unverified()

    def test_the_guest_must_be_windows(self) -> None:
        # Codex round 7: a committed bundle whose guest reads Ubuntu, macOS or
        # FreeBSD still verified as a Windows clean-VM proof.
        for guest_os in (
            "Ubuntu 24.04 LTS",
            "macOS 15.4",
            "FreeBSD 14.2",
            "Windows Subsystem for Linux (Ubuntu)",
            "Windows Subsystem for Android",  # Codex round 8
        ):
            with self.subTest(guest_os=guest_os):
                self._edit_proof(guest_os=guest_os)
                self._assert_unverified()

    def test_windows_guest_names_verify(self) -> None:
        for guest_os in (
            "Windows 11 Pro 26100",
            "Microsoft Windows 11 Enterprise 10.0.26100",
            "Windows 10 Pro 19045",
            "Windows Server 2025",
            "Microsoft Windows Server 2022 Datacenter",
        ):
            with self.subTest(guest_os=guest_os):
                self._edit_proof(guest_os=guest_os)
                self.assertTrue(status_mod.read_status().clean_vm_verified)

    def test_the_install_log_and_screenshot_must_hold_evidence(self) -> None:
        # Codex round 9: an empty install log, or a screenshot that is not an
        # image (empty, or the smoke JSON copied in), still verified.
        smoke = (self.bundle / "studio-smoke.json").read_bytes()
        for name, data in (("install.log", b""), ("launch.png", b""), ("launch.png", smoke), ("launch.png", b"fake image")):
            with self.subTest(name=name, size=len(data)):
                original = (self.bundle / name).read_bytes()
                (self.bundle / name).write_bytes(data)
                status_mod._write_manifest(self.bundle)
                self._assert_unverified()
                (self.bundle / name).write_bytes(original)
                status_mod._write_manifest(self.bundle)

    def test_a_structurally_broken_png_does_not_count(self) -> None:
        # Codex round 10: a header-only check accepted TINY_PNG[:24]. The PNG
        # must be complete: valid chunk CRCs, IHDR first, IDAT, IEND last, and
        # image data that decompresses to the size the header implies.
        bad_crc = bytearray(TINY_PNG)
        bad_crc[29] ^= 0xFF  # a byte of the IHDR CRC
        no_iend = TINY_PNG[: TINY_PNG.index(b"IEND") - 4]
        short_data = bytearray(TINY_PNG)
        short_data[20:24] = (2).to_bytes(4, "big")  # IHDR claims height 2
        short_data[29:33] = (zlib.crc32(bytes(short_data[12:29])) & 0xFFFFFFFF).to_bytes(4, "big")  # valid CRC
        for label, data in (
            ("truncated header", TINY_PNG[:24]),
            ("bad crc", bytes(bad_crc)),
            ("no IEND", no_iend),
            ("trailing bytes", TINY_PNG + b"x"),
            ("data shorter than the header implies", bytes(short_data)),
        ):
            with self.subTest(label=label):
                self.assertFalse(status_mod.is_png_screenshot(data))
                (self.bundle / "launch.png").write_bytes(data)
                status_mod._write_manifest(self.bundle)
                self._assert_unverified()
        self.assertTrue(status_mod.is_png_screenshot(TINY_PNG))

    def test_an_undecodable_or_unsupported_png_does_not_count(self) -> None:
        # Codex round 11: CRC-correct PNGs that no decoder can render, or that
        # the reader does not support, must not pass (and must not raise).
        huge = _png(0xFFFFFFFF, 0xFFFFFFFF, 6, 16, b"\x00")
        for label, data in (
            ("scanline filter 5", _png(1, 1, 6, 8, b"\x05\x00\x00\x00\x00")),
            ("truecolour at depth 4", _png(1, 1, 2, 4, b"\x00\x00\x00")),
            ("palette image", _png(1, 1, 3, 8, b"\x00\x00", ((b"PLTE", b"\x00\x00\x00"),))),
            ("4-billion-pixel header", huge),
            ("over the dimension cap", _png(20000, 1, 0, 8, b"\x00" + bytes(20000))),
        ):
            with self.subTest(label=label):
                self.assertFalse(status_mod.is_png_screenshot(data))
                (self.bundle / "launch.png").write_bytes(data)
                status_mod._write_manifest(self.bundle)
                self._assert_unverified()
        for label, data in (
            ("greyscale 16", _png(2, 2, 0, 16, (b"\x00" + bytes(4)) * 2)),
            ("rgba 8 with every filter", _png(1, 5, 6, 8, b"".join(bytes([f]) + bytes(4) for f in range(5)))),
        ):
            with self.subTest(label=label):
                self.assertTrue(status_mod.is_png_screenshot(data))

    def test_png_chunk_structure_must_be_legal(self) -> None:
        # Codex round 11: a duplicate IHDR, a split IDAT run, an unknown
        # critical chunk, or a palette in a greyscale image must not pass.
        raw = b"\x00" + bytes(4)

        def chunk(kind: bytes, body: bytes) -> bytes:
            return len(body).to_bytes(4, "big") + kind + body + (zlib.crc32(kind + body) & 0xFFFFFFFF).to_bytes(4, "big")

        ihdr = chunk(b"IHDR", (1).to_bytes(4, "big") * 2 + bytes([8, 6, 0, 0, 0]))
        data = zlib.compress(raw)
        sig, iend = b"\x89PNG\r\n\x1a\n", chunk(b"IEND", b"")
        for label, png in (
            ("duplicate IHDR", sig + ihdr + ihdr + chunk(b"IDAT", data) + iend),
            ("split IDAT run", sig + ihdr + chunk(b"IDAT", data[:4]) + chunk(b"tEXt", b"k\x00v") + chunk(b"IDAT", data[4:]) + iend),
            ("unknown critical chunk", sig + ihdr + chunk(b"ABCD", b"") + chunk(b"IDAT", data) + iend),
            ("palette in greyscale", _png(1, 1, 0, 8, b"\x00\x00", ((b"PLTE", b"\x00\x00\x00"),))),
        ):
            with self.subTest(label=label):
                self.assertFalse(status_mod.is_png_screenshot(png))
        ok = sig + ihdr + chunk(b"tEXt", b"k\x00v") + chunk(b"IDAT", data[:4]) + chunk(b"IDAT", data[4:]) + iend
        self.assertTrue(status_mod.is_png_screenshot(ok), "ancillary chunks and a consecutive IDAT run are fine")

    def test_standard_ancillary_chunks_must_be_well_formed(self) -> None:
        # Codex round 12: an empty tRNS or pHYs passed. Standard fixed-size
        # ancillary chunks are checked for size, colour type, count and order.
        cases = (
            ("empty tRNS", 0, 8, b"\x00\x00", ((b"tRNS", b""),)),
            ("tRNS in rgba", 6, 8, b"\x00" + bytes(4), ((b"tRNS", b"\x00" * 6),)),
            ("short pHYs", 0, 8, b"\x00\x00", ((b"pHYs", b"\x00" * 8),)),
            ("empty pHYs", 0, 8, b"\x00\x00", ((b"pHYs", b""),)),
            ("pHYs with unit 7", 0, 8, b"\x00\x00", ((b"pHYs", b"\x00" * 8 + b"\x07"),)),
            ("sRGB intent 9", 2, 8, b"\x00" + bytes(3), ((b"sRGB", b"\x09"),)),
            ("two gAMA", 0, 8, b"\x00\x00", ((b"gAMA", GAMMA_2_2), (b"gAMA", GAMMA_2_2))),
        )
        for label, colour, depth, raw, extra in cases:
            with self.subTest(label=label):
                self.assertFalse(status_mod.is_png_screenshot(_png(1, 1, colour, depth, raw, extra)))
        after_idat = _png(1, 1, 0, 8, b"\x00\x00")
        cut = after_idat.index(b"IEND") - 4
        gama = (4).to_bytes(4, "big") + b"gAMA" + GAMMA_2_2 + (zlib.crc32(b"gAMA" + GAMMA_2_2) & 0xFFFFFFFF).to_bytes(4, "big")
        problem = status_mod.png_screenshot_problem(after_idat[:cut] + gama + after_idat[cut:])
        self.assertIn("before the image data", problem or "", "a valid gamma rejected for its position")
        two = status_mod.png_screenshot_problem(_png(1, 1, 0, 8, b"\x00\x00", ((b"gAMA", GAMMA_2_2), (b"gAMA", GAMMA_2_2))))
        self.assertIn("more than once", two or "", "a valid gamma rejected for repeating")
        for label, colour, raw, extra in (
            ("grey tRNS", 0, b"\x00\x00", ((b"tRNS", b"\x00\x01"),)),
            ("pHYs metres", 2, b"\x00" + bytes(3), ((b"pHYs", (3780).to_bytes(4, "big") * 2 + b"\x01"),)),
            ("sRGB and text", 6, b"\x00" + bytes(4), ((b"sRGB", b"\x00"), (b"tEXt", b"Software\x00Snipping Tool"))),
        ):
            with self.subTest(label=label):
                self.assertTrue(status_mod.is_png_screenshot(_png(1, 1, colour, 8, raw, extra)))

    def test_palette_and_ancillary_values_must_be_legal(self) -> None:
        # Codex round 12: payload values and PLTE-dependent chunks. A
        # truecolour PNG may carry a suggested palette of 1-256 RGB triples.
        rgb = b"\x00" + bytes(3)
        cases = (
            ("PLTE of 1 byte", 2, rgb, ((b"PLTE", b"\x00"),)),
            ("PLTE of 0 bytes", 2, rgb, ((b"PLTE", b""),)),
            ("PLTE of 257 entries", 6, b"\x00" + bytes(4), ((b"PLTE", bytes(771)),)),
            ("gAMA of 0", 2, rgb, ((b"gAMA", bytes(4)),)),
            ("gAMA above 2^31-1", 2, rgb, ((b"gAMA", (0x80000000).to_bytes(4, "big")),)),
            ("pHYs above 2^31-1", 2, rgb, ((b"pHYs", (0x80000000).to_bytes(4, "big") + (1).to_bytes(4, "big") + b"\x01"),)),
            ("cHRM above 2^31-1", 2, rgb, ((b"cHRM", (0x80000000).to_bytes(4, "big") + bytes(28)),)),
            ("sBIT of 0", 2, rgb, ((b"sBIT", b"\x00\x08\x08"),)),
            ("sBIT over depth", 2, rgb, ((b"sBIT", b"\x09\x08\x08"),)),
            ("tIME month 13", 2, rgb, ((b"tIME", (2026).to_bytes(2, "big") + bytes([13, 1, 0, 0, 0])),)),
            ("tRNS before PLTE", 2, rgb, ((b"tRNS", bytes(6)), (b"PLTE", bytes(3)))),
            ("hIST without PLTE", 2, rgb, ((b"hIST", bytes(2)),)),
            ("hIST of the wrong size", 2, rgb, ((b"PLTE", bytes(6)), (b"hIST", bytes(2)))),
        )
        for label, colour, raw, extra in cases:
            with self.subTest(label=label):
                self.assertFalse(status_mod.is_png_screenshot(_png(1, 1, colour, 8, raw, extra)))
        for label, colour, raw, extra in (
            ("suggested palette of 2", 2, rgb, ((b"PLTE", bytes(6)),)),
            ("PLTE, tRNS and hIST in order", 2, rgb, ((b"PLTE", bytes(6)), (b"tRNS", bytes(6)), (b"hIST", bytes(4)))),
            ("gAMA 1/2.2", 2, rgb, ((b"gAMA", (45455).to_bytes(4, "big")),)),
            ("tIME", 2, rgb, ((b"tIME", (2026).to_bytes(2, "big") + bytes([9, 25, 12, 0, 0])),)),
        ):
            with self.subTest(label=label):
                self.assertTrue(status_mod.is_png_screenshot(_png(1, 1, colour, 8, raw, extra)))

    def test_ancillary_chunks_keep_their_order_around_plte(self) -> None:
        # Codex round 13: gAMA, cHRM, sRGB, iCCP and sBIT must precede PLTE;
        # bKGD, tRNS and hIST must follow it (PNG chunk ordering).
        rgb = b"\x00" + bytes(3)
        plte = (b"PLTE", bytes(6))
        for kind, body in (
            (b"gAMA", GAMMA_2_2),
            (b"cHRM", (31270).to_bytes(4, "big") * 8),
            (b"sRGB", b"\x00"),
            (b"sBIT", b"\x08\x08\x08"),
        ):
            with self.subTest(after_plte=kind):
                problem = status_mod.png_screenshot_problem(_png(1, 1, 2, 8, rgb, (plte, (kind, body))))
                self.assertIn("before PLTE", problem or "")
                self.assertIsNone(status_mod.png_screenshot_problem(_png(1, 1, 2, 8, rgb, ((kind, body), plte))))
        problem = status_mod.png_screenshot_problem(_png(1, 1, 2, 8, rgb, ((b"bKGD", bytes(6)), plte)))
        self.assertIn("after PLTE", problem or "")
        self.assertIsNone(status_mod.png_screenshot_problem(_png(1, 1, 2, 8, rgb, (plte, (b"bKGD", bytes(6))))))

    def test_an_oversized_png_is_rejected_by_name(self) -> None:
        huge = _png(2147483647, 2147483647, 6, 16, b"\x00")
        problem = status_mod.png_screenshot_problem(huge)
        self.assertIsNotNone(problem)
        self.assertIn("16384", problem)
        (self.bundle / "launch.png").write_bytes(huge)
        status_mod._write_manifest(self.bundle)
        self._assert_unverified()

    def test_a_png_with_zero_size_does_not_count(self) -> None:
        # Codex round 11: build the zero-width image with consistent CRCs, so
        # the dimension check itself is what rejects it.
        zero = _png(0, 1, 6, 8, b"\x00")
        problem = status_mod.png_screenshot_problem(zero)
        self.assertIsNotNone(problem)
        self.assertIn("between 1 and", problem, "the dimension check, not a later size check, rejects it")
        (self.bundle / "launch.png").write_bytes(zero)
        status_mod._write_manifest(self.bundle)
        self._assert_unverified()

    def test_created_at_must_be_a_time_with_a_zone(self) -> None:
        # Codex round 10 (LOW): created_at was only checked to be a string.
        for value in ("", "not-a-date", "2026-09-24T12:00:00", "2026-09-24T12:00:00+00:99"):
            with self.subTest(created_at=value):
                self._edit_proof(created_at=value)
                self._assert_unverified()

    def test_the_bundle_name_must_carry_a_real_time(self) -> None:
        _record_committed_bundle(self.repo, "99999999-9999-nuc")
        self._assert_unverified()

    def test_nonfinite_json_constants_do_not_count(self) -> None:
        # Codex round 10 (LOW): "strict JSON" accepted NaN in an extra field.
        record = self.bundle / status_mod.PROOF_FILE
        original = record.read_bytes()
        record.write_bytes(original.rstrip()[:-1] + b', "extra": NaN}')
        status_mod._write_manifest(self.bundle)
        self._assert_unverified()
        record.write_bytes(original)
        (self.bundle / "studio-smoke.json").write_text(
            '{"status": "passed", "source_included": false, "provider_api_called": false, "extra": Infinity}',
            encoding="utf-8",
        )
        status_mod._write_manifest(self.bundle)
        self._assert_unverified()

    def test_vm_tool_type_identifiers_are_out_of_contract(self) -> None:
        # Codex round 8: the recorder takes the guest's own systeminfo OS name.
        # A hypervisor's guest-type identifier is not that name, so it fails
        # closed rather than being mapped.
        for guest_os in ("windows11_64Guest", "Windows11_64"):
            with self.subTest(guest_os=guest_os):
                self._edit_proof(guest_os=guest_os)
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
