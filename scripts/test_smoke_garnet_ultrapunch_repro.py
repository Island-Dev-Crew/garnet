#!/usr/bin/env python3
"""Tests for the S110 ultrapunch reproduction recorder."""
from __future__ import annotations

import importlib.util
import contextlib
import io
import json
import os
import sys
import tempfile
import textwrap
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("smoke_garnet_ultrapunch_repro.py")
SPEC = importlib.util.spec_from_file_location("smoke_garnet_ultrapunch_repro", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
repro = importlib.util.module_from_spec(SPEC)
sys.modules["smoke_garnet_ultrapunch_repro"] = repro
SPEC.loader.exec_module(repro)


class UltrapunchReproRecorderTests(unittest.TestCase):
    def test_recorder_writes_verified_bundle_with_accept_and_rejects(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            fake = root / "fake_garnet.py"
            fake.write_text(
                textwrap.dedent(
                    """
                    import pathlib
                    import sys

                    args = sys.argv[1:]
                    if args[:2] == ["caps-log", "--verify"]:
                        path = pathlib.Path(args[2])
                        sys.exit(0 if path.is_file() else 1)

                    if not args or args[0] != "agent-loop":
                        print("unexpected command", args, file=sys.stderr)
                        sys.exit(99)

                    proposal = pathlib.Path(args[args.index("--proposal") + 1]).name
                    record = pathlib.Path(args[args.index("--record-dir") + 1])
                    seal_out = None
                    if "--seal-out" in args:
                        seal_out = pathlib.Path(args[args.index("--seal-out") + 1])
                    record.mkdir(parents=True, exist_ok=True)

                    if proposal == "accept_proposal.garnet":
                        for name in [
                            "capability_manifest.json",
                            "diff_caps.txt",
                            "seal.json",
                            "transparency_log.jsonl",
                            "decision.md",
                        ]:
                            (record / name).write_text(f"{name}\\n", encoding="utf-8")
                        if seal_out is not None:
                            seal_out.write_text("seal-out\\n", encoding="utf-8")
                        print("ACCEPTED on capability+depth evidence")
                        sys.exit(0)

                    if proposal == "reject_widen.garnet":
                        (record / "decision.md").write_text(
                            "REFUSED at diff-caps; authority expanded\\n",
                            encoding="utf-8",
                        )
                        print("AUTHORITY EXPANDED", file=sys.stderr)
                        sys.exit(7)

                    if proposal == "reject_overdepth.garnet":
                        (record / "decision.md").write_text(
                            "REFUSED at run; max_depth trap\\n",
                            encoding="utf-8",
                        )
                        print("max_depth trap", file=sys.stderr)
                        sys.exit(8)

                    print("unknown proposal", proposal, file=sys.stderr)
                    sys.exit(98)
                    """
                ).strip()
                + "\n",
                encoding="utf-8",
            )
            if os.name != "nt":
                fake.chmod(0o755)

            out = root / "proof"
            with contextlib.redirect_stdout(io.StringIO()):
                rc = repro.main(
                    [
                        "--platform",
                        "windows",
                        "--garnet",
                        sys.executable,
                        "--garnet-arg",
                        str(fake),
                        "--output-dir",
                        str(out),
                        "--format",
                        "json",
                    ]
                )
            self.assertEqual(rc, 0)

            summary_path = out / "garnet-ultrapunch-repro.json"
            manifest_path = out / "MANIFEST.sha256"
            self.assertTrue(summary_path.is_file())
            self.assertTrue(manifest_path.is_file())
            self.assertTrue((out / "garnet-ultrapunch-repro.md").is_file())

            data = json.loads(summary_path.read_text(encoding="utf-8"))
            self.assertEqual(data["schema"], "garnet.ultrapunch.repro.v1")
            self.assertEqual(data["platform"], "windows")
            self.assertEqual(data["status"], "passed")
            self.assertTrue(data["accept"]["chain_verified"])
            self.assertTrue(data["reject_widen"]["refused"])
            self.assertTrue(data["reject_overdepth"]["refused"])
            self.assertFalse(data["reject_widen"]["sealed"])
            self.assertFalse(data["reject_overdepth"]["sealed"])
            self.assertFalse(data["source_included"])
            self.assertFalse(data["provider_api_called"])

            self.assertTrue(repro.verify_bundle(summary_path))


if __name__ == "__main__":
    unittest.main()
