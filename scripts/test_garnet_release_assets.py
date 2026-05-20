#!/usr/bin/env python3
"""Static contract tests for v0.5 release installer assets."""

from __future__ import annotations

import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
INSTALLER = ROOT / "installer" / "sh.garnet-lang.org" / "install.sh"
WORKFLOW = ROOT / ".github" / "workflows" / "linux-packages.yml"


class GarnetReleaseAssetsTests(unittest.TestCase):
    def test_installer_mac_tarball_fallback_names_are_release_assets(self) -> None:
        text = INSTALLER.read_text(encoding="utf-8")

        self.assertIn("Darwin) printf 'pkg'", text)
        self.assertIn("garnet-%s-%s.tar.gz", text)
        self.assertIn("aarch64-apple-darwin", text)
        self.assertIn("x86_64-apple-darwin", text)
        self.assertIn("warn \"native $_format release asset unavailable; trying tarball release asset\"", text)

    def test_tag_workflow_builds_macos_cli_tarballs(self) -> None:
        text = WORKFLOW.read_text(encoding="utf-8")

        self.assertIn("macos-cli-tarballs:", text)
        self.assertIn("runs-on: macos-latest", text)
        self.assertIn("target: [aarch64-apple-darwin, x86_64-apple-darwin]", text)
        self.assertIn("cargo build --release --locked -p garnet-cli --target", text)
        self.assertIn("dist/garnet-${version}-${target}.tar.gz", text)
        self.assertIn("garnet-macos-cli-${{ matrix.target }}", text)

    def test_tag_release_publishes_unified_checksummed_assets(self) -> None:
        text = WORKFLOW.read_text(encoding="utf-8")

        self.assertIn("needs: [smoke-deb, smoke-rpm, macos-cli-tarballs, shellcheck-installer]", text)
        self.assertIn("pattern: garnet-macos-cli-*", text)
        self.assertIn("sha256sum *.deb *.rpm *.tar.gz > SHA256SUMS", text)
        self.assertIn("release-dist/*.deb", text)
        self.assertIn("release-dist/*.rpm", text)
        self.assertIn("release-dist/*.tar.gz", text)
        self.assertIn("release-dist/SHA256SUMS", text)


if __name__ == "__main__":
    unittest.main()
