#!/usr/bin/env python3
"""Tests for the S107 Mac-domain proof recorder."""
from __future__ import annotations

import contextlib
import importlib.util
import io
import json
import os
import sys
import tempfile
import textwrap
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("smoke_garnet_mac_domain_proofs.py")
SPEC = importlib.util.spec_from_file_location("smoke_garnet_mac_domain_proofs", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
mac_domains = importlib.util.module_from_spec(SPEC)
sys.modules["smoke_garnet_mac_domain_proofs"] = mac_domains
SPEC.loader.exec_module(mac_domains)


class MacDomainProofRecorderTests(unittest.TestCase):
    def test_recorder_writes_verified_six_domain_bundle(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            fake = root / "fake_garnet.py"
            fake.write_text(
                textwrap.dedent(
                    """
                    import json
                    import pathlib
                    import sys

                    args = sys.argv[1:]
                    if args[:2] == ["caps-log", "--verify"]:
                        sys.exit(0 if pathlib.Path(args[2]).is_file() else 1)

                    if args and args[0] == "caps":
                        print('{"schema":"garnet-capability-manifest-v1","aggregate":["fs"],"functions":[],"wildcard":false}')
                        sys.exit(0)

                    if args and args[0] == "check":
                        print("0 diagnostics")
                        sys.exit(0)

                    if args and args[0] == "diff-caps":
                        joined = " ".join(args)
                        cap = "proc" if "proc_escalation" in joined or "supply_chain" in joined else "net"
                        print(f"caps GAINED:  {cap}")
                        print("diff-caps: AUTHORITY EXPANDED")
                        sys.exit(1)

                    if args and args[0] == "mcp-caps":
                        if "--format" in args:
                            print(json.dumps({"schema":"garnet.mcp_caps/v1","enforced":False,"high_authority":[{"tool":"shell","cap":"proc"}]}))
                        else:
                            print("aggregate authority: ffi, fs, net, proc")
                            print("! high-authority: `shell` declares `proc` — review")
                        sys.exit(0)

                    if not args or args[0] != "agent-loop":
                        print("unexpected command", args, file=sys.stderr)
                        sys.exit(99)

                    proposal = pathlib.Path(args[args.index("--proposal") + 1]).name
                    record = pathlib.Path(args[args.index("--record-dir") + 1])
                    seal_out = pathlib.Path(args[args.index("--seal-out") + 1])
                    record.mkdir(parents=True, exist_ok=True)

                    if proposal == "accept_proposal.garnet":
                        for name in [
                            "capability_manifest.json",
                            "diff_caps.txt",
                            "seal.json",
                            "transparency_log.jsonl",
                            "decision.md",
                            "run_output.txt",
                        ]:
                            (record / name).write_text(f"{name}\\n", encoding="utf-8")
                        seal_out.write_text("seal-out\\n", encoding="utf-8")
                        print("ACCEPTED on capability+depth evidence")
                        sys.exit(0)

                    if proposal == "reject_widen.garnet":
                        (record / "diff_caps.txt").write_text("AUTHORITY EXPANDED\\n", encoding="utf-8")
                        (record / "decision.md").write_text("refused\\n", encoding="utf-8")
                        print("AUTHORITY EXPANDED")
                        sys.exit(7)

                    if proposal == "reject_overdepth.garnet":
                        (record / "diff_caps.txt").write_text("no expansion\\n", encoding="utf-8")
                        (record / "run_trap.txt").write_text("@max_depth(4) exceeded\\n", encoding="utf-8")
                        (record / "decision.md").write_text("trapped\\n", encoding="utf-8")
                        print("@max_depth(4) exceeded")
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

            out = root / "bundle"
            with contextlib.redirect_stdout(io.StringIO()):
                rc = mac_domains.record_mac_domains(
                    garnet=[sys.executable, str(fake)],
                    output_dir=out,
                    format_="json",
                )
            self.assertEqual(rc, 0)

            summary = out / "garnet-mac-domain-proofs.json"
            self.assertTrue(summary.is_file())
            self.assertTrue((out / "garnet-mac-domain-proofs.md").is_file())
            self.assertTrue((out / "MANIFEST.sha256").is_file())
            self.assertTrue(mac_domains.verify_bundle(summary))

            data = json.loads(summary.read_text(encoding="utf-8"))
            self.assertEqual(data["schema"], "garnet.mac_domain_proofs.v1")
            self.assertEqual(data["status"], "passed")
            self.assertEqual(data["domain_count"], 6)
            domains = {domain["id"]: domain for domain in data["domains"]}
            self.assertTrue(domains["accept_provenance_dossier"]["sealed"])
            self.assertFalse(domains["mcp_tool_authority_creep"]["enforced"])
            for domain_id, domain in domains.items():
                if domain_id != "accept_provenance_dossier":
                    self.assertFalse(domain["sealed"])


if __name__ == "__main__":
    unittest.main()
