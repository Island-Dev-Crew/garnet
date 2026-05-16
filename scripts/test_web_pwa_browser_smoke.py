#!/usr/bin/env python3
"""Regression checks for the dependency-free browser PWA smoke."""
from __future__ import annotations

from pathlib import Path
import unittest

SCRIPT = Path(__file__).with_name("smoke_garnet_web_pwa_browser.mjs")


class WebPwaBrowserSmokeTests(unittest.TestCase):
    def test_browser_smoke_uses_chrome_devtools_without_playwright_dependency(self) -> None:
        script = SCRIPT.read_text(encoding="utf-8")

        self.assertIn("remote-debugging-port", script)
        self.assertIn("new WebSocket", script)
        self.assertIn("Network.emulateNetworkConditions", script)
        self.assertNotIn("playwright", script.lower())

    def test_browser_smoke_records_service_worker_offline_evidence(self) -> None:
        script = SCRIPT.read_text(encoding="utf-8")

        self.assertIn("navigator.serviceWorker.ready", script)
        self.assertIn("caches.keys()", script)
        self.assertIn("offlineNavigation", script)
        self.assertIn("manifestFetch", script)

    def test_browser_smoke_retries_profile_cleanup_after_chrome_exit(self) -> None:
        script = SCRIPT.read_text(encoding="utf-8")

        self.assertIn("async function waitForProcessExit", script)
        self.assertIn("async function removeBrowserProfile", script)
        self.assertIn("maxRetries", script)
        self.assertIn("await removeBrowserProfile(userDataDir)", script)


if __name__ == "__main__":
    unittest.main()
