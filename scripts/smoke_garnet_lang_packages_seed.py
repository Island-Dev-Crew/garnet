#!/usr/bin/env python3
"""Smoke the S18 local `@garnet-lang/*` registry seed.

This is filesystem-registry source proof. It does not create or publish the
external `github.com/garnet-lang/*` repositories.
"""
from __future__ import annotations

import shutil
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REGISTRY = ROOT / "examples" / "garnet_lang_registry_seed"
PROJECT = ROOT / "examples" / "mvp_18_all_official_packages"
PACKAGES = ["http-client", "llm", "cli", "test-property", "log"]


def run(cmd: list[str], *, cwd: Path = ROOT) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=cwd,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=True,
    )


def cargo_garnet(args: list[str], *, cwd: Path) -> subprocess.CompletedProcess[str]:
    return run(
        [
            "cargo",
            "run",
            "--quiet",
            "--manifest-path",
            str(ROOT / "Cargo.toml"),
            "-p",
            "garnet-cli",
            "--",
            *args,
        ],
        cwd=cwd,
    )


def main() -> int:
    for package in PACKAGES:
        cargo_garnet(["check", str(REGISTRY / package / "0.1.0" / "lib.garnet")], cwd=ROOT)
    cargo_garnet(["check", str(PROJECT / "src" / "main.garnet")], cwd=ROOT)

    run(["cargo", "run", "--quiet", "-p", "garnet-registry-stub", "--", "build", str(REGISTRY)])
    run(["cargo", "run", "--quiet", "-p", "garnet-registry-stub", "--", "verify", str(REGISTRY)])

    with tempfile.TemporaryDirectory(prefix="garnet-s18-packages-") as temp:
        project = Path(temp) / "project"
        shutil.copytree(PROJECT, project)
        for package in PACKAGES:
            cargo_garnet(
                ["add", "--registry", str(REGISTRY), f"{package}@0.1.0"],
                cwd=project,
            )
        result = cargo_garnet(["run", "src/main.garnet"], cwd=project)

    if "S18 packages local-registry seed: ok" not in result.stdout:
        raise AssertionError(
            "S18 package seed smoke did not produce the expected success marker\n"
            f"stdout:\n{result.stdout}\nstderr:\n{result.stderr}"
        )
    print(result.stdout.strip())
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
