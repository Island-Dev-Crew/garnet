#!/usr/bin/env python3
"""Regression tests for Garnet Studio packaged readiness resources."""
from __future__ import annotations

import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACKAGE_SCRIPT = ROOT / "scripts" / "package_garnet_studio_macos.sh"
DMG_SMOKE_SCRIPT = ROOT / "scripts" / "smoke_garnet_studio_dmg.sh"


class GarnetStudioPackagingResourceTests(unittest.TestCase):
    def test_package_script_stages_agentic_matrix_dependencies(self) -> None:
        script = PACKAGE_SCRIPT.read_text(encoding="utf-8")

        for name in [
            "garnet_adoption_surface_status.py",
            "garnet_assist_context_pack.py",
            "garnet_converter_assist_plan.py",
            "garnet_converter_status.py",
            "garnet_mit_readiness_status.py",
            "garnet_promo_video_status.py",
            "garnet_readiness_status.py",
            "garnet_studio_notarization_status.py",
            "export_garnet_promo_video_site.mjs",
            "qa_garnet_promo_video.mjs",
            "render_garnet_promo_video.mjs",
            "run_agentic_dogfood_matrix.py",
        ]:
            with self.subTest(name=name):
                self.assertIn(f'cp "${{ROOT}}/scripts/{name}"', script)
                self.assertIn(f'chmod 0755 "${{APP_DIR}}/Contents/Resources/scripts/{name}"', script)

        self.assertIn(
            'cp "${ROOT}/scripts/smoke_garnet_web_pwa_offline.mjs" '
            '"${APP_DIR}/Contents/Resources/scripts/smoke_garnet_web_pwa_offline.mjs"',
            script,
        )
        self.assertIn('cp -R "${ROOT}/docs" "${APP_DIR}/Contents/Resources/docs"', script)
        self.assertIn('mkdir -p "${APP_DIR}/Contents/Resources/assets"', script)
        self.assertIn(
            'cp "${ROOT}/assets/garnet-logo.png" '
            '"${APP_DIR}/Contents/Resources/assets/garnet-logo.png"',
            script,
        )
        self.assertIn(
            'mkdir -p "${APP_DIR}/Contents/Resources/apps/garnet-studio-macos/Sources/GarnetStudio"',
            script,
        )
        self.assertIn(
            'cp "${ROOT}/apps/garnet-studio-macos/Sources/GarnetStudio/GarnetStudioApp.swift" '
            '"${APP_DIR}/Contents/Resources/apps/garnet-studio-macos/Sources/GarnetStudio/GarnetStudioApp.swift"',
            script,
        )
        self.assertIn(
            'mkdir -p "${APP_DIR}/Contents/Resources/apps/garnet-studio-macos/Sources/GarnetStudio/Resources"',
            script,
        )
        self.assertIn(
            'cp "${ROOT}/apps/garnet-studio-macos/Sources/GarnetStudio/Resources/garnet-logo.png" '
            '"${APP_DIR}/Contents/Resources/apps/garnet-studio-macos/Sources/GarnetStudio/Resources/garnet-logo.png"',
            script,
        )
        self.assertIn('cp "${ROOT}/README.md" "${APP_DIR}/Contents/Resources/README.md"', script)
        self.assertIn('cp "${ROOT}/CURRENT_STATE.md" "${APP_DIR}/Contents/Resources/CURRENT_STATE.md"', script)
        self.assertIn('cp -R "${ROOT}/C_Language_Specification" "${APP_DIR}/Contents/Resources/C_Language_Specification"', script)
        self.assertIn('mkdir -p "${APP_DIR}/Contents/Resources/F_Project_Management/DOGFOOD"', script)
        self.assertIn(
            'cp "${ROOT}/F_Project_Management/GARNET_LANGUAGE_COMPLETION_IMPLEMENTATION_PLAN.md" '
            '"${APP_DIR}/Contents/Resources/F_Project_Management/GARNET_LANGUAGE_COMPLETION_IMPLEMENTATION_PLAN.md"',
            script,
        )
        self.assertIn(
            'cp "${ROOT}/F_Project_Management/DOGFOOD/GARNET_v0_5_DOGFOOD_READINESS_PHASE_LOG.md" '
            '"${APP_DIR}/Contents/Resources/F_Project_Management/DOGFOOD/GARNET_v0_5_DOGFOOD_READINESS_PHASE_LOG.md"',
            script,
        )
        self.assertIn('chmod 0755 "${APP_DIR}/Contents/Resources/scripts/smoke_garnet_web_pwa_offline.mjs"', script)

    def test_package_script_preserves_dmg_smoke_desktop_evidence(self) -> None:
        script = PACKAGE_SCRIPT.read_text(encoding="utf-8")

        self.assertIn('"${ROOT}/scripts/smoke_garnet_studio_dmg.sh" --copy-to-desktop "${DMG_PATH}"', script)

    def test_package_script_uses_developer_id_hardened_runtime_when_configured(self) -> None:
        script = PACKAGE_SCRIPT.read_text(encoding="utf-8")

        self.assertIn('APPLE_DEV_ID_APP="${APPLE_DEV_ID_APP:-}"', script)
        self.assertIn('codesign --force --deep --options runtime --timestamp --sign "${APPLE_DEV_ID_APP}" "${APP_DIR}"', script)
        self.assertIn('codesign --force --deep --sign - "${APP_DIR}"', script)

    def test_dmg_smoke_requires_packaged_pwa_assets(self) -> None:
        script = DMG_SMOKE_SCRIPT.read_text(encoding="utf-8")

        self.assertIn('OFFLINE_PWA_SMOKE="${INSTALLED_APP}/Contents/Resources/scripts/smoke_garnet_web_pwa_offline.mjs"', script)
        self.assertIn('ADOPTION_STATUS="${INSTALLED_APP}/Contents/Resources/scripts/garnet_adoption_surface_status.py"', script)
        self.assertIn('ASSIST_CONTEXT="${INSTALLED_APP}/Contents/Resources/scripts/garnet_assist_context_pack.py"', script)
        self.assertIn('ASSIST_PLAN="${INSTALLED_APP}/Contents/Resources/scripts/garnet_converter_assist_plan.py"', script)
        self.assertIn('MIT_STATUS="${INSTALLED_APP}/Contents/Resources/scripts/garnet_mit_readiness_status.py"', script)
        self.assertIn('PROMO_STATUS="${INSTALLED_APP}/Contents/Resources/scripts/garnet_promo_video_status.py"', script)
        self.assertIn('PROMO_EXPORT="${INSTALLED_APP}/Contents/Resources/scripts/export_garnet_promo_video_site.mjs"', script)
        self.assertIn('PROMO_QA="${INSTALLED_APP}/Contents/Resources/scripts/qa_garnet_promo_video.mjs"', script)
        self.assertIn('PROMO_RENDER="${INSTALLED_APP}/Contents/Resources/scripts/render_garnet_promo_video.mjs"', script)
        self.assertIn('NOTARIZATION_STATUS="${INSTALLED_APP}/Contents/Resources/scripts/garnet_studio_notarization_status.py"', script)
        self.assertIn('PROMO_ASSETS_DIR="${INSTALLED_APP}/Contents/Resources/assets"', script)
        self.assertIn('PROMO_STUDIO_SOURCE="${INSTALLED_APP}/Contents/Resources/apps/garnet-studio-macos/Sources/GarnetStudio/GarnetStudioApp.swift"', script)
        self.assertIn('PROMO_STUDIO_LOGO="${INSTALLED_APP}/Contents/Resources/apps/garnet-studio-macos/Sources/GarnetStudio/Resources/garnet-logo.png"', script)
        self.assertIn('DOCS_DIR="${INSTALLED_APP}/Contents/Resources/docs"', script)
        self.assertIn('PROMO_DESIGN="${DOCS_DIR}/promo/DESIGN.md"', script)
        self.assertIn('PROMO_COMPOSITION="${DOCS_DIR}/promo/composition.html"', script)
        self.assertIn('"${OFFLINE_PWA_SMOKE}" --docs-dir "${DOCS_DIR}"', script)

    def test_dmg_smoke_writes_manifest_verified_evidence_bundle(self) -> None:
        script = DMG_SMOKE_SCRIPT.read_text(encoding="utf-8")

        self.assertIn('OUTPUT_DIR="${ROOT}/target/macos/garnet-studio-dmg-smoke-${STAMP}"', script)
        self.assertIn('--copy-to-desktop', script)
        self.assertIn('dmg-smoke-report.md', script)
        self.assertIn('dmg-smoke-data.env', script)
        self.assertIn('packaged-pwa-offline-handler.json', script)
        self.assertIn('MANIFEST.sha256', script)
        self.assertIn('shasum -a 256 -c MANIFEST.sha256 >MANIFEST.verify.log', script)


if __name__ == "__main__":
    unittest.main()
