#!/usr/bin/env python3
"""Static contract tests for v0.5 VS Code release assets."""

from __future__ import annotations

import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACKAGE_SCRIPT = ROOT / "scripts" / "package_garnet_vscode_extension.sh"
SMOKE_SCRIPT = ROOT / "scripts" / "verify_org_release_smoke.sh"
WORKFLOW = ROOT / ".github" / "workflows" / "vscode-extension.yml"


class GarnetVSCodeReleaseAssetsTests(unittest.TestCase):
    def test_packaging_script_labels_host_native_assets(self) -> None:
        text = PACKAGE_SCRIPT.read_text(encoding="utf-8")

        for target in ("darwin-arm64", "darwin-x64", "linux-x64", "linux-arm64", "win32-x64", "win32-arm64"):
            self.assertIn(target, text)
        self.assertIn("VSIX_NAME=\"garnet-${VERSION}-lsp-mvp-${TARGET}.vsix\"", text)
        self.assertIn("cargo build -p garnet-lsp --release --locked", text)
        self.assertIn("node scripts/bundle-server.mjs", text)
        self.assertIn("npx vsce package --out", text)
        self.assertIn("extension/server/{server_exe}", text)
        self.assertIn("Marketplace/OpenVSX publication", text)
        self.assertIn("MANIFEST.sha256", text)

    def test_ci_builds_and_publishes_vscode_release_assets_on_tags(self) -> None:
        text = WORKFLOW.read_text(encoding="utf-8")

        self.assertIn("tags: ['v*']", text)
        self.assertIn("./scripts/package_garnet_vscode_extension.sh --output-dir target/vscode", text)
        self.assertIn("target/vscode/*.vsix", text)
        self.assertIn("if: startsWith(github.ref, 'refs/tags/v')", text)
        self.assertIn(
            "softprops/action-gh-release@3bb12739c298aeb8a4eeaf626c5b8d85266b0e65",
            text,
        )
        self.assertIn("find dist-vsix -name '*.vsix'", text)
        self.assertIn("release-vsix/*.vsix", text)
        self.assertIn("fail_on_unmatched_files: true", text)

    def test_release_smoke_requires_release_backed_vsix_structure(self) -> None:
        text = SMOKE_SCRIPT.read_text(encoding="utf-8")

        self.assertIn("verifying release-backed VSIX asset", text)
        self.assertIn("GARNET_VSIX_ASSET", text)
        self.assertIn("garnet-${SEMVER}-lsp-mvp-${vsix_target}.vsix", text)
        self.assertIn("extension/package.json", text)
        self.assertIn("extension/dist/extension.js", text)
        self.assertIn("extension/server/garnet-lsp", text)
        self.assertIn("extension/server/garnet-lsp.exe", text)
        self.assertIn("GARNET_SKIP_VSIX_CHECK", text)


if __name__ == "__main__":
    unittest.main()
