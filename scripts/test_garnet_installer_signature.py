#!/usr/bin/env python3
"""D-26: docs/install.sh verifies SHA256SUMS.asc against the pinned key when gpg exists.

Each case runs the real installer against a file:// release directory holding a
stand-in `garnet` tarball, with throwaway keys made for the test. The installer is
never given a test-only mode: it is pointed at the test keys the way a mirror
would be (GARNET_SIGNING_KEYS_URL and GARNET_SIGNING_KEY_FPR).
"""
from __future__ import annotations

import hashlib
import io
import os
import re
import shutil
import subprocess
import tarfile
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
INSTALLER = ROOT / "docs" / "install.sh"
KEYRING = ROOT / "docs" / "garnet-release-keys.asc"
RELEASE_KEY_0_8 = "04D56F91F03817DDFFEBC62AC14DF6E713956ED1"
VERSION = "0.8.2"
TRIPLES = (
    "x86_64-unknown-linux-gnu",
    "aarch64-unknown-linux-gnu",
    "x86_64-apple-darwin",
    "aarch64-apple-darwin",
)
GPG = shutil.which("gpg")


def gpg(home: Path, *args: str, stdin: bytes | None = None) -> subprocess.CompletedProcess[bytes]:
    return subprocess.run(
        ["gpg", "--homedir", str(home), "--batch", "--yes", *args],
        input=stdin,
        capture_output=True,
        check=True,
    )


def make_key(home: Path, name: str) -> str:
    home.mkdir(mode=0o700, parents=True)
    gpg(home, "--passphrase", "", "--pinentry-mode", "loopback",
        "--quick-gen-key", f"{name} <{name}@example.invalid>", "ed25519", "sign", "never")
    listing = gpg(home, "--with-colons", "--list-secret-keys").stdout.decode()
    return re.search(r"^fpr:+([0-9A-F]{40}):", listing, re.M).group(1)


def stand_in_tarball() -> bytes:
    script = f"#!/bin/sh\necho 'garnet {VERSION}'\n".encode()
    buf = io.BytesIO()
    with tarfile.open(fileobj=buf, mode="w:gz") as tar:
        info = tarfile.TarInfo("garnet")
        info.mode = 0o755
        info.size = len(script)
        tar.addfile(info, io.BytesIO(script))
    return buf.getvalue()


@unittest.skipUnless(GPG, "gpg is needed to make the test release keys")
class InstallerSignatureTests(unittest.TestCase):
    def setUp(self) -> None:
        self._tmp = tempfile.TemporaryDirectory()
        self.tmp = Path(self._tmp.name)
        self.release = self.tmp / "release"
        self.release.mkdir()
        data = stand_in_tarball()
        lines = []
        for triple in TRIPLES:
            name = f"garnet-{VERSION}-{triple}.tar.gz"
            (self.release / name).write_bytes(data)
            lines.append(f"{hashlib.sha256(data).hexdigest()}  {name}\n")
        self.sums = self.release / "SHA256SUMS"
        self.sums.write_text("".join(lines))
        self.signer_home = self.tmp / "signer"
        self.signer = make_key(self.signer_home, "release")
        self.other_home = self.tmp / "other"
        self.other = make_key(self.other_home, "impostor")
        self.keys = self.tmp / "keys.asc"
        self.keys.write_bytes(gpg(self.signer_home, "--armor", "--export", self.signer).stdout
                              + gpg(self.other_home, "--armor", "--export", self.other).stdout)

    def tearDown(self) -> None:
        self._tmp.cleanup()

    def sign(self, home: Path, fpr: str) -> None:
        gpg(home, "--local-user", fpr, "--armor", "--output", str(self.sums) + ".asc",
            "--detach-sign", str(self.sums))

    def run_installer(self, *, path: str | None = None, **overrides: str) -> subprocess.CompletedProcess[str]:
        prefix = self.tmp / "prefix"
        env = {
            "PATH": path if path is not None else os.environ["PATH"],
            "HOME": str(self.tmp / "home"),
            "GARNET_VERSION": VERSION,
            "GARNET_BASE_URL": f"file://{self.release}",
            "GARNET_FORMAT": "tar",
            "GARNET_PREFIX": str(prefix),
            "GARNET_SIGNING_KEYS_URL": f"file://{self.keys}",
            "GARNET_SIGNING_KEY_FPR": self.signer,
            # A fallback would clone this and fail at once, never reach the network.
            "GARNET_SOURCE_REPO_URL": "file:///nonexistent/garnet.git",
        }
        env.update(overrides)
        (self.tmp / "home").mkdir(exist_ok=True)
        return subprocess.run(["sh", str(INSTALLER)], env=env, capture_output=True, text=True, timeout=120)

    def assert_refused(self, result: subprocess.CompletedProcess[str], reason: str) -> None:
        output = result.stdout + result.stderr
        self.assertNotEqual(0, result.returncode, output)
        self.assertIn(reason, output)
        self.assertNotIn("falling back to source install", output)
        self.assertFalse((self.tmp / "prefix" / "bin" / "garnet").exists(), output)

    def test_a_signature_by_the_pinned_key_installs(self) -> None:
        self.sign(self.signer_home, self.signer)
        result = self.run_installer()
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)
        self.assertIn(f"SHA256SUMS signature verified (key {self.signer})", result.stdout)

    def test_a_tampered_signature_is_refused_without_falling_back(self) -> None:
        self.sign(self.signer_home, self.signer)
        with self.sums.open("a") as f:  # the signed SHA256SUMS no longer matches its .asc
            f.write("0" * 64 + "  extra.tar.gz\n")
        self.assert_refused(self.run_installer(GARNET_INSTALL_MODE="auto"), "SHA256SUMS.asc does not verify")

    def test_a_missing_signature_is_refused(self) -> None:
        self.assert_refused(self.run_installer(GARNET_INSTALL_MODE="auto"), "SHA256SUMS.asc is missing")

    def test_a_signature_by_another_key_is_refused(self) -> None:
        self.sign(self.other_home, self.other)
        self.assert_refused(self.run_installer(), "not signed by the pinned release key")

    def test_an_inline_signed_message_is_not_a_signature_over_sha256sums(self) -> None:
        # A real signature by the pinned key over some other text must not pass.
        other = self.tmp / "other.txt"
        other.write_text("signed, but not SHA256SUMS\n")
        gpg(self.signer_home, "--local-user", self.signer, "--clearsign",
            "--output", str(self.sums) + ".asc", str(other))
        self.assert_refused(self.run_installer(), "SHA256SUMS.asc does not verify")

    def test_without_gpg_it_warns_and_installs_on_the_checksum(self) -> None:
        self.sign(self.signer_home, self.signer)
        shim = self.tmp / "no-gpg-bin"
        shim.mkdir()
        for directory in os.environ["PATH"].split(os.pathsep):
            if not os.path.isdir(directory):
                continue
            for entry in os.listdir(directory):
                target = Path(directory) / entry
                if entry.startswith("gpg") or (shim / entry).exists():
                    continue
                if target.is_file() and os.access(target, os.X_OK):
                    (shim / entry).symlink_to(target)
        result = self.run_installer(path=str(shim))
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)
        self.assertIn("gpg not found: SHA256SUMS.asc is not verified", result.stderr)

    def test_an_opt_out_warns_and_installs_on_the_checksum(self) -> None:
        result = self.run_installer(GARNET_VERIFY_SIGNATURE="0")
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)
        self.assertIn("signature verification is off (GARNET_VERIFY_SIGNATURE=0)", result.stderr)

    def test_releases_before_0_8_1_were_never_signed(self) -> None:
        for triple in TRIPLES:
            src = self.release / f"garnet-{VERSION}-{triple}.tar.gz"
            src.rename(self.release / f"garnet-0.8.0-{triple}.tar.gz")
        self.sums.write_text(self.sums.read_text().replace(f"garnet-{VERSION}-", "garnet-0.8.0-"))
        result = self.run_installer(GARNET_VERSION="0.8.0", GARNET_SIGNING_KEY_FPR="")
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)
        self.assertIn("v0.8.0 predates signed releases", result.stderr)


class PinnedKeyTests(unittest.TestCase):
    def test_the_installer_pins_the_0_8_key_for_0_8_1_and_0_8_2(self) -> None:
        text = INSTALLER.read_text()
        self.assertRegex(text, r"0\.8\.1\|0\.8\.2\)\s*printf '%s' '" + RELEASE_KEY_0_8 + "'")

    def test_the_published_keyring_holds_the_0_8_key(self) -> None:
        self.assertTrue(KEYRING.is_file(), "docs/garnet-release-keys.asc is what the installer fetches")
        if not GPG:
            self.skipTest("gpg is needed to read the keyring")
        with tempfile.TemporaryDirectory() as home:
            os.chmod(home, 0o700)
            listing = subprocess.run(
                ["gpg", "--homedir", home, "--batch", "--with-colons", "--show-keys", str(KEYRING)],
                capture_output=True, text=True, check=True,
            ).stdout
        self.assertIn(f"fpr:::::::::{RELEASE_KEY_0_8}:", listing)


if __name__ == "__main__":
    unittest.main()
