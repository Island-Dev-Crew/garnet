#!/usr/bin/env python3
"""Static contract for the Linux and macOS package jobs (T5b: C6-04, C6-16).

The Linux assets are built on Ubuntu 22.04 runners and must import no glibc
symbol newer than GLIBC_2.34, so they install on Debian 12, Ubuntu 22.04 and
RHEL 9; a container smoke on each proves it for both architectures before a
release publishes.
"""
from __future__ import annotations

import os
import re
import stat
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
WORKFLOW = ROOT / ".github" / "workflows" / "linux-packages.yml"
FLOOR_SCRIPT = ROOT / "scripts" / "check_glibc_floor.sh"


def job(name: str) -> str:
    text = WORKFLOW.read_text(encoding="utf-8")
    start = text.index(f"\n  {name}:\n")
    following = re.search(r"\n  [a-z0-9-]+:\n", text[start + 1:])
    end = start + 1 + following.start() if following else len(text)
    return text[start:end]


class LinuxBuildBaseTests(unittest.TestCase):
    def test_both_linux_builds_run_on_ubuntu_22_04(self) -> None:
        self.assertIn("\n    runs-on: ubuntu-22.04\n", job("build-packages"))
        self.assertIn("\n    runs-on: ubuntu-22.04-arm\n", job("build-packages-arm64"))

    def test_the_build_caches_are_keyed_to_the_build_image(self) -> None:
        # A cache restored from a 24.04 build would reuse objects compiled
        # against glibc 2.39 headers and raise the floor again.
        for name in ("build-packages", "build-packages-arm64"):
            with self.subTest(job=name):
                keys = re.findall(r"(?:key|restore-keys): (.*)", job(name))
                self.assertTrue(keys)
                for key in keys:
                    self.assertIn("ubuntu-22.04", key)

    def test_both_builds_check_the_glibc_floor_before_packaging(self) -> None:
        for name in ("build-packages", "build-packages-arm64"):
            with self.subTest(job=name):
                text = job(name)
                self.assertIn("scripts/check_glibc_floor.sh target/release/garnet 2.34", text)
                self.assertLess(
                    text.index("check_glibc_floor.sh"), text.index("- name: Build .deb")
                )

    def test_no_linux_job_floats_on_ubuntu_latest(self) -> None:
        self.assertNotIn("ubuntu-latest", WORKFLOW.read_text(encoding="utf-8"))


class OlderDistroSmokeTests(unittest.TestCase):
    def test_the_smoke_covers_each_older_distro_on_both_architectures(self) -> None:
        text = job("smoke-older-distros")
        for image in ("debian:12", "ubuntu:22.04", "almalinux:9"):
            for arch, runner, artifact in (
                ("x86_64", "ubuntu-24.04", "garnet-linux-packages"),
                ("arm64", "ubuntu-24.04-arm", "garnet-linux-packages-arm64"),
            ):
                with self.subTest(image=image, arch=arch):
                    self.assertRegex(
                        text,
                        rf"- image: {re.escape(image)}\n\s+arch: {arch}\n\s+runner: {re.escape(runner)}\n"
                        rf"\s+artifact: {re.escape(artifact)}\n",
                    )
        self.assertIn("container: ${{ matrix.image }}", text)
        self.assertIn("garnet check /tmp/bad.garnet", text)

    def test_a_release_waits_for_the_older_distro_smoke(self) -> None:
        self.assertRegex(job("release"), r"\n      - smoke-older-distros\n")


class MacosInstallerTests(unittest.TestCase):
    def test_the_macos_job_runs_install_sh_against_its_tarball(self) -> None:
        text = job("macos-cli-tarballs")
        self.assertIn("GARNET_FORMAT=tar", text)
        self.assertIn("sh docs/install.sh", text)
        self.assertIn('"${prefix}/bin/garnet" --version', text)


class GlibcFloorScriptTests(unittest.TestCase):
    """check_glibc_floor.sh reads `objdump -T`; a stub objdump feeds it."""

    def run_floor(self, versions: list[str], floor: str = "2.34") -> subprocess.CompletedProcess[str]:
        with tempfile.TemporaryDirectory() as tmp:
            stub = Path(tmp) / "objdump"
            lines = "".join(
                f"0000000000000000      DF *UND*  0000000000000000 ({v}) sym{i}\\n"
                for i, v in enumerate(versions)
            )
            stub.write_text(f"#!/bin/sh\nprintf '{lines}'\n")
            stub.chmod(stub.stat().st_mode | stat.S_IEXEC)
            binary = Path(tmp) / "garnet"
            binary.write_bytes(b"\x7fELF")
            env = {**os.environ, "PATH": f"{tmp}{os.pathsep}{os.environ['PATH']}"}
            return subprocess.run(
                ["sh", str(FLOOR_SCRIPT), str(binary), floor],
                capture_output=True, text=True, env=env,
            )

    def test_passes_at_or_below_the_floor(self) -> None:
        result = self.run_floor(["GLIBC_2.2.5", "GLIBC_2.17", "GLIBC_2.34", "GLIBC_2.9"])
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)
        self.assertIn("GLIBC_2.34", result.stdout)

    def test_fails_above_the_floor(self) -> None:
        result = self.run_floor(["GLIBC_2.2.5", "GLIBC_2.35", "GLIBC_2.39"])
        self.assertNotEqual(0, result.returncode)
        self.assertIn("GLIBC_2.39", result.stdout + result.stderr)

    def test_fails_when_it_finds_no_glibc_symbols(self) -> None:
        self.assertNotEqual(0, self.run_floor([]).returncode)


if __name__ == "__main__":
    unittest.main()
