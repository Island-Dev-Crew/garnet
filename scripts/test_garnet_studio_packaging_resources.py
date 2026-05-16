#!/usr/bin/env python3
"""Regression tests for Garnet Studio packaged readiness resources."""
from __future__ import annotations

import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACKAGE_SCRIPT = ROOT / "scripts" / "package_garnet_studio_macos.sh"
DMG_SMOKE_SCRIPT = ROOT / "scripts" / "smoke_garnet_studio_dmg.sh"


class GarnetStudioPackagingResourceTests(unittest.TestCase):
    def test_package_script_stages_pwa_assets_for_bundled_matrix(self) -> None:
        script = PACKAGE_SCRIPT.read_text(encoding="utf-8")

        self.assertIn(
            'cp "${ROOT}/scripts/smoke_garnet_web_pwa_offline.mjs" '
            '"${APP_DIR}/Contents/Resources/scripts/smoke_garnet_web_pwa_offline.mjs"',
            script,
        )
        self.assertIn('cp -R "${ROOT}/docs" "${APP_DIR}/Contents/Resources/docs"', script)
        self.assertIn('chmod 0755 "${APP_DIR}/Contents/Resources/scripts/smoke_garnet_web_pwa_offline.mjs"', script)

    def test_dmg_smoke_requires_packaged_pwa_assets(self) -> None:
        script = DMG_SMOKE_SCRIPT.read_text(encoding="utf-8")

        self.assertIn('OFFLINE_PWA_SMOKE="${INSTALLED_APP}/Contents/Resources/scripts/smoke_garnet_web_pwa_offline.mjs"', script)
        self.assertIn('DOCS_DIR="${INSTALLED_APP}/Contents/Resources/docs"', script)
        self.assertIn('"${OFFLINE_PWA_SMOKE}" --docs-dir "${DOCS_DIR}"', script)


if __name__ == "__main__":
    unittest.main()
