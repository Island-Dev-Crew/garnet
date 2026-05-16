#!/usr/bin/env python3
"""Regression tests for the live Pages PWA smoke script."""

from __future__ import annotations

import http.server
import shutil
import socketserver
import subprocess
import tempfile
import threading
import unittest
from functools import partial
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "smoke_garnet_pages_pwa.sh"
DOCS = ROOT / "docs"


class PagesHandler(http.server.SimpleHTTPRequestHandler):
    extensions_map = {
        **http.server.SimpleHTTPRequestHandler.extensions_map,
        ".webmanifest": "application/manifest+json",
    }

    def log_message(self, format: str, *args: object) -> None:
        return


class ThreadingHTTPServer(socketserver.ThreadingMixIn, http.server.HTTPServer):
    daemon_threads = True


class PagesPwaSmokeTests(unittest.TestCase):
    def test_missing_studio_adoption_copy_is_a_strict_blocker(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_path = Path(temp_dir)
            fixture = temp_path / "docs"
            shutil.copytree(DOCS, fixture)

            index_path = fixture / "index.html"
            index = index_path.read_text(encoding="utf-8")
            replacements = {
                "Garnet Studio workbench": "Studio section removed",
                "Codex Run": "local app action",
                "dist/Garnet Studio.app": "staged app bundle",
                "Assist Plan": "migration planner",
            }
            for phrase, replacement in replacements.items():
                index = index.replace(phrase, replacement)
            index_path.write_text(index, encoding="utf-8")

            handler = partial(PagesHandler, directory=str(fixture))
            with ThreadingHTTPServer(("127.0.0.1", 0), handler) as server:
                thread = threading.Thread(target=server.serve_forever, daemon=True)
                thread.start()
                base_url = f"http://127.0.0.1:{server.server_port}"
                output_dir = temp_path / "evidence"

                result = subprocess.run(
                    [
                        str(SCRIPT),
                        "--base-url",
                        base_url,
                        "--output-dir",
                        str(output_dir),
                        "--strict",
                    ],
                    cwd=ROOT,
                    text=True,
                    stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE,
                    check=False,
                )

                server.shutdown()

        self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn("Garnet Studio workbench", result.stderr)
        self.assertIn("Codex Run", result.stderr)
        self.assertIn("Assist Plan", result.stderr)


if __name__ == "__main__":
    unittest.main()
