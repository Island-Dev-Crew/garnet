#!/usr/bin/env python3
"""Contract tests for the hermetic W-PLAY package builder core."""
from __future__ import annotations

import importlib.util
import tempfile
import unittest
from pathlib import Path
from unittest import mock

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts/build_playground_wasm.py"
SPEC = importlib.util.spec_from_file_location("build_playground_wasm", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
builder = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(builder)


class HermeticPlaygroundBuilderTests(unittest.TestCase):
    def test_offline_pinned_argv_and_normalized_environment(self) -> None:
        args = set(builder.WASM_PACK_ARGS)
        for token in ("no-install", "--locked", "--offline"):
            self.assertIn(token, args)
        ambient = {"PATH": "kept", "RUSTFLAGS": "bad", "CARGO_HOME": "bad",
                   "WASM_PACK_CACHE": "bad", "NODE_OPTIONS": "bad", "ESBUILD_BINARY_PATH": "bad"}
        env = builder.build_env(ROOT, ROOT / "target/x", ambient)
        self.assertEqual("kept", env["PATH"])
        self.assertTrue(builder.DECLARED_BUILD_ENV.issubset(env))
        self.assertFalse(any(key in env for key in ambient if key != "PATH"))

    def test_snapshot_bracket_rejects_each_observed_change(self) -> None:
        base = {"head": "a", "inputs": ("x",), "locks": "l", "source": "s", "tools": "t"}
        for key in base:
            changed = dict(base)
            changed[key] = "changed"
            states, finished = iter((base, changed)), []
            with self.assertRaises(builder.BuildError):
                builder.bracket(lambda: next(states), lambda _: b"built", lambda *_: finished.append(True))
            self.assertFalse(finished)

    def test_canonical_inputs_and_json_duplicate_keys(self) -> None:
        self.assertEqual(builder.canonical_source(b"x\r\n"), builder.canonical_source(b"x\n"))
        left = builder.canonical_json({"z": 1, "a": {"y": 2, "b": 3}})
        self.assertEqual(b'{"a":{"b":3,"y":2},"z":1}\n', left)
        with self.assertRaises(builder.BuildError):
            builder.decode_json(b'{"schema":"one","schema":"two"}\n')

    def test_containment_reparse_and_output_inventory_fail_closed(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            target = builder.secure_chain(root, root / "target", create=True)
            self.assertEqual(root / "target", target)
            with mock.patch.object(builder, "is_reparse", return_value=True):
                with self.assertRaises(builder.BuildError):
                    builder.secure_entry(target, directory=True)
            raw = root / "raw"
            raw.mkdir()
            (raw / "expected").write_bytes(b"x")
            (raw / "extra").write_bytes(b"x")
            with self.assertRaises(builder.BuildError):
                builder.require_exact_files(raw, {"expected"})
        with self.assertRaises(builder.BuildError):
            builder.secure_chain(ROOT, ROOT.parent / "escape", create=False)

    def test_wrapper_contract_uses_exact_exports_and_relative_default_url(self) -> None:
        exports = {"check_source", "default", "diff_caps_source", "initSync", "run_source"}
        valid = 'const u=new URL("garnet_wasm_bg.wasm",import.meta.url);export{u};'
        builder.validate_wrapper_contract(ROOT, valid.encode(), exports)
        invalid = [
            ("run_source check_source diff_caps_source", exports),
            (valid, exports - {"default"}),
            (valid.replace("garnet_wasm_bg.wasm", "https://evil.invalid/x.wasm"), exports),
        ]
        for text, observed in invalid:
            with self.assertRaises(builder.BuildError):
                builder.validate_wrapper_contract(ROOT, text.encode(), observed)

    def test_probe_is_the_only_cli_mode(self) -> None:
        with mock.patch("sys.stderr"):
            with self.assertRaises(SystemExit):
                builder.parse_args([])
            self.assertTrue(builder.parse_args(["--probe"]).probe)


if __name__ == "__main__":
    unittest.main()
