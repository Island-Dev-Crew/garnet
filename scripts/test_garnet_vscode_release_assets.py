#!/usr/bin/env python3
"""Static contract tests for v0.5 VS Code release assets."""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACKAGE_SCRIPT = ROOT / "scripts" / "package_garnet_vscode_extension.sh"
SMOKE_SCRIPT = ROOT / "scripts" / "verify_org_release_smoke.sh"
WORKFLOW = ROOT / ".github" / "workflows" / "vscode-extension.yml"
COLLECT_SCRIPT = ROOT / "scripts" / "collect_garnet_vsix_release_assets.py"


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
            "softprops/action-gh-release@efb35369e0ad2afab669f228072c1b0d510eae64",
            text,
        )
        self.assertIn(
            'python3 scripts/collect_garnet_vsix_release_assets.py --tag "${GITHUB_REF_NAME}"',
            text,
        )
        self.assertIn("release-vsix/*.vsix", text)
        self.assertIn("fail_on_unmatched_files: true", text)

    def test_release_vsix_publishes_no_generated_notes(self) -> None:
        # C6-13: the release body comes from one curated file in linux-packages.yml;
        # a second publisher with generated notes would replace it.
        text = WORKFLOW.read_text(encoding="utf-8")
        self.assertNotIn("generate_release_notes", text)
        self.assertNotIn("body_path", text)

    def test_packaging_script_clears_its_stale_outputs_before_packaging(self) -> None:
        # C6-06: a restored target/ cache carried VSIX files from earlier versions
        # into target/vscode/*.vsix, which the release job then published.
        text = PACKAGE_SCRIPT.read_text(encoding="utf-8")
        clear_vsix = 'rm -f "${OUTPUT_DIR}"/garnet-*-lsp-mvp-*.vsix'
        clear_evidence = 'rm -rf "${OUTPUT_DIR}"/garnet-vscode-release-assets-*'
        self.assertIn(clear_vsix, text)
        self.assertIn(clear_evidence, text)
        self.assertLess(text.index(clear_vsix), text.index("npx vsce package --out"))
        self.assertLess(text.index(clear_evidence), text.index('mkdir -p "${EVIDENCE_DIR}"'))

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


class CollectVsixReleaseAssetsTests(unittest.TestCase):
    """C6-06: only this tag's two VSIX files reach the release."""

    def setUp(self) -> None:
        self._tmp = tempfile.TemporaryDirectory()
        self.tmp = Path(self._tmp.name)
        self.source = self.tmp / "dist-vsix"
        self.dest = self.tmp / "release-vsix"
        self.package_json = self.tmp / "package.json"
        self.package_json.write_text(json.dumps({"name": "garnet", "version": "0.8.3"}))

    def tearDown(self) -> None:
        self._tmp.cleanup()

    def vsix(self, name: str, subdir: str = "") -> None:
        folder = self.source / subdir
        folder.mkdir(parents=True, exist_ok=True)
        (folder / name).write_bytes(b"PK\x03\x04" + name.encode())

    def collect(self, tag: str = "v0.8.3", expect: str = "2") -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [sys.executable, str(COLLECT_SCRIPT), "--tag", tag, "--from", str(self.source),
             "--to", str(self.dest), "--package-json", str(self.package_json), "--expect", expect],
            capture_output=True, text=True,
        )

    def published(self) -> list[str]:
        return sorted(p.name for p in self.dest.iterdir()) if self.dest.is_dir() else []

    def test_copies_exactly_this_tags_two_files_and_skips_stale_ones(self) -> None:
        self.vsix("garnet-0.8.3-lsp-mvp-linux-x64.vsix", "a")
        self.vsix("garnet-0.8.3-lsp-mvp-darwin-arm64.vsix", "b")
        self.vsix("garnet-0.8.2-lsp-mvp-linux-x64.vsix", "a")
        self.vsix("garnet-0.8.30-lsp-mvp-linux-x64.vsix", "b")
        result = self.collect()
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)
        self.assertEqual(
            ["garnet-0.8.3-lsp-mvp-darwin-arm64.vsix", "garnet-0.8.3-lsp-mvp-linux-x64.vsix"],
            self.published(),
        )
        self.assertIn("garnet-0.8.2-lsp-mvp-linux-x64.vsix", result.stdout)

    def test_refuses_a_count_other_than_expected(self) -> None:
        self.vsix("garnet-0.8.3-lsp-mvp-linux-x64.vsix")
        result = self.collect()
        self.assertNotEqual(0, result.returncode)
        self.assertIn("expected 2", result.stderr)
        self.assertEqual([], self.published())

    def test_refuses_the_same_asset_twice(self) -> None:
        self.vsix("garnet-0.8.3-lsp-mvp-linux-x64.vsix", "a")
        self.vsix("garnet-0.8.3-lsp-mvp-linux-x64.vsix", "b")
        result = self.collect()
        self.assertNotEqual(0, result.returncode)
        self.assertEqual([], self.published())

    def test_refuses_a_package_version_that_is_not_the_tag(self) -> None:
        self.vsix("garnet-0.8.2-lsp-mvp-linux-x64.vsix")
        self.vsix("garnet-0.8.2-lsp-mvp-darwin-arm64.vsix")
        self.package_json.write_text(json.dumps({"version": "0.8.2"}))
        result = self.collect()
        self.assertNotEqual(0, result.returncode)
        self.assertIn("does not match release tag v0.8.3", result.stderr)
        self.assertEqual([], self.published())

    def test_refuses_a_tag_that_is_not_a_release_version(self) -> None:
        for tag in ("0.8.3", "v0.8", "v0.8.3-rc1", "main"):
            with self.subTest(tag=tag):
                self.assertNotEqual(0, self.collect(tag=tag).returncode)


if __name__ == "__main__":
    unittest.main()
