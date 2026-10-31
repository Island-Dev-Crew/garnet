#!/usr/bin/env python3
"""install.sh judges the host glibc against the requested release's floor.

Releases through v0.8.2 were built on Ubuntu 24.04 and need glibc 2.39. From
v0.8.3 the Linux assets are built on Ubuntu 22.04 under a GLIBC_2.34 floor
(scripts/garnet_check_glibc_floor.sh), so Debian 12, Ubuntu 22.04 and RHEL 9 get the
release assets. Stub `uname` and `getconf` make the real installer see a Linux
host with a chosen glibc; the release directory is empty, so every case stops
after the glibc decision.
"""
from __future__ import annotations

import os
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
INSTALLER = ROOT / "docs" / "install.sh"
MIRROR = ROOT / "installer" / "sh.garnet-lang.org" / "install.sh"


class InstallerGlibcFloorTests(unittest.TestCase):
    def run_on(self, glibc: str, version: str) -> str:
        with tempfile.TemporaryDirectory() as tmp:
            bin_dir = Path(tmp) / "bin"
            bin_dir.mkdir()
            (bin_dir / "uname").write_text(
                '#!/bin/sh\ncase "$1" in -m) echo x86_64 ;; *) echo Linux ;; esac\n'
            )
            (bin_dir / "getconf").write_text(
                f'#!/bin/sh\n[ "$1" = GNU_LIBC_VERSION ] && echo "glibc {glibc}"\n'
            )
            for stub in bin_dir.iterdir():
                stub.chmod(0o755)
            release = Path(tmp) / "release"
            release.mkdir()
            env = {
                "PATH": f"{bin_dir}{os.pathsep}{os.environ['PATH']}",
                "HOME": tmp,
                "GARNET_VERSION": version,
                "GARNET_BASE_URL": f"file://{release}",
                "GARNET_FORMAT": "tar",
                "GARNET_INSTALL_MODE": "release",
                "GARNET_PREFIX": str(Path(tmp) / "prefix"),
                "GARNET_VERIFY_SIGNATURE": "0",
            }
            result = subprocess.run(["sh", str(INSTALLER)], env=env, capture_output=True,
                                    text=True, timeout=60)
            self.assertNotEqual(0, result.returncode, "the empty release directory cannot install")
            return result.stdout + result.stderr

    def test_releases_through_0_8_2_still_need_glibc_2_39(self) -> None:
        out = self.run_on("2.36", "0.8.2")
        self.assertIn("host glibc 2.36 is older than 2.39", out)

    def test_0_8_3_assets_are_offered_on_debian_12(self) -> None:
        out = self.run_on("2.36", "0.8.3")
        self.assertNotIn("is older than", out)

    def test_0_8_3_assets_are_offered_on_rhel_9(self) -> None:
        self.assertNotIn("is older than", self.run_on("2.34", "0.8.3"))

    def test_0_8_3_assets_are_skipped_below_glibc_2_34(self) -> None:
        out = self.run_on("2.33", "0.8.3")
        self.assertIn("host glibc 2.33 is older than 2.34", out)

    def test_the_published_copy_is_identical(self) -> None:
        self.assertEqual(INSTALLER.read_bytes(), MIRROR.read_bytes())


if __name__ == "__main__":
    unittest.main()
