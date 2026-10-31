#!/usr/bin/env python3
"""Reproduce the recorded CLI journey with a locally built Garnet binary.

Run: python3 replay.py /absolute/path/to/garnet
All output goes into a temporary directory. No program is executed and no
signing key is used. These are source/content checks, not human approval.
"""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile


def replay(binary):
    binary = str(Path(binary).resolve())
    here = Path(__file__).resolve().parent
    records = []
    with tempfile.TemporaryDirectory(prefix="garnet-acceptance-") as folder:
        work = Path(folder)
        for name in ("before.garnet", "accepted.garnet", "changed.garnet"):
            (work / name).write_bytes((here / name).read_bytes())

        def run(args, expected):
            result = subprocess.run([binary, *args], cwd=work, text=True,
                                    capture_output=True, timeout=30)
            record = {"command": "garnet " + " ".join(args),
                      "exit_code": result.returncode,
                      "stdout": result.stdout, "stderr": result.stderr}
            records.append(record)
            assert result.returncode == expected, record

        run(["check", "before.garnet"], 1)
        run(["check", "accepted.garnet"], 0)
        run(["diff-caps", "before.garnet", "accepted.garnet"], 1)
        run(["seal", "accepted.garnet", "--out", "seal.json"], 0)
        first = (work / "seal.json").read_bytes()
        run(["seal", "accepted.garnet", "--out", "repeat.json"], 0)
        assert first == (work / "repeat.json").read_bytes()
        run(["verify", "accepted.garnet", "seal.json"], 0)
        run(["verify", "changed.garnet", "seal.json"], 2)
        assert not (work / "report.txt").exists()
        return {"schema": "garnet.website.cli-record/1",
                "scope": "Recorded current-source CLI; seal/v2 content binding, unsigned. No program execution, signature verification, or human approval attestation.",
                "source_revision": "aba11b329e3284845ba7c9cd4c870998503a9030",
                "binary_sha256": hashlib.sha256(Path(binary).read_bytes()).hexdigest(),
                "repeat_seal_byte_identical": True, "filesystem_call_executed": False,
                "sources": {name: hashlib.sha256((here / name).read_bytes()).hexdigest()
                            for name in ("before.garnet", "accepted.garnet", "changed.garnet")},
                "records": records, "seal": json.loads(first)}


if __name__ == "__main__":
    if len(sys.argv) != 2:
        raise SystemExit("usage: python3 replay.py /absolute/path/to/garnet")
    print(json.dumps(replay(sys.argv[1]), indent=2))
