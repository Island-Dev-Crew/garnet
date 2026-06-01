#!/usr/bin/env python3
"""Regression tests for the Windows release-tooling status reporter (S88)."""
from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_release_tooling_status.py")


def load_reporter():
    spec = importlib.util.spec_from_file_location("garnet_release_tooling_status", SCRIPT)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules["garnet_release_tooling_status"] = module
    spec.loader.exec_module(module)
    return module


class ReleaseToolingStatusTests(unittest.TestCase):
    def test_reporter_script_exists(self) -> None:
        self.assertTrue(SCRIPT.is_file(), "S88 reporter script must exist")

    def test_absent_tools_are_honest_and_gate_ok(self) -> None:
        if not SCRIPT.is_file():
            self.skipTest("reporter missing; existence test covers the red state")
        rt = load_reporter()
        status = rt.read_status(which=lambda _name: None, candidates=lambda _name: [])
        self.assertEqual(status.summary, "tools absent — not verified here")
        self.assertTrue(status.ok, "absence is pending-infra, not a fake failure")
        self.assertTrue(all(t.state == "absent" for t in status.tools))
        self.assertEqual(
            rt.main(
                ["--gate", "--format", "json"],
                which=lambda _name: None,
                candidates=lambda _name: [],
            ),
            0,
        )

    def test_present_tools_run_real_probe_commands(self) -> None:
        if not SCRIPT.is_file():
            self.skipTest("reporter missing; existence test covers the red state")
        rt = load_reporter()
        calls: list[tuple[str, ...]] = []

        def fake_which(name: str) -> str | None:
            return f"C:/tools/{name}.exe"

        def fake_run(argv: list[str], **_kwargs):
            calls.append(tuple(argv))
            return rt.ProbeResult(returncode=0, stdout="ok", stderr="")

        status = rt.read_status(which=fake_which, run=fake_run)
        self.assertTrue(status.ok)
        self.assertEqual({t.state for t in status.tools}, {"verified"})
        self.assertTrue(any(call[0] == "cosign" and call[1] == "sign-blob" for call in calls))
        self.assertTrue(any(call[0] == "cosign" and call[1] == "verify-blob" for call in calls))
        self.assertTrue(any(call[:2] == ("syft", "scan") for call in calls))
        self.assertTrue(any(call[:2] == ("cyclonedx", "validate") for call in calls))
        self.assertTrue(any(call[0] == "wasmtime" and "fuel=1000" in call for call in calls))
        self.assertTrue(any(call[0] == "wasmtime" and "epoch-interruption=y" in call for call in calls))

    def test_winget_paths_are_used_when_path_is_not_refreshed(self) -> None:
        if not SCRIPT.is_file():
            self.skipTest("reporter missing; existence test covers the red state")
        rt = load_reporter()
        calls: list[tuple[str, ...]] = []
        candidates = {
            "cosign": ["C:/winget/cosign-windows-amd64.exe"],
            "syft": ["C:/winget/syft.exe"],
            "cyclonedx": ["C:/winget/cyclonedx.exe"],
            "wasmtime": ["C:/Program Files/Wasmtime/bin/wasmtime.exe"],
        }

        def fake_run(argv: list[str], **_kwargs):
            calls.append(tuple(argv))
            return rt.ProbeResult(returncode=0, stdout="ok", stderr="")

        status = rt.read_status(
            which=lambda _name: None,
            run=fake_run,
            candidates=lambda name: candidates[name],
        )
        self.assertEqual({t.state for t in status.tools}, {"verified"})
        self.assertTrue(
            any(
                call[0] == "C:/winget/cosign-windows-amd64.exe"
                and call[1] == "verify-blob"
                for call in calls
            )
        )
        self.assertTrue(any(call[:2] == ("C:/winget/syft.exe", "scan") for call in calls))
        self.assertTrue(
            any(call[:2] == ("C:/winget/cyclonedx.exe", "validate") for call in calls)
        )
        self.assertTrue(
            any(call[0] == "C:/Program Files/Wasmtime/bin/wasmtime.exe" for call in calls)
        )

    def test_present_tool_failure_is_reported_not_stamped_verified(self) -> None:
        if not SCRIPT.is_file():
            self.skipTest("reporter missing; existence test covers the red state")
        rt = load_reporter()

        def fake_run(argv: list[str], **_kwargs):
            code = 2 if argv[0] == "wasmtime" else 0
            return rt.ProbeResult(returncode=code, stdout="", stderr="boom" if code else "")

        status = rt.read_status(which=lambda name: f"C:/tools/{name}.exe", run=fake_run)
        by_name = {t.name: t for t in status.tools}
        self.assertEqual(by_name["wasmtime"].state, "failed")
        self.assertIn("boom", by_name["wasmtime"].evidence)
        self.assertFalse(status.ok)

    def test_markdown_preserves_honest_scope(self) -> None:
        if not SCRIPT.is_file():
            self.skipTest("reporter missing; existence test covers the red state")
        rt = load_reporter()
        md = rt.render_markdown(
            rt.read_status(which=lambda _name: None, candidates=lambda _name: [])
        )
        self.assertIn("tools absent — not verified here", md)
        self.assertIn("never stamps signed/SBOM/fuel without the tool", md)

    def test_agent_contract_is_registered(self) -> None:
        if not SCRIPT.is_file():
            self.skipTest("reporter missing; existence test covers the red state")
        rt = load_reporter()
        self.assertEqual(
            rt.read_status(which=lambda _name: None, candidates=lambda _name: []).schema,
            "garnet.release_tooling_status/v1",
        )


if __name__ == "__main__":
    unittest.main()
