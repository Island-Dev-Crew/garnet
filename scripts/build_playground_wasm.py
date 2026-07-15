#!/usr/bin/env python3
"""Hermetically build and validate Garnet's web-target Wasm package in temp."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shutil
import stat
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACKAGE_REL = Path("docs/playground/pkg")
ARTIFACTS = ("garnet_wasm.js", "garnet_wasm_bg.wasm")
PUBLISHED = (*ARTIFACTS, "provenance.json")
INPUT_ROOTS = (
    "garnet-wasm", "garnet-check-v0.3", "garnet-interp-v0.3", "garnet-memory-v0.3",
    "garnet-parser-v0.3", "garnet-prim-macros", "garnet-stdlib",
)
VERSIONS = {
    "rustc": "rustc 1.95.0 (59807616e 2026-04-14)",
    "cargo": "cargo 1.95.0 (f2d3ce0bd 2026-03-21)", "node": "v22.22.2",
    "wasm_pack": "wasm-pack 0.15.0", "esbuild": "0.25.12",
}
ESBUILD_REL = Path("apps/garnet-studio/node_modules/esbuild/bin/esbuild")
WASM_PACK_ARGS = (
    "build", "garnet-wasm", "--mode", "no-install", "--target", "web", "--release",
    "--no-typescript", "--no-pack", "--no-opt", "--out-name", "garnet_wasm",
    "--out-dir", "{out_dir}", "--", "--locked", "--offline",
)
DECLARED_BUILD_ENV = frozenset({
    "CARGO_TARGET_DIR", "CARGO_NET_OFFLINE", "CARGO_INCREMENTAL", "CARGO_ENCODED_RUSTFLAGS",
})
EXPECTED_EXPORTS = frozenset({"check_source", "default", "diff_caps_source", "initSync", "run_source"})


class BuildError(RuntimeError):
    """A package contract failed closed."""


def sha256(raw: bytes) -> str:
    return hashlib.sha256(raw).hexdigest()


def canonical_source(raw: bytes) -> bytes:
    return raw.replace(b"\r\n", b"\n")


def canonical_json(value: object) -> bytes:
    return (json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n").encode()


def decode_json(raw: bytes) -> dict:
    def unique(pairs):
        result = {}
        for key, value in pairs:
            if key in result:
                raise BuildError(f"duplicate JSON key: {key}")
            result[key] = value
        return result
    try:
        value = json.loads(raw.decode("utf-8"), object_pairs_hook=unique)
    except (UnicodeError, json.JSONDecodeError) as error:
        raise BuildError(f"invalid JSON: {error}") from error
    if not isinstance(value, dict):
        raise BuildError("JSON root must be an object")
    return value


def _run(argv: list[str], root: Path, env: dict[str, str] | None = None) -> str:
    try:
        result = subprocess.run(argv, cwd=root, env=env, stdout=subprocess.PIPE,
                                stderr=subprocess.STDOUT, text=True, encoding="utf-8",
                                errors="replace", check=False, shell=False)
    except FileNotFoundError as error:
        raise BuildError(f"required tool is missing: {argv[0]}") from error
    if result.returncode:
        raise BuildError(f"command failed ({result.returncode}): {argv[0]}\n{result.stdout}")
    return result.stdout.strip()


def _git(root: Path, *args: str) -> str:
    return _run(["git", *args], root)


def build_env(root: Path, target: Path, ambient: dict[str, str] | None = None) -> dict[str, str]:
    env = dict(os.environ if ambient is None else ambient)
    for key in list(env):
        if key.upper().startswith(("RUST", "CARGO_", "WASM_PACK", "NODE_OPTIONS", "ESBUILD_")):
            del env[key]
    env.update({"CARGO_TARGET_DIR": str(target), "CARGO_NET_OFFLINE": "true",
                "CARGO_INCREMENTAL": "0", "CARGO_ENCODED_RUSTFLAGS": f"--remap-path-prefix={root}=."})
    return env


def is_reparse(path: Path) -> bool:
    info = path.lstat()
    return stat.S_ISLNK(info.st_mode) or bool(getattr(info, "st_file_attributes", 0)
                                             & getattr(stat, "FILE_ATTRIBUTE_REPARSE_POINT", 0))


def secure_entry(path: Path, directory: bool) -> None:
    if not os.path.lexists(path) or is_reparse(path):
        raise BuildError(f"missing or reparse-point path: {path.name}")
    mode = path.lstat().st_mode
    if not (stat.S_ISDIR(mode) if directory else stat.S_ISREG(mode)):
        raise BuildError(f"non-regular path: {path.name}")


def secure_chain(root: Path, path: Path, create: bool = False) -> Path:
    secure_entry(root, directory=True)
    if not path.resolve(strict=False).is_relative_to(root.resolve(strict=True)):
        raise BuildError("build path escapes the repository")
    current = root
    for part in path.relative_to(root).parts:
        current /= part
        if not os.path.lexists(current) and create:
            current.mkdir()
        secure_entry(current, directory=True)
    return path


def require_exact_files(directory: Path, expected: set[str]) -> None:
    secure_entry(directory, directory=True)
    actual = {path.name for path in directory.iterdir()}
    if actual != expected:
        raise BuildError(f"unexpected outputs: expected {sorted(expected)}, got {sorted(actual)}")
    for name in actual:
        secure_entry(directory / name, directory=False)


def source_inputs(root: Path) -> list[str]:
    exact, selected = {"Cargo.lock", "Cargo.toml", ".cargo/config.toml"}, []
    for path in _git(root, "ls-files").splitlines():
        under = any(path.startswith(prefix + "/") for prefix in INPUT_ROOTS)
        if path in exact or under and (path.endswith((".rs", ".toml")) or path.endswith("AGENTS.md")):
            selected.append(path)
    if not all(path in selected for path in ("Cargo.lock", "Cargo.toml", "garnet-wasm/src/lib.rs")):
        raise BuildError("source-input inventory is incomplete")
    return sorted(selected)


def source_digest(root: Path, inputs: list[str]) -> str:
    digest = hashlib.sha256()
    for relative in inputs:
        digest.update(relative.encode() + b"\0" + canonical_source((root / relative).read_bytes()) + b"\0")
    return digest.hexdigest()


def _binary_identity(command: str, env: dict[str, str]) -> str:
    path_value = next((value for key, value in env.items() if key.upper() == "PATH"), None)
    found = shutil.which(command, path=path_value)
    if not found or not Path(found).resolve().is_file():
        raise BuildError(f"required tool is missing: {command}")
    return sha256(Path(found).resolve().read_bytes())


def toolchain(root: Path, env: dict[str, str]) -> tuple[dict, Path]:
    commands = {"rustc": "rustc", "cargo": "cargo", "node": "node", "wasm_pack": "wasm-pack"}
    tools = {}
    for name, command in commands.items():
        actual = _run([command, "--version"], root, env)
        if actual != VERSIONS[name]:
            raise BuildError(f"{name} version mismatch: {actual!r}")
        tools[name] = {"binary_sha256": _binary_identity(command, env),
                       "command": command, "version": actual}
    package_lock = decode_json((root / "apps/garnet-studio/package-lock.json").read_bytes())
    if package_lock.get("packages", {}).get("node_modules/esbuild", {}).get("version") != VERSIONS["esbuild"]:
        raise BuildError("locked esbuild version mismatch")
    esbuild = root / ESBUILD_REL
    secure_entry(esbuild, directory=False)
    if _run(["node", str(esbuild), "--version"], root, env) != VERSIONS["esbuild"]:
        raise BuildError("installed esbuild version mismatch")
    platform_bins = [path for path in (root / "apps/garnet-studio/node_modules/@esbuild").rglob("esbuild*")
                     if path.is_file()]
    if len(platform_bins) != 1:
        raise BuildError("esbuild platform-binary inventory is not exact")
    tools["esbuild"] = {"binary_sha256": sha256(esbuild.read_bytes()),
                        "command": ESBUILD_REL.as_posix(),
                        "platform_binary_sha256": sha256(platform_bins[0].read_bytes()),
                        "version": VERSIONS["esbuild"]}
    return tools, esbuild


def snapshot(root: Path) -> dict:
    status = _git(root, "status", "--porcelain", "--untracked-files=no")
    if status:
        raise BuildError("package build requires a clean tracked tree")
    untracked = _git(root, "ls-files", "--others", "--exclude-standard", "--", PACKAGE_REL.as_posix())
    if untracked:
        raise BuildError(f"unexpected package-local untracked path: {untracked.splitlines()[0]}")
    inputs = source_inputs(root)
    tools, _ = toolchain(root, build_env(root, root / "target/snapshot-only"))
    return {"build_parent_commit_observed": _git(root, "rev-parse", "HEAD"),
            "cargo_lock_sha256": sha256(canonical_source((root / "Cargo.lock").read_bytes())),
            "inputs": inputs, "source_tree_sha256": source_digest(root, inputs),
            "studio_package_lock_sha256": sha256(canonical_source(
                (root / "apps/garnet-studio/package-lock.json").read_bytes())),
            "tools": tools, "tracked_clean": True}


def validate_wrapper_contract(root: Path, raw: bytes, exports: set[str]) -> None:
    try:
        text = raw.decode("utf-8")
    except UnicodeError as error:
        raise BuildError("generated wrapper is not UTF-8") from error
    relative = re.findall(r'new URL\((["\'])garnet_wasm_bg\.wasm\1,import\.meta\.url\)', text)
    if exports != EXPECTED_EXPORTS or len(relative) != 1 or text.count("new URL(") != 1:
        raise BuildError("wrapper failed exact ESM export/relative-URL validation")
    leaks = (str(root), root.as_posix(), "file://", "sourceMappingURL")
    if any(marker in text for marker in leaks) or len(text.splitlines()) != 1:
        raise BuildError("wrapper contains a checkout/source-map path or is not one-line ESM")
    if re.search(r"(?<![A-Za-z0-9_])[A-Za-z]:[\\/]|/(?:Users|home|tmp)/", text):
        raise BuildError("wrapper contains an absolute filesystem path")


def _esm_exports(root: Path, wrapper: Path, env: dict[str, str]) -> set[str]:
    script = "import(process.argv[1]).then(m=>process.stdout.write(JSON.stringify(Object.keys(m).sort())))"
    output = _run(["node", "--input-type=module", "--eval", script, wrapper.resolve().as_uri()], root, env)
    try:
        exports = json.loads(output)
    except json.JSONDecodeError as error:
        raise BuildError("Node ESM import returned invalid export JSON") from error
    if not isinstance(exports, list) or not all(isinstance(item, str) for item in exports):
        raise BuildError("Node ESM import returned an invalid export inventory")
    return set(exports)


def build_package(root: Path, observed: dict) -> dict[str, bytes]:
    target = secure_chain(root, root / "target", create=True)
    with tempfile.TemporaryDirectory(prefix="playground-wasm-", dir=target) as td:
        temp, raw_dir, out_dir = Path(td), Path(td) / "raw", Path(td) / "out"
        secure_entry(temp, directory=True)
        env = build_env(root, temp / "cargo-target")
        tools, esbuild = toolchain(root, env)
        if tools != observed["tools"]:
            raise BuildError("tool identity changed before build")
        out_arg = os.path.relpath(raw_dir, root / "garnet-wasm")
        _run(["wasm-pack", *(arg.format(out_dir=out_arg) for arg in WASM_PACK_ARGS)], root, env)
        require_exact_files(raw_dir, {*ARTIFACTS, ".gitignore"})
        out_dir.mkdir()
        wrapper = out_dir / ARTIFACTS[0]
        _run(["node", str(esbuild), str(raw_dir / ARTIFACTS[0]), "--minify", "--format=esm",
              "--platform=browser", "--legal-comments=none", "--charset=utf8",
              f"--outfile={wrapper}", "--log-level=warning"], root, env)
        shutil.copyfile(raw_dir / ARTIFACTS[1], out_dir / ARTIFACTS[1])
        require_exact_files(out_dir, set(ARTIFACTS))
        artifacts = {name: (out_dir / name).read_bytes() for name in ARTIFACTS}
        validate_wrapper_contract(root, artifacts[ARTIFACTS[0]], _esm_exports(root, wrapper, env))
        if len(artifacts[ARTIFACTS[1]]) >= 3 * 1024 * 1024:
            raise BuildError("Wasm exceeds the conservative 3 MiB ceiling")
        source = {key: observed[key] for key in (
            "build_parent_commit_observed", "cargo_lock_sha256", "inputs",
            "source_tree_sha256", "studio_package_lock_sha256")}
        manifest = {"artifacts": {name: {"bytes": len(raw), "sha256": sha256(raw)}
                                  for name, raw in artifacts.items()},
                    "build": {"profile": "release", "target": "web", "wasm_opt": False},
                    "schema": "garnet.playground.wasm-package/1", "source": source, "tools": tools}
        return {**artifacts, "provenance.json": canonical_json(manifest)}


def bracket(read, build, finish):
    before, payload = (state := read()), build(state)
    if read() != before:
        raise BuildError("repository/tool snapshot changed during package build")
    return finish(payload, before)


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--probe", action="store_true", required=True,
                        help="build and validate in temp; publish nothing")
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    parse_args(argv)
    if Path.cwd().resolve() != ROOT.resolve():
        raise BuildError("run the builder from the repository root")
    payload = bracket(lambda: snapshot(ROOT), lambda state: build_package(ROOT, state), lambda data, _: data)
    manifest = decode_json(payload["provenance.json"])
    print(canonical_json({"artifacts": manifest["artifacts"], "build": manifest["build"],
                          "build_parent_commit_observed": manifest["source"]["build_parent_commit_observed"],
                          "schema": "garnet.playground.wasm-probe/1"}).decode(), end="")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except BuildError as error:
        raise SystemExit(f"error: {error}") from error
