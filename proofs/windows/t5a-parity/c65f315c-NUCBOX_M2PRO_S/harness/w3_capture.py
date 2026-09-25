#!/usr/bin/env python3
"""Garnet T5a native-Windows parity capture harness (NUC evidence seat).

One invocation (`all`) runs, in order, stopping at the first failed gate:
  clone   remote readback of the named T5a head, fresh clone of main, the T5a
          head checked out detached in a separate git worktree
  env     environment captures
  build   cargo build --locked -p garnet-cli in the T5a worktree
  check1  diff-caps: the four named tests, then the same inputs by hand
  check2  seal subject digest: LF file vs CRLF file
  check3  seal bytes with and without a stub cosign.exe first on PATH
  check4  verify: the flagship seal/v2 accepted; a one-byte-tampered copy rejected
  check5  cargo test: minimum_shelf_package, mcp_stdio, seal_attestation
  check6  python scripts/test_garnet_windows_clean_vm_installer_status.py
  post    post-run state

Every step is a child process from an argv list (no shell), stdin closed, stdout
and stderr captured unedited into streams/. Hand-run inputs are written by a
recorded step into a scratch directory outside both checkouts, and their exact
bytes are kept in inputs.json. Every record carries this file's sha256. This is
evidence for the T5a review record; it is not a review.

Usage (Python 3.12, from native PowerShell):
    python w3_capture.py all --stamp YYYYMMDD-HHMM
    python w3_capture.py render --stamp YYYYMMDD-HHMM
"""
from __future__ import annotations

import argparse
import base64
import datetime as dt
import hashlib
import json
import os
import shutil
import subprocess
import sys
import time
from pathlib import Path

REPO_URL = "https://github.com/Island-Dev-Crew/garnet.git"
T5A_BRANCH = "train/t5a-evidence-tools"
T5A_HEAD = "c65f315c0c55c75b2f00c4217c3d1cc57fb6ff59"
STAGE_ROOT = Path(r"C:\gw1-stage\w3")
HOST = os.environ.get("COMPUTERNAME", "UNKNOWN-HOST")
PY312_DIR = r"C:\Users\IslandDevCrew\AppData\Local\Programs\Python\Python312"
PY312 = PY312_DIR + r"\python.exe"
POWERSHELL = os.path.join(os.environ["SystemRoot"], "System32", "WindowsPowerShell", "v1.0", "powershell.exe")
DEFAULT_PATH = os.environ["PATH"]
PINNED_PATH = PY312_DIR + ";" + PY312_DIR + r"\Scripts;" + DEFAULT_PATH
NOREPLY = "275140286+IslandDevCrew@users.noreply.github.com"
HARNESS_SHA256 = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
NO_PROMPT = {"GIT_TERMINAL_PROMPT": "0"}
DIFF_TESTS = ["nested_module_gain", "directory_mode_names", "swapping_two_definitions",
              "directory_mode_reports_reordered"]

# The hand-run inputs, byte for byte as garnet-cli/tests/diff_caps.rs writes them at the T5a head.
NESTED_OLD = ("module a {\n  @caps()\n  def f() -> int { 0 }\n}\nmodule b {\n  @caps()\n  def f() -> int { 0 }\n}\n"
              "@caps(fs)\ndef main() -> int { 0 }\n")
NESTED_NEW = ("module a {\n  @caps(fs)\n  def f() -> int { 0 }\n}\nmodule b {\n  @caps()\n  def f() -> int { 0 }\n}\n"
              "@caps(fs)\ndef main() -> int { 0 }\n")
SWAP_OLD = "@caps(fs)\ndef f() -> int { 0 }\n@caps()\ndef f() -> int { 0 }\n@caps(fs)\ndef main() -> int { 0 }\n"
SWAP_NEW = "@caps()\ndef f() -> int { 0 }\n@caps(fs)\ndef f() -> int { 0 }\n@caps(fs)\ndef main() -> int { 0 }\n"
FS_FIRST = "@caps(fs)\ndef f() -> int { 0 }\n@caps()\ndef f() -> int { 0 }\n"
FS_LAST = "@caps()\ndef f() -> int { 0 }\n@caps(fs)\ndef f() -> int { 0 }\n"
COSIGN_STUB_RS = 'fn main() {\n    println!("cosign stub v0");\n}\n'


class Ctx:
    def __init__(self, stamp: str):
        self.stamp = stamp
        self.clone = rf"C:\gw3-{stamp}"
        self.wt = rf"C:\gw3-{stamp}-t5a"
        self.scratch = STAGE_ROOT / f"scratch-{stamp}"
        self.seat = STAGE_ROOT / f"{T5A_HEAD[:8]}-{HOST}"
        self.log = STAGE_ROOT / f"steps-{stamp}.jsonl"
        self.run_json = STAGE_ROOT / f"run-{stamp}.json"

    def input_files(self) -> dict[str, bytes]:
        lf = (Path(self.wt) / "examples" / "minimum-shelf-flagship" / "tool.garnet").read_bytes()
        crlf = lf.replace(b"\r\n", b"\n").replace(b"\n", b"\r\n")
        files = {
            "diff/nested/old.garnet": NESTED_OLD, "diff/nested/new.garnet": NESTED_NEW,
            "diff/dirnames/old/a.garnet": "@caps()\ndef helper() -> int { 0 }\n",
            "diff/dirnames/old/b.garnet": "@caps(net)\ndef helper() -> int { 0 }\n",
            "diff/dirnames/new/a.garnet": "@caps(fs)\ndef helper() -> int { 0 }\n",
            "diff/dirnames/new/b.garnet": "@caps(net)\ndef helper() -> int { 0 }\n",
            "diff/swap/old.garnet": SWAP_OLD, "diff/swap/new.garnet": SWAP_NEW,
            "diff/dirswap/old/a.garnet": FS_FIRST, "diff/dirswap/new/a.garnet": FS_LAST,
            "diff/dirswap/old/b.garnet": "@caps()\ndef g() -> int { 0 }\n",
            "diff/dirswap/new/b.garnet": "@caps()\ndef g() -> int { 0 }\n",
            "stub/cosign.rs": COSIGN_STUB_RS,
        }
        out = {k: v.encode("utf-8") for k, v in files.items()}
        out["seal/lf/tool.garnet"] = lf
        out["seal/crlf/tool.garnet"] = crlf
        return out


def utcnow() -> str:
    return dt.datetime.now(dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%S.%fZ")


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def ps(script: str) -> list[str]:
    blob = base64.b64encode(script.encode("utf-16-le")).decode("ascii")
    return [POWERSHELL, "-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-EncodedCommand", blob]


def pyc(code: str) -> list[str]:
    return [PY312, "-I", "-c", code]


def step(id, stem, title, argv, display, *, cwd, path="pinned", env=None, timeout=900, script=None,
         gate=False, section, expect_exit=0):
    return dict(id=id, stem=stem, title=title, argv=argv, display=display, cwd=cwd, path=path, env=env or {},
                timeout=timeout, script=script, gate=gate, section=section, expect_exit=expect_exit)


def ps_step(id, stem, title, script, **kw):
    return step(id, stem, title, ps(script), script, script=script, **kw)


def garnet(ctx: Ctx) -> str:
    return rf"{ctx.wt}\target\debug\garnet.exe"


def steps_for(phase: str, ctx: Ctx) -> list[dict]:
    C, W, S = ctx.clone, ctx.wt, str(ctx.scratch)
    G = garnet(ctx)
    streams = ctx.seat / "streams"
    if phase == "clone":
        pre = ("import os, sys\n"
               f"bad = False\n"
               f"for p in (r'{C}', r'{W}', r'{S}'):\n"
               "    e = os.path.exists(p); bad |= e; print(f'{p} exists={e}')\n"
               "sys.exit(1 if bad else 0)\n")
        readback = ("import subprocess, sys\n"
                    f"out = subprocess.run(['git', '-c', 'credential.helper=', 'ls-remote', r'{REPO_URL}', "
                    f"'refs/heads/{T5A_BRANCH}', 'refs/heads/main'], capture_output=True, text=True)\n"
                    "print(out.stdout, end=''); print(out.stderr, end='', file=sys.stderr)\n"
                    f"tip = [l.split()[0] for l in out.stdout.splitlines() if l.endswith('refs/heads/{T5A_BRANCH}')]\n"
                    f"print('named_head={T5A_HEAD}')\n"
                    "print('branch_tip=' + (tip[0] if tip else 'ABSENT'))\n"
                    f"sys.exit(0 if tip and tip[0] == '{T5A_HEAD}' else 1)\n")
        s = dict(section="env")
        return [
            step("C00", "precheck", "Clone, worktree and scratch directories absent", pyc(pre),
                 "python -I -c <precheck>", cwd="C:\\", gate=True, **s),
            step("C01", "readback", "Remote readback: the T5a branch tip must equal the named full head",
                 pyc(readback), f"git ls-remote {REPO_URL} refs/heads/{T5A_BRANCH} refs/heads/main (+ equality gate)",
                 cwd="C:\\", env=NO_PROMPT, gate=True, **s),
            step("C02", "clone", "Fresh clone of main with core.autocrlf=false set before checkout",
                 ["git", "-c", "credential.helper=", "clone", "-c", "core.autocrlf=false", REPO_URL, C],
                 f"git clone -c core.autocrlf=false {REPO_URL} {C}", cwd="C:\\", env=NO_PROMPT, timeout=1800,
                 gate=True, **s),
            step("C03", "user-name", "Commit identity for the evidence clone: name",
                 ["git", "-C", C, "config", "user.name", "Jon Isaac"], 'git config user.name "Jon Isaac"', cwd=C,
                 gate=True, **s),
            step("C04", "user-email", "Commit identity for the evidence clone: noreply address",
                 ["git", "-C", C, "config", "user.email", NOREPLY], f'git config user.email "{NOREPLY}"', cwd=C,
                 gate=True, **s),
            step("C05", "fetch-t5a", "Fetch the named T5a head by full SHA",
                 ["git", "-C", C, "-c", "credential.helper=", "fetch", "origin", T5A_HEAD],
                 f"git fetch origin {T5A_HEAD}", cwd=C, env=NO_PROMPT, gate=True, **s),
            step("C06", "worktree", "Check out the T5a head detached in a separate worktree",
                 ["git", "-C", C, "worktree", "add", "--detach", W, T5A_HEAD],
                 f"git worktree add --detach {W} {T5A_HEAD}", cwd=C, gate=True, **s),
            step("C07", "wt-head", "Worktree HEAD (must be the named head)", ["git", "-C", W, "rev-parse", "HEAD"],
                 "git -C <worktree> rev-parse HEAD", cwd=W, gate=True, **s),
        ]
    if phase == "env":
        s = dict(cwd=W, section="env")
        return [
            step("E01", "hostname", "Hostname", ["hostname"], "hostname", **s),
            ps_step("E02", "systeminfo", "OS name, version and system type",
                    'systeminfo | findstr /B /C:"OS Name" /C:"OS Version" /C:"System Type"', **s),
            step("E03", "rustc", "Rust compiler", ["rustc", "-Vv"], "rustc -Vv", **s),
            step("E04", "cargo", "Cargo", ["cargo", "-V"], "cargo -V", **s),
            step("E05", "python-pinned", "python with Python312 first on PATH", ["python", "--version"],
                 "python --version", **s),
            step("E06", "python-default", "python on the machine's default PATH (disclosure: not the pinned interpreter)",
                 ["python", "--version"], "python --version", path="default", **s),
            step("E07", "git", "Git", ["git", "--version"], "git --version", **s),
            step("E08", "gh-auth", "GitHub CLI auth status (tokens are masked by gh)", ["gh", "auth", "status"],
                 "gh auth status", **s),
            step("E09", "wt-log", "T5a worktree history", ["git", "-C", W, "log", "--oneline", "-5"],
                 "git -C <worktree> log --oneline -5", **s),
            step("E10", "wt-base", "Merge base of the T5a head with main",
                 ["git", "-C", W, "merge-base", "HEAD", "origin/main"], "git -C <worktree> merge-base HEAD origin/main",
                 **s),
            step("E11", "wt-status", "Worktree status (must be clean)", ["git", "-C", W, "status", "--short", "--branch"],
                 "git -C <worktree> status --short --branch", **s),
            step("E12", "autocrlf", "core.autocrlf at every config level",
                 ["git", "-C", W, "config", "--show-origin", "--get-all", "core.autocrlf"],
                 "git config --show-origin --get-all core.autocrlf", **s),
            step("E13", "check-attr", "Checkout attributes for the flagship and the seal-sensitive prelude",
                 ["git", "-C", W, "check-attr", "text", "eol", "--", "examples/minimum-shelf-flagship/tool.garnet",
                  "examples/minimum-shelf-flagship/tool.seal.json", "garnet-interp-v0.3/src/prelude.rs"],
                 "git check-attr text eol -- <flagship tool.garnet, tool.seal.json, prelude.rs>", **s),
            step("E14", "where-cosign", "Is a real cosign on the pinned PATH? (expected: no)", ["where.exe", "cosign"],
                 "where.exe cosign", expect_exit=1, **s),
            ps_step("E15", "processes", "Process count and OneDrive processes",
                    "$p = Get-Process\n\"process_count=$($p.Count)\"\n"
                    "$p | Where-Object { $_.ProcessName -like '*OneDrive*' } | Sort-Object Id | "
                    "ForEach-Object { \"onedrive_process=$($_.ProcessName) pid=$($_.Id)\" }\n", **s),
            ps_step("E16", "env-vars", "Toolchain-relevant environment variables inherited by every step",
                    "Get-ChildItem env: | Where-Object { $_.Name -match '^(GARNET|CARGO|RUSTUP|RUSTC|RUSTFLAGS|RUST_|PYTHON|GIT_|GH_)' } | "
                    "Sort-Object Name | ForEach-Object { \"$($_.Name)=$($_.Value)\" }\n"
                    "\"(end of GARNET/CARGO/RUST/PYTHON/GIT/GH variables)\"\n", **s),
        ]
    if phase == "build":
        return [step("B01", "cargo-build", "Build garnet-cli at the T5a head",
                     ["cargo", "build", "--locked", "-p", "garnet-cli"], "cargo build --locked -p garnet-cli",
                     cwd=W, timeout=5400, gate=True, section="build"),
                step("B02", "garnet-version", "The built binary", [G, "--version"], "target\\debug\\garnet.exe --version",
                     cwd=W, section="build")]
    if phase == "check1":
        write_inputs = (
            "import json, sys, hashlib\n"
            "from pathlib import Path\n"
            f"root = Path(r'{S}')\n"
            f"files = json.loads(open(r'{ctx.seat / 'inputs.json'}', encoding='utf-8').read())['files']\n"
            "for rel, meta in sorted(files.items()):\n"
            "    p = root / rel; p.parent.mkdir(parents=True, exist_ok=True)\n"
            "    data = bytes.fromhex(meta['hex']); p.write_bytes(data)\n"
            "    assert hashlib.sha256(p.read_bytes()).hexdigest() == meta['sha256']\n"
            "    print(f\"{meta['sha256']}  {len(data):>5} B  crlf={data.count(bytes([13, 10]))}  {rel}\")\n"
        )
        s = dict(cwd=S, section="check1")
        d = lambda *p: str(ctx.scratch.joinpath(*p))
        return [
            step("K00", "write-inputs", "Write the hand-run inputs (exact bytes from inputs.json) to the scratch dir",
                 pyc(write_inputs), "python -I -c <write inputs.json files into the scratch directory>", cwd="C:\\",
                 gate=True, section="check1"),
            step("K1a", "diff-caps-tests", "The four named diff_caps tests",
                 ["cargo", "test", "--locked", "-p", "garnet-cli", "--test", "diff_caps", "--", *DIFF_TESTS],
                 "cargo test --locked -p garnet-cli --test diff_caps -- " + " ".join(DIFF_TESTS), cwd=W, timeout=3600,
                 section="check1"),
            step("K1b", "nested", "By hand: nested-module gain (expected `~ a::f gained: fs`, exit 0)",
                 [G, "diff-caps", d("diff", "nested", "old.garnet"), d("diff", "nested", "new.garnet")],
                 "garnet diff-caps diff/nested/old.garnet diff/nested/new.garnet", expect_exit=0, **s),
            step("K1c", "nested-machine", "By hand: nested-module gain, --machine",
                 [G, "diff-caps", "--machine", d("diff", "nested", "old.garnet"), d("diff", "nested", "new.garnet")],
                 "garnet diff-caps --machine diff/nested/old.garnet diff/nested/new.garnet", expect_exit=None, **s),
            step("K1d", "dirnames", "By hand: directory mode names functions by file (expected `~ a.garnet::helper gained: fs`)",
                 [G, "diff-caps", d("diff", "dirnames", "old"), d("diff", "dirnames", "new")],
                 "garnet diff-caps diff/dirnames/old diff/dirnames/new", expect_exit=None, **s),
            step("K1e", "swap", "By hand: swapped definitions of one name (expected `~ f gained: fs`)",
                 [G, "diff-caps", d("diff", "swap", "old.garnet"), d("diff", "swap", "new.garnet")],
                 "garnet diff-caps diff/swap/old.garnet diff/swap/new.garnet", expect_exit=None, **s),
            step("K1f", "dirswap", "By hand: reordered definitions in a two-file directory (expected `~ a.garnet::f gained: fs`)",
                 [G, "diff-caps", d("diff", "dirswap", "old"), d("diff", "dirswap", "new")],
                 "garnet diff-caps diff/dirswap/old diff/dirswap/new", expect_exit=None, **s),
            step("K1g", "expectations", "Check the hand-run outputs against the trigger's expectations",
                 pyc(check1_code(streams)), "python -I -c <assert the expected lines and exit codes>", cwd="C:\\",
                 section="check1"),
        ]
    if phase == "check2":
        s = dict(cwd=S, section="check2")
        return [
            step("K2a", "seal-lf", "Seal the flagship source saved with LF",
                 [G, "seal", str(ctx.scratch / "seal" / "lf" / "tool.garnet")], "garnet seal seal/lf/tool.garnet", **s),
            step("K2b", "seal-crlf", "Seal the same source saved with CRLF",
                 [G, "seal", str(ctx.scratch / "seal" / "crlf" / "tool.garnet")], "garnet seal seal/crlf/tool.garnet",
                 **s),
            step("K2c", "compare", "Subject digests must be equal (LF vs CRLF)", pyc(check2_code(streams)),
                 "python -I -c <compare subject digests of K2a and K2b>", cwd="C:\\", section="check2"),
        ]
    if phase == "check3":
        stub = ctx.scratch / "stub"
        s = dict(cwd=S, section="check3")
        return [
            step("K3a", "build-stub", "Compile the stub cosign.exe (prints a version line, exits 0)",
                 ["rustc", "-O", "-o", str(stub / "cosign.exe"), str(stub / "cosign.rs")],
                 "rustc -O -o stub/cosign.exe stub/cosign.rs", gate=True, **s),
            step("K3b", "stub-version", "The stub answers `cosign version`", [str(stub / "cosign.exe"), "version"],
                 "stub/cosign.exe version", **s),
            step("K3c", "where-with-stub", "Where cosign resolves with the stub directory first on PATH",
                 ["where.exe", "cosign"], "where.exe cosign", path="stub", **s),
            step("K3d", "seal-without", "Seal with no cosign on PATH",
                 [G, "seal", str(ctx.scratch / "seal" / "lf" / "tool.garnet")], "garnet seal seal/lf/tool.garnet", **s),
            step("K3e", "seal-with-stub", "Seal with the stub cosign.exe first on PATH",
                 [G, "seal", str(ctx.scratch / "seal" / "lf" / "tool.garnet")], "garnet seal seal/lf/tool.garnet",
                 path="stub", **s),
            step("K3f", "compare", "Seal bytes must be identical; stderr must still say UNSIGNED", pyc(check3_code(streams)),
                 "python -I -c <compare K3d and K3e>", cwd="C:\\", section="check3"),
        ]
    if phase == "check4":
        flag = Path(W) / "examples" / "minimum-shelf-flagship"
        tampered = ctx.scratch / "tamper" / "tool.seal.json"
        s = dict(cwd=W, section="check4")
        return [
            step("K4a", "verify-flagship", "The flagship seal/v2 is accepted",
                 [G, "verify", str(flag / "tool.garnet"), str(flag / "tool.seal.json")],
                 "garnet verify examples/minimum-shelf-flagship/tool.garnet examples/minimum-shelf-flagship/tool.seal.json",
                 **s),
            step("K4b", "tamper", "Make a copy with one byte changed (the first hex digit of the subject digest)",
                 pyc(tamper_code(flag / "tool.seal.json", tampered)), "python -I -c <copy the seal, flip one byte>",
                 cwd="C:\\", gate=True, section="check4"),
            step("K4c", "verify-tampered", "The tampered copy is rejected (expected rc 2)",
                 [G, "verify", str(flag / "tool.garnet"), str(tampered)],
                 "garnet verify examples/minimum-shelf-flagship/tool.garnet <scratch>/tamper/tool.seal.json",
                 expect_exit=2, **s),
        ]
    if phase == "check5":
        return [step("K5", "shelf-mcp-seal-tests", "minimum_shelf_package, mcp_stdio and seal_attestation tests",
                     ["cargo", "test", "--locked", "-p", "garnet-cli", "--test", "minimum_shelf_package", "--test",
                      "mcp_stdio", "--test", "seal_attestation"],
                     "cargo test --locked -p garnet-cli --test minimum_shelf_package --test mcp_stdio --test seal_attestation",
                     cwd=W, timeout=3600, section="check5")]
    if phase == "check6":
        return [step("K6", "clean-vm-reader-tests", "Committed-bundle reader tests, natively (verbose, so each skip is named)",
                     ["python", "scripts/test_garnet_windows_clean_vm_installer_status.py", "-v"],
                     "python scripts/test_garnet_windows_clean_vm_installer_status.py -v", cwd=W, timeout=1800,
                     section="check6")]
    if phase == "post":
        readback = ("import subprocess\n"
                    f"out = subprocess.run(['git', '-c', 'credential.helper=', 'ls-remote', r'{REPO_URL}', "
                    f"'refs/heads/{T5A_BRANCH}', 'refs/heads/main'], capture_output=True, text=True)\n"
                    "print(out.stdout, end='')\n")
        s = dict(section="post")
        return [
            step("P01", "wt-status", "T5a worktree status after every step",
                 ["git", "-C", W, "status", "--short", "--branch"], "git -C <worktree> status --short --branch", cwd=W,
                 **s),
            step("P02", "readback-after", "Remote readback after every step", pyc(readback),
                 f"git ls-remote {REPO_URL} refs/heads/{T5A_BRANCH} refs/heads/main", cwd="C:\\", env=NO_PROMPT, **s),
            ps_step("P03", "processes-after", "Process count after every step",
                    "\"process_count=$((Get-Process).Count)\"\n", cwd=W, **s),
        ]
    raise SystemExit(f"unknown phase {phase}")


def check1_code(streams: Path) -> str:
    return (
        "import sys\n"
        f"S = r'{streams}'\n"
        "def out(stem): return open(S + '\\\\' + stem + '.stdout', 'rb').read().decode('utf-8', 'replace')\n"
        "import json\n"
        "rec = {}\n"
        "ok = True\n"
        "def expect(label, cond):\n"
        "    global ok\n"
        "    ok &= bool(cond); print(('PASS ' if cond else 'FAIL ') + label)\n"
        "expect('K1a ran the four named tests: `4 passed; 0 failed`', 'test result: ok. 4 passed; 0 failed' in out('K1a-diff-caps-tests'))\n"
        "expect('K1b stdout contains `~ a::f gained: fs`', '~ a::f gained: fs' in out('K1b-nested'))\n"
        "expect('K1c --machine names a::f', '{\"name\":\"a::f\",\"gained\":[\"fs\"]}' in out('K1c-nested-machine'))\n"
        "expect('K1d stdout contains `~ a.garnet::helper gained: fs`', '~ a.garnet::helper gained: fs' in out('K1d-dirnames'))\n"
        "expect('K1d stdout has no `helper gained: net`', 'helper gained: net' not in out('K1d-dirnames'))\n"
        "expect('K1e stdout contains `~ f gained: fs`', '~ f gained: fs' in out('K1e-swap'))\n"
        "expect('K1f stdout contains `~ a.garnet::f gained: fs`', '~ a.garnet::f gained: fs' in out('K1f-dirswap'))\n"
        "sys.exit(0 if ok else 1)\n"
    )


def check2_code(streams: Path) -> str:
    return (
        "import json, sys\n"
        f"S = r'{streams}'\n"
        "def stmt(stem): return json.loads(open(S + '\\\\' + stem + '.stdout', 'rb').read().decode('utf-8'))\n"
        "a, b = stmt('K2a-seal-lf'), stmt('K2b-seal-crlf')\n"
        "da, db = a['subject'][0]['digest'], b['subject'][0]['digest']\n"
        "print('lf   subject=' + json.dumps(a['subject']))\n"
        "print('crlf subject=' + json.dumps(b['subject']))\n"
        "print('lf   predicate.source_blake3=' + str(a['predicate'].get('source_blake3')))\n"
        "print('crlf predicate.source_blake3=' + str(b['predicate'].get('source_blake3')))\n"
        "print('subject_identity=' + str(a['predicate'].get('subject_identity')))\n"
        "print('subject_digests_equal=' + str(da == db))\n"
        "print('whole_statements_equal=' + str(a == b))\n"
        "sys.exit(0 if da == db else 1)\n"
    )


def check3_code(streams: Path) -> str:
    return (
        "import sys\n"
        f"S = r'{streams}'\n"
        "def rd(n): return open(S + '\\\\' + n, 'rb').read()\n"
        "o1, o2 = rd('K3d-seal-without.stdout'), rd('K3e-seal-with-stub.stdout')\n"
        "e1, e2 = rd('K3d-seal-without.stderr'), rd('K3e-seal-with-stub.stderr')\n"
        "import hashlib\n"
        "print('without stdout sha256=' + hashlib.sha256(o1).hexdigest() + f' bytes={len(o1)}')\n"
        "print('with    stdout sha256=' + hashlib.sha256(o2).hexdigest() + f' bytes={len(o2)}')\n"
        "print('seal_bytes_identical=' + str(o1 == o2))\n"
        "print('with-stub stderr says UNSIGNED=' + str(b'UNSIGNED' in e2))\n"
        "print('stderr differs between the runs=' + str(e1 != e2))\n"
        "sys.exit(0 if o1 == o2 and b'UNSIGNED' in e2 else 1)\n"
    )


def tamper_code(src: Path, dst: Path) -> str:
    return (
        "import hashlib, json\n"
        "from pathlib import Path\n"
        f"src, dst = Path(r'{src}'), Path(r'{dst}')\n"
        "data = bytearray(src.read_bytes())\n"
        "digest = json.loads(data.decode('utf-8'))['subject'][0]['digest']\n"
        "key, value = next(iter(digest.items()))\n"
        "i = bytes(data).index(value.encode())\n"
        "old = chr(data[i]); new = '0' if old != '0' else '1'\n"
        "data[i] = ord(new)\n"
        "dst.parent.mkdir(parents=True, exist_ok=True); dst.write_bytes(bytes(data))\n"
        "json.loads(dst.read_bytes().decode('utf-8'))\n"
        "print(f'subject digest ({key}) first hex digit at byte offset {i}: {old!r} -> {new!r}')\n"
        "print('original sha256=' + hashlib.sha256(src.read_bytes()).hexdigest())\n"
        "print('tampered sha256=' + hashlib.sha256(dst.read_bytes()).hexdigest())\n"
        "print('bytes differing=' + str(sum(1 for x, y in zip(src.read_bytes(), dst.read_bytes()) if x != y)))\n"
    )


PHASES = ("clone", "env", "build", "check1", "check2", "check3", "check4", "check5", "check6", "post")


def kill_tree(pid: int) -> None:
    subprocess.run(["taskkill", "/T", "/F", "/PID", str(pid)], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)


def run_step(ctx: Ctx, s: dict, phase: str) -> dict:
    env = dict(os.environ)
    env["PATH"] = {"pinned": PINNED_PATH, "default": DEFAULT_PATH,
                   "stub": str(ctx.scratch / "stub") + ";" + PINNED_PATH}[s["path"]]
    env.update(s["env"])
    exe = s["argv"][0] if os.path.isabs(s["argv"][0]) else (shutil.which(s["argv"][0], path=env["PATH"]) or s["argv"][0])
    argv = [exe] + s["argv"][1:]
    streams = ctx.seat / "streams"
    streams.mkdir(parents=True, exist_ok=True)
    outs = [(streams / f"{s['id']}-{s['stem']}.stdout", "stdout"), (streams / f"{s['id']}-{s['stem']}.stderr", "stderr")]
    cwd = s["cwd"] if Path(s["cwd"]).exists() else "C:\\"
    started, t0, timed_out = utcnow(), time.monotonic(), False
    handles = [open(p, "wb") for p, _ in outs]
    try:
        proc = subprocess.Popen(argv, cwd=cwd, env=env, stdin=subprocess.DEVNULL, stdout=handles[0], stderr=handles[1])
        try:
            rc = proc.wait(timeout=s["timeout"])
        except subprocess.TimeoutExpired:
            timed_out = True
            kill_tree(proc.pid)
            rc = proc.wait()
    finally:
        for h in handles:
            h.close()
    rec = {k: s[k] for k in ("id", "stem", "title", "display", "section", "script", "expect_exit")}
    rec.update(phase=phase, cwd=cwd, argv=argv, resolved_exe=exe, path_mode=s["path"], env_overrides=s["env"],
               started_utc=started, ended_utc=utcnow(), duration_s=round(time.monotonic() - t0, 3), exit_code=rc,
               timed_out=timed_out, timeout_s=s["timeout"], harness_sha256=HARNESS_SHA256,
               streams=[{"file": p.relative_to(ctx.seat).as_posix(), "kind": k, "bytes": p.stat().st_size,
                         "sha256": sha256_file(p)} for p, k in outs])
    return rec


def write_inputs_json(ctx: Ctx) -> None:
    files = ctx.input_files()
    doc = {"note": "Exact bytes of every hand-run input, written by step K00 into the scratch directory. "
                   "The diff-caps inputs match garnet-cli/tests/diff_caps.rs at the T5a head; the seal inputs are "
                   "examples/minimum-shelf-flagship/tool.garnet at the T5a head (LF) and the same text with CRLF.",
           "files": {rel: {"sha256": hashlib.sha256(b).hexdigest(), "bytes": len(b), "crlf": b.count(b"\r\n"),
                           "text": b.decode("utf-8"), "hex": b.hex()} for rel, b in sorted(files.items())}}
    (ctx.seat / "inputs.json").write_bytes((json.dumps(doc, ensure_ascii=False, indent=2, sort_keys=True) + "\n")
                                           .encode("utf-8"))


def cmd_all(ctx: Ctx) -> int:
    if ctx.log.exists() or ctx.seat.exists():
        raise SystemExit(f"refusing to mix runs: {ctx.log} or {ctx.seat} already exists")
    ctx.seat.mkdir(parents=True)
    run = {"harness_sha256": HARNESS_SHA256, "python": sys.executable, "python_version": sys.version,
           "argv": sys.argv, "started_utc": utcnow(), "clone": ctx.clone, "worktree": ctx.wt,
           "scratch": str(ctx.scratch), "t5a_head": T5A_HEAD, "stage": str(ctx.seat), "host": HOST,
           "phases": list(PHASES), "stopped_at": None, "unexpected_exits": []}
    for phase in PHASES:
        if phase == "check1":
            write_inputs_json(ctx)
        for s in steps_for(phase, ctx):
            rec = run_step(ctx, s, phase)
            with open(ctx.log, "a", encoding="utf-8", newline="\n") as f:
                f.write(json.dumps(rec, ensure_ascii=False) + "\n")
            asserted = s["expect_exit"] is not None
            flag = "" if not asserted or rec["exit_code"] == s["expect_exit"] else "  <-- UNEXPECTED EXIT"
            print(f"{rec['id']} {rec['stem']}: exit={rec['exit_code']} "
                  f"(expected {s['expect_exit'] if asserted else 'not asserted'}) {rec['duration_s']}s{flag}", flush=True)
            if flag:
                run["unexpected_exits"].append(rec["id"])
                if s["gate"]:
                    run["stopped_at"] = rec["id"]
                    break
        if run["stopped_at"]:
            print(f"GATE FAILED at {run['stopped_at']}; stopping", flush=True)
            break
    run["ended_utc"] = utcnow()
    ctx.run_json.write_text(json.dumps(run, indent=2, sort_keys=True) + "\n", encoding="utf-8", newline="\n")
    return 2 if run["stopped_at"] else (1 if run["unexpected_exits"] else 0)


# ---------------------------------------------------------------- render ----

def census_row(path: Path, rel: str) -> dict:
    data = path.read_bytes()
    bom = "utf-8" if data.startswith(b"\xef\xbb\xbf") else "utf-16le" if data.startswith(b"\xff\xfe") else \
        "utf-16be" if data.startswith(b"\xfe\xff") else "none"
    try:
        data.decode("utf-8")
        utf8 = True
    except UnicodeDecodeError:
        utf8 = False
    crlf = data.count(b"\r\n")
    lf = data.count(b"\n") - crlf
    cr = data.count(b"\r") - crlf
    endings = ("empty" if not data else "no-newline" if not (crlf or lf or cr) else "CRLF" if crlf and not lf and not cr
               else "LF" if lf and not crlf and not cr else "MIXED")
    return dict(path=rel, bytes=len(data), sha256=hashlib.sha256(data).hexdigest(), utf8=utf8, bom=bom, endings=endings,
                lf=lf, crlf=crlf, cr=cr, nonascii=sum(1 for b in data if b > 127), nul=data.count(0),
                finalnl=data.endswith(b"\n"))


def section_doc(title: str, recs: list[dict], ctx: Ctx, preamble: str) -> bytes:
    out = bytearray(f"# {title}\n{preamble}".encode())
    for r in recs:
        out += b"\n" + b"=" * 78 + b"\n" + f"[{r['id']}] {r['title']}\n".encode()
        if r["script"]:
            out += b"$ (PowerShell 5.1)\n  " + r["script"].rstrip("\n").replace("\n", "\n  ").encode() + b"\n"
        else:
            out += f"$ {r['display']}\n".encode()
        exp = r["expect_exit"] if r["expect_exit"] is not None else "not asserted"
        out += (f"exit_code={r['exit_code']} expected={exp} timed_out={r['timed_out']} started={r['started_utc']} "
                f"duration_s={r['duration_s']} path={r['path_mode']}\n").encode()
        for st in r["streams"]:
            raw = (ctx.seat / st["file"]).read_bytes()
            out += f"--- {st['kind']} ({st['file']}, {st['bytes']} bytes, sha256 {st['sha256']})\n".encode() + raw
            if raw and not raw.endswith(b"\n"):
                out += b"\n[no trailing newline in the captured stream]\n"
            out += f"--- end {st['kind']}\n".encode()
    return bytes(out)


def cmd_render(ctx: Ctx) -> None:
    recs = [json.loads(l) for l in ctx.log.read_text(encoding="utf-8").splitlines() if l.strip()]
    run = json.loads(ctx.run_json.read_text(encoding="utf-8"))
    by: dict[str, list] = {}
    for r in recs:
        by.setdefault(r["section"], []).append(r)
    pre = (f"folder: proofs/windows/t5a-parity/{ctx.seat.name}/\n"
           f"t5a head: {T5A_HEAD} (branch {T5A_BRANCH}; read back from the remote in C01 and P02)\n"
           f"host: {HOST} (%COMPUTERNAME%; `hostname` prints the DNS casing, see E01)\n"
           f"stamp: {ctx.stamp} (UTC)\n"
           f"harness: harness/w3_capture.py, sha256 {run['harness_sha256']}; every step ran from this one invocation\n"
           f"  ({' '.join(Path(a).name if i == 0 else a for i, a in enumerate(run['argv']))}), "
           f"{run['started_utc']} to {run['ended_utc']}.\n"
           "scope: evidence for the T5a review record, gathered on native Windows. It is not a review.\n"
           "python: every python step ran with Python312 first on PATH except E06, which shows the machine's default\n"
           "  PATH (an unrelated 3.11.15 virtualenv). Path mode `stub` puts the stub cosign.exe directory first.\n"
           "inputs: hand-run inputs are written by K00 into a scratch directory outside both checkouts; their exact\n"
           "  bytes are in inputs.json.\n"
           "powershell: PowerShell steps run powershell.exe 5.1 with -EncodedCommand; with redirected output it writes\n"
           "  progress/error records to stderr as CLIXML. The raw streams are authoritative.\n"
           "captures: stdout and stderr are the raw child-process bytes, unedited; see encoding-census.txt.\n")
    docs = {"env.txt": ("Readback, clone, worktree and environment", ["env"]),
            "build.txt": ("Build at the T5a head", ["build"]),
            "check-1-diff-caps.txt": ("Check 1: diff-caps, tests and by hand", ["check1"]),
            "check-2-seal-line-endings.txt": ("Check 2: seal subject digest, LF vs CRLF", ["check2"]),
            "check-3-seal-cosign.txt": ("Check 3: seal bytes with and without a stub cosign.exe", ["check3"]),
            "check-4-verify-tamper.txt": ("Check 4: flagship seal accepted, tampered copy rejected", ["check4"]),
            "check-5-tests.txt": ("Check 5: shelf, MCP and seal tests", ["check5"]),
            "check-6-clean-vm-reader.txt": ("Check 6: committed-bundle reader tests", ["check6"]),
            "post.txt": ("Post-run state", ["post"])}
    for name, (title, sections) in docs.items():
        (ctx.seat / name).write_bytes(section_doc(title, [r for sec in sections for r in by.get(sec, [])], ctx, pre))
    lines = [f"# Commands run for proofs/windows/t5a-parity/{ctx.seat.name}/", pre.rstrip("\n"),
             f"run: stopped_at={run['stopped_at']} unexpected_exits={run['unexpected_exits']}", ""]
    for r in recs:
        lines += ["=" * 78, f"[{r['id']}] {r['title']}   (phase {r['phase']})"]
        if r["script"]:
            lines.append("command (PowerShell script):")
            lines += ["    " + l for l in r["script"].rstrip("\n").split("\n")]
            argv_shown = r["argv"][:-1] + ["<base64 UTF-16LE of the script above>"]
        else:
            lines.append(f"command: {r['display']}")
            argv_shown = r["argv"]
        exp = r["expect_exit"] if r["expect_exit"] is not None else "not asserted"
        lines += ["argv: " + json.dumps(argv_shown, ensure_ascii=False), f"resolved_exe: {r['resolved_exe']}",
                  f"cwd: {r['cwd']}   path: {r['path_mode']}   env_overrides: {json.dumps(r['env_overrides'])}",
                  f"started_utc: {r['started_utc']}   ended_utc: {r['ended_utc']}   duration_s: {r['duration_s']}",
                  f"exit_code: {r['exit_code']}   expected: {exp}   timed_out: {r['timed_out']}   timeout_s: {r['timeout_s']}"]
        lines += [f"stream: {st['kind']:<7} {st['sha256']}  {st['bytes']:>9} B  {st['file']}" for st in r["streams"]]
    (ctx.seat / "commands.txt").write_bytes(("\n".join(lines) + "\n").encode("utf-8"))
    (ctx.seat / "commands.json").write_bytes(
        (json.dumps({"run": run, "steps": recs}, ensure_ascii=False, indent=2, sort_keys=True) + "\n").encode())
    rels = sorted(p.relative_to(ctx.seat).as_posix() for p in ctx.seat.rglob("*")
                  if p.is_file() and p.name not in ("MANIFEST.sha256", "encoding-census.txt"))
    rows = [census_row(ctx.seat / rel, rel) for rel in rels]
    hdr = (f"{'path':<56} {'bytes':>8} {'utf8':>5} {'bom':>5} {'endings':>10} {'lf':>6} {'crlf':>6} {'cr':>3} "
           f"{'nonascii':>8} {'nul':>3} {'finalnl':>7}  sha256")
    cl = ["# Encoding census (UTF-8 validity, BOM, line endings) for every file in this folder, sorted bytewise",
          "# excludes this file and MANIFEST.sha256 (both UTF-8, no BOM, LF).",
          "# lf = bare LF, crlf = CRLF pairs, cr = lone CR; endings = LF | CRLF | MIXED | no-newline | empty.", "", hdr]
    cl += [f"{r['path']:<56} {r['bytes']:>8} {str(r['utf8']):>5} {r['bom']:>5} {r['endings']:>10} {r['lf']:>6} "
           f"{r['crlf']:>6} {r['cr']:>3} {r['nonascii']:>8} {r['nul']:>3} {str(r['finalnl']):>7}  {r['sha256']}"
           for r in rows]
    (ctx.seat / "encoding-census.txt").write_bytes(("\n".join(cl) + "\n").encode("utf-8"))
    allf = sorted(p.relative_to(ctx.seat).as_posix() for p in ctx.seat.rglob("*")
                  if p.is_file() and p.name != "MANIFEST.sha256")
    (ctx.seat / "MANIFEST.sha256").write_bytes("".join(f"{sha256_file(ctx.seat / r)}  {r}\n" for r in allf).encode())
    print(f"rendered {len(allf)} files + MANIFEST.sha256 in {ctx.seat}")


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("mode", choices=("all", "render"))
    ap.add_argument("--stamp", required=True)
    a = ap.parse_args()
    ctx = Ctx(a.stamp)
    if a.mode == "render":
        cmd_render(ctx)
        return 0
    return cmd_all(ctx)


if __name__ == "__main__":
    sys.exit(main())
