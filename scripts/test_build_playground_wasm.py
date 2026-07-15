#!/usr/bin/env python3
import importlib.util, tempfile, unittest
from pathlib import Path
from unittest import mock
ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("build_playground_wasm", ROOT / "scripts/build_playground_wasm.py")
assert SPEC is not None and SPEC.loader is not None
builder = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(builder)
class HermeticPlaygroundBuilderTests(unittest.TestCase):
    def test_offline_normalized_environment_and_explicit_probe(self) -> None:
        for token in ("no-install", "--locked", "--offline"):
            self.assertIn(token, set(builder.WASM_PACK_ARGS))
        ambient = {"PATH": "kept", "RUSTFLAGS": "bad", "CARGO_HOME": "bad",
                   "WASM_PACK_CACHE": "bad", "NODE_OPTIONS": "bad", "ESBUILD_BINARY_PATH": "bad"}
        env = builder.build_env(ROOT, ROOT / "target/x", ambient)
        self.assertEqual({"PATH"}, set(ambient) & set(env))
        self.assertTrue(builder.DECLARED_BUILD_ENV.issubset(env))
        with mock.patch("sys.stderr"):
            with self.assertRaises(SystemExit): builder.parse_args([])
        self.assertTrue(builder.parse_args(["--probe"]).probe)
    def test_snapshot_bracket_and_canonical_json_fail_closed(self) -> None:
        base = {"head": "a", "inputs": ("x",), "locks": "l", "source": "s", "tools": "t"}
        for key in base:
            changed, finished = dict(base), []
            changed[key] = "changed"
            states = iter((base, changed))
            with self.assertRaises(builder.BuildError):
                builder.bracket(lambda: next(states), lambda _: b"built", lambda *_: finished.append(True))
            self.assertFalse(finished)
        self.assertEqual(b'{"a":{"b":3,"y":2},"z":1}\n', builder.canonical_json({"z": 1, "a": {"y": 2, "b": 3}}))
        with self.assertRaises(builder.BuildError): builder.decode_json(b'{"schema":1,"schema":2}')
    def test_esbuild_entry_and_platform_escape_fail_before_use(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root, raw = Path(td) / "repo", Path(td) / "repo/raw"; root.mkdir(); target = builder.secure_chain(root, root / "target", create=True)
            raw.mkdir(); (raw / "expected").write_bytes(b"x"); (raw / "extra").write_bytes(b"x")
            self.assertRaises(builder.BuildError, builder.require_exact_files, raw, {"expected"})
            self.assertRaises(builder.BuildError, builder.secure_chain, root, root.parent / "escape")
            binary = target / "esbuild"; binary.write_bytes(b"generator")
            with mock.patch.object(builder, "is_reparse", side_effect=lambda path: path == binary):
                self.assertRaises(builder.BuildError, builder.contained_binary_identity, root, binary)
            lock = root / "apps/garnet-studio/package-lock.json"; lock.parent.mkdir(parents=True); lock.write_bytes(b'{"packages":{"node_modules/esbuild":{"version":"0.25.12"}}}')
            esbuild = root / builder.ESBUILD_REL; esbuild.parent.mkdir(parents=True); esbuild.write_bytes(b"generator"); platform = root / "apps/garnet-studio/node_modules/@esbuild"; platform.mkdir()
            outside = root.parent / "outside/esbuild"; outside.parent.mkdir(); outside.write_bytes(b"generator")
            versions = tuple(builder.VERSIONS[name] for name in ("rustc", "cargo", "node", "wasm_pack"))
            for relative, reparse in ((builder.ESBUILD_REL, esbuild.parent), (Path("../outside/esbuild"), None), (builder.ESBUILD_REL, platform)):
                with self.subTest(relative=relative), mock.patch.object(builder, "ESBUILD_REL", relative), \
                        mock.patch.object(builder, "is_reparse", side_effect=lambda path, bad=reparse: path == bad), mock.patch.object(builder, "rustup_tool_identity", return_value={}), \
                        mock.patch.object(builder, "_binary_identity", return_value="id"), mock.patch.object(builder, "_run", side_effect=versions) as run:
                    self.assertRaises(builder.BuildError, builder.toolchain, root, {"PATH": "fixed"})
                self.assertEqual(4, run.call_count, "esbuild executed before containment validation")
    def test_wrapper_contract_is_exact(self) -> None:
        exports = {"check_source", "default", "diff_caps_source", "initSync", "run_source"}
        valid = 'const u=new URL("garnet_wasm_bg.wasm",import.meta.url);export{u};'
        builder.validate_wrapper_contract(ROOT, valid.encode(), exports)
        for text, observed in (("names only", exports), (valid, exports - {"default"}),
                               (valid.replace("garnet_wasm_bg.wasm", "https://evil/x.wasm"), exports)):
            with self.assertRaises(builder.BuildError): builder.validate_wrapper_contract(ROOT, text.encode(), observed)
    def test_rustup_proxy_is_separate_from_distinct_generators(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td); proxy, rustc, cargo = (root / name for name in ("rustup", "rustc-real", "cargo-real"))
            for path, raw in ((proxy, b"proxy"), (rustc, b"rustc"), (cargo, b"cargo")): path.write_bytes(raw)
            with mock.patch.object(builder.shutil, "which", return_value=str(proxy)), \
                    mock.patch.object(builder, "_run", side_effect=(str(rustc), str(cargo))):
                rustc_id = builder.rustup_tool_identity("rustc", root, {"PATH": str(root)})
                cargo_id = builder.rustup_tool_identity("cargo", root, {"PATH": str(root)})
            self.assertEqual(rustc_id["launcher_sha256"], cargo_id["launcher_sha256"])
            self.assertNotEqual(rustc_id["binary_sha256"], cargo_id["binary_sha256"])
    def test_source_fence_and_operator_check_are_exact(self) -> None:
        expected = {*builder.INPUT_ROOTS, "Cargo.lock", "Cargo.toml", ".cargo", ".cargo/config.toml"}
        self.assertEqual(expected, set(builder.SOURCE_WATCH_PATHS)); builder.require_no_untracked("")
        with self.assertRaises(builder.BuildError): builder.require_no_untracked("garnet-wasm/src/injected.rs\n")
        checks = (ROOT / "garnet-wasm/AGENTS.md").read_text(encoding="utf-8").split("## Required Checks", 1)[1]
        self.assertNotIn("wasm-pack build", checks); self.assertIn("build_playground_wasm.py --probe", checks)
if __name__ == "__main__": unittest.main()
