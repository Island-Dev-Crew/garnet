#!/usr/bin/env python3
"""Garnet native-Windows baseline capture harness (NUC evidence seat).

One invocation (`all`) makes a fresh clone of the public repo, records the
environment, runs the installer checks, builds garnet-cli, runs the workspace
tests and records post-run state, in that order, stopping at the first failed
gate. Every step is a child process started from an argv list (never through a
shell) with stdin closed. stdout and stderr are written unedited to separate
files under streams/, except the cargo test step, whose stdout and stderr share
one file (cargo-test.log) so each failure stays next to the test binary that
produced it. Each step appends one JSON record (including this file's sha256)
to <stage>/steps-<stamp>.jsonl. `render` then writes env.txt, installer-*.txt,
build-steps.txt, commands.txt / commands.json, test-summary.txt, the encoding
census and MANIFEST.sha256 from those records.

The bundle is staged outside the clone. The harness writes nothing in the clone
except through the commands it runs: `git config` sets the clone's commit
identity, and cargo writes its git-ignored target/ directory and test caches.

Usage (Python 3.12, launched from native PowerShell):
    python nuc_capture.py all --stamp YYYYMMDD-HHMM
    python nuc_capture.py render --stamp YYYYMMDD-HHMM
"""
from __future__ import annotations

import argparse
import base64
import datetime as dt
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import time
from pathlib import Path

REPO_URL = "https://github.com/Island-Dev-Crew/garnet.git"
CLONE = r"C:\gw1c-20260924"
STAGE_ROOT = Path(r"C:\gw1-stage")
HOST = os.environ.get("COMPUTERNAME", "UNKNOWN-HOST")
PY312_DIR = r"C:\Users\IslandDevCrew\AppData\Local\Programs\Python\Python312"
PY312 = PY312_DIR + r"\python.exe"
POWERSHELL = os.path.join(os.environ["SystemRoot"], "System32", "WindowsPowerShell", "v1.0", "powershell.exe")
DEFAULT_PATH = os.environ["PATH"]
PINNED_PATH = PY312_DIR + ";" + PY312_DIR + r"\Scripts;" + DEFAULT_PATH
EMPTY_RELEASE_DIR = r"C:\gw1-stage\empty-release-dir"
SCRATCH_ZIP = r"C:\gw1-stage\scratch\garnet-0.8.2-x86_64-pc-windows-msvc.zip"
LIVE_INSTALLER_URL = "https://garnet-lang.org/install.ps1"
RELEASE = "https://github.com/Island-Dev-Crew/garnet/releases/download/v0.8.2"
NOREPLY = "275140286+IslandDevCrew@users.noreply.github.com"
PREFIXES = ("garnet-nuc-install-test", "garnet-nuc-install-test-main", "garnet-nuc-install-test-hint",
            "garnet-nuc-install-test-v081", "garnet-nuc-install-test-mode-source")
INSTALL_PS1 = CLONE + r"\docs\install.ps1"
HARNESS_SHA256 = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()


def utcnow() -> str:
    return dt.datetime.now(dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%S.%fZ")


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def ps(script: str) -> list[str]:
    """powershell.exe argv carrying `script` exactly, via -EncodedCommand (UTF-16LE base64)."""
    blob = base64.b64encode(script.encode("utf-16-le")).decode("ascii")
    return [POWERSHELL, "-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-EncodedCommand", blob]


def pyc(code: str) -> list[str]:
    return [PY312, "-I", "-c", code]


def step(id, stem, title, argv, display, *, cwd=CLONE, path="pinned", env=None, merged_file=None,
         timeout=900, ps_script=None, gate=False, section="env", expect_exit=0, fetched=None):
    return dict(id=id, stem=stem, title=title, argv=argv, display=display, cwd=cwd, path=path,
                env=env or {}, merged_file=merged_file, timeout=timeout, ps_script=ps_script,
                gate=gate, section=section, expect_exit=expect_exit, fetched=fetched or [])


def ps_step(id, stem, title, script, **kw):
    return step(id, stem, title, ps(script), script, ps_script=script, **kw)


def curl(url: str, out: str) -> list[str]:
    return ["curl.exe", "-sS", "-L", "--proto", "=https", "--proto-redir", "=https", "--create-dirs", "-o", out,
            "-w", "http_code=%{http_code} size=%{size_download} redirects=%{num_redirects}\\n", url]


NO_PROMPT = {"GIT_TERMINAL_PROMPT": "0"}

PRECHECK = (
    "import os, sys\n"
    f"clone = r'{CLONE}'\n"
    "bad = os.path.exists(clone)\n"
    "print(f'{clone} exists={bad}')\n"
    f"for s in {PREFIXES!r}:\n"
    "    p = os.path.join(os.environ['TEMP'], s); e = os.path.exists(p); bad |= e\n"
    "    print(f'{p} exists={e}')\n"
    f"d = r'{EMPTY_RELEASE_DIR}'\n"
    "entries = sorted(os.listdir(d)) if os.path.isdir(d) else None\n"
    "print(f'{d} isdir={os.path.isdir(d)} entries={entries}')\n"
    "bad |= entries != []\n"
    "sys.exit(1 if bad else 0)\n"
)

USER_PATH_PROBE = (
    "$u = [Environment]::GetEnvironmentVariable('Path', 'User')\n"
    "$b = [Text.Encoding]::UTF8.GetBytes([string]$u)\n"
    "$h = [BitConverter]::ToString([Security.Cryptography.SHA256]::Create().ComputeHash($b)).Replace('-', '').ToLower()\n"
    "\"user_path_sha256=$h\"\n"
    "\"user_path_entries=$(@($u -split ';' | Where-Object { $_ }).Count)\"\n"
    "\"user_path_mentions_garnet_nuc_install_test=$([bool]($u -match 'garnet-nuc-install-test'))\"\n"
)

LIVE_RUN = (
    '$env:GARNET_PREFIX = "$env:TEMP\\garnet-nuc-install-test"\n'
    '$env:GARNET_NO_MODIFY_PATH = "1"\n'
    "irm https://garnet-lang.org/install.ps1 | iex\n"
    '& "$env:GARNET_PREFIX\\bin\\garnet.exe" --version\n'
)


def main_run(prefix: str, extra: str = "", version_line: bool = True) -> str:
    return (
        f'$env:GARNET_PREFIX = "$env:TEMP\\{prefix}"\n'
        '$env:GARNET_NO_MODIFY_PATH = "1"\n'
        + extra
        + f"Get-Content -Raw -LiteralPath '{INSTALL_PS1}' | iex\n"
        + ('& "$env:GARNET_PREFIX\\bin\\garnet.exe" --version\n' if version_line else "")
    )


EXE_PROBE = (
    f"foreach ($s in {', '.join(repr(p) for p in PREFIXES)}) {{\n"
    "    $exe = Join-Path $env:TEMP \"$s\\bin\\garnet.exe\"\n"
    "    if (Test-Path -LiteralPath $exe) {\n"
    "        $h = (Get-FileHash -Algorithm SHA256 -LiteralPath $exe).Hash.ToLower()\n"
    "        $sig = (Get-AuthenticodeSignature -LiteralPath $exe).Status\n"
    "        \"$s garnet.exe sha256=$h bytes=$((Get-Item -LiteralPath $exe).Length) authenticode=$sig\"\n"
    "    } else {\n"
    "        \"$s garnet.exe absent\"\n"
    "    }\n"
    "}\n"
)


def compare_code(live_path: str) -> str:
    return (
        "import hashlib, sys\n"
        f"live = open(r'{live_path}', 'rb').read()\n"
        f"main = open(r'{INSTALL_PS1}', 'rb').read()\n"
        "print('live  sha256=' + hashlib.sha256(live).hexdigest() + f' bytes={len(live)}')\n"
        "print('main  sha256=' + hashlib.sha256(main).hexdigest() + f' bytes={len(main)}')\n"
        "print('byte_identical=' + str(live == main))\n"
        "sys.exit(0 if live == main else 1)\n"
    )


def decode_code(streams: str, stem: str, needle: str) -> str:
    return (
        "import glob, html, re, sys\n"
        f"paths = sorted(glob.glob(r'{streams}\\{stem}.*'))\n"
        "raw = b''.join(open(p, 'rb').read() for p in paths)\n"
        f"err = open(r'{streams}\\{stem}.stderr', 'rb').read().decode('utf-8', 'replace')\n"
        "text = html.unescape(''.join(re.findall(r'<S S=\"Error\">(.*?)</S>', err, re.S))).replace('_x000D__x000A_', '\\n')\n"
        f"needle = {needle!r}\n"
        "found = needle.encode() in raw or needle in text.replace('\\n', '')\n"
        "print('streams=' + ','.join(p.rsplit(chr(92), 1)[-1] for p in paths))\n"
        "print(f'contains {needle!r}: {found}')\n"
        "print('--- decoded view of the CLIXML error records in the stderr stream (derived; the raw stream is authoritative)')\n"
        "print(text.rstrip())\n"
        "sys.exit(0 if found else 1)\n"
    )


def zip_code(sums: str) -> str:
    return (
        "import hashlib, os, zipfile\n"
        f"z = r'{SCRATCH_ZIP}'\n"
        "data = open(z, 'rb').read()\n"
        "print('zip_sha256=' + hashlib.sha256(data).hexdigest() + f' bytes={len(data)} (the zip is kept outside the bundle)')\n"
        f"want = [l.split()[0].decode() for l in open(r'{sums}', 'rb').read().splitlines() "
        "if l.endswith(b'garnet-0.8.2-x86_64-pc-windows-msvc.zip')]\n"
        "print('SHA256SUMS_entry=' + (want[0] if want else 'ABSENT'))\n"
        "print('zip_matches_SHA256SUMS=' + str(bool(want) and want[0] == hashlib.sha256(data).hexdigest()))\n"
        "with zipfile.ZipFile(z) as zf:\n"
        "    for i in zf.infolist(): print(f'entry {i.filename} size={i.file_size}')\n"
        "    exe = zf.read('garnet.exe')\n"
        "print('zip_garnet_exe_sha256=' + hashlib.sha256(exe).hexdigest())\n"
    )


def steps_for(phase: str, bundle: Path) -> list[dict]:
    streams = str(bundle / "streams")
    live = str(bundle / "streams" / "L01-install.ps1.live")
    sums = str(bundle / "streams" / "L07-SHA256SUMS.v0.8.2")
    if phase == "clone":
        return [
            step("C00", "precheck", "Clone directory and throwaway install prefixes absent; empty release dir empty",
                 pyc(PRECHECK), "python -I -c <precheck>", cwd="C:\\", gate=True),
            step("C01", "remote-main", "Remote readback of main before cloning (no credential)",
                 ["git", "-c", "credential.helper=", "ls-remote", REPO_URL, "refs/heads/main"],
                 f"git -c credential.helper= ls-remote {REPO_URL} refs/heads/main", cwd="C:\\", env=NO_PROMPT),
            step("C02", "clone", "Fresh clone with core.autocrlf=false set before checkout",
                 ["git", "-c", "credential.helper=", "clone", "-c", "core.autocrlf=false", REPO_URL, CLONE],
                 f"git clone -c core.autocrlf=false {REPO_URL} {CLONE}", cwd="C:\\", env=NO_PROMPT,
                 timeout=1800, gate=True),
            step("C03", "user-name", "Commit identity for this clone: name", ["git", "-C", CLONE, "config", "user.name",
                 "Jon Isaac"], 'git config user.name "Jon Isaac"', gate=True),
            step("C04", "user-email", "Commit identity for this clone: noreply address",
                 ["git", "-C", CLONE, "config", "user.email", NOREPLY], f'git config user.email "{NOREPLY}"', gate=True),
        ]
    if phase == "env":
        return [
            step("E01", "hostname", "Hostname", ["hostname"], "hostname"),
            ps_step("E02", "systeminfo", "OS name, version and system type",
                    'systeminfo | findstr /B /C:"OS Name" /C:"OS Version" /C:"System Type"'),
            step("E03", "rustc", "Rust compiler", ["rustc", "-Vv"], "rustc -Vv"),
            step("E04", "cargo", "Cargo", ["cargo", "-V"], "cargo -V"),
            step("E05", "rustup-active", "Active rustup toolchain", ["rustup", "show", "active-toolchain"],
                 "rustup show active-toolchain"),
            step("E06", "node", "Node.js", ["node", "-v"], "node -v"),
            step("E07", "python-pinned", "python with Python312 first on PATH (every other step uses this PATH)",
                 ["python", "--version"], "python --version"),
            step("E08", "where-python-pinned", "Where python resolves on the pinned PATH",
                 ["where.exe", "python"], "where.exe python"),
            step("E09", "python-default", "python on the machine's default PATH (disclosure: not the pinned interpreter)",
                 ["python", "--version"], "python --version", path="default"),
            step("E10", "where-python-default", "Where python resolves on the machine's default PATH",
                 ["where.exe", "python"], "where.exe python", path="default"),
            step("E11", "where-python3-pinned", "Where python3 resolves on the pinned PATH (python3 is not pinned)",
                 ["where.exe", "python3"], "where.exe python3"),
            step("E12", "git", "Git", ["git", "--version"], "git --version"),
            step("E13", "gh-auth", "GitHub CLI auth status (tokens are masked by gh)", ["gh", "auth", "status"],
                 "gh auth status"),
            step("E14", "head", "Clone HEAD", ["git", "-C", CLONE, "rev-parse", "HEAD"], "git rev-parse HEAD"),
            step("E15", "log", "Recent history", ["git", "-C", CLONE, "log", "--oneline", "-6"], "git log --oneline -6"),
            step("E16", "status", "Worktree status (must be clean)",
                 ["git", "-C", CLONE, "status", "--short", "--branch"], "git status --short --branch"),
            step("E17", "autocrlf", "core.autocrlf at every config level",
                 ["git", "-C", CLONE, "config", "--show-origin", "--get-all", "core.autocrlf"],
                 "git config --show-origin --get-all core.autocrlf"),
            step("E18", "identity", "Commit identity in this clone",
                 ["git", "-C", CLONE, "config", "--local", "--get-regexp", "^user\\."],
                 "git config --local --get-regexp ^user\\."),
            step("E19", "check-attr-prelude", "Checkout attributes for the seal-sensitive prelude",
                 ["git", "-C", CLONE, "check-attr", "text", "eol", "--", "garnet-interp-v0.3/src/prelude.rs"],
                 "git check-attr text eol -- garnet-interp-v0.3/src/prelude.rs"),
            ps_step("E20", "processes", "Process count and OneDrive processes",
                    "$p = Get-Process\n\"process_count=$($p.Count)\"\n"
                    "$p | Where-Object { $_.ProcessName -like '*OneDrive*' } | Sort-Object Id | "
                    "ForEach-Object { \"onedrive_process=$($_.ProcessName) pid=$($_.Id)\" }\n"),
            ps_step("E21", "env-vars", "Toolchain-relevant environment variables inherited by every step",
                    "Get-ChildItem env: | Where-Object { $_.Name -match '^(GARNET|CARGO|RUSTUP|RUSTC|RUSTFLAGS|RUST_|PYTHON|GIT_|GH_)' } | "
                    "Sort-Object Name | ForEach-Object { \"$($_.Name)=$($_.Value)\" }\n"
                    "\"(end of GARNET/CARGO/RUST/PYTHON/GIT/GH variables)\"\n"),
            ps_step("E22", "wsl", "WSL distributions (context only; nothing in this bundle runs in WSL)",
                    "wsl.exe -l -v | Out-String | ForEach-Object { $_ -replace \"`0\", '' }\n"),
            ps_step("E23", "ps-host", "PowerShell host used by every PowerShell step (powershell.exe)",
                    "$PSVersionTable.GetEnumerator() | Sort-Object Name | ForEach-Object { \"$($_.Name)=$($_.Value -join ',')\" }\n"),
        ]
    if phase == "installers":
        return [
            ps_step("L00", "user-path-before", "User PATH fingerprint before any installer run", USER_PATH_PROBE,
                    section="live"),
            step("L01", "fetch-live-installer", "Companion fetch of the live installer bytes, for hashing (not executed)",
                 curl(LIVE_INSTALLER_URL, live), f"curl.exe -sS -L --proto =https -o streams/L01-install.ps1.live {LIVE_INSTALLER_URL}",
                 gate=True, section="live", fetched=[live]),
            step("L02", "compare-live-main", "Gate: live installer bytes must equal docs/install.ps1 at main before any run",
                 pyc(compare_code(live)), "python -I -c <sha256 live vs main; exit 1 unless identical>", gate=True,
                 section="live"),
            step("L03", "live-blob", "git blob id of the fetched live installer bytes",
                 ["git", "-C", CLONE, "hash-object", "--no-filters", live],
                 "git hash-object --no-filters streams/L01-install.ps1.live", section="live"),
            step("L04", "main-blob", "git blob id of docs/install.ps1 at main",
                 ["git", "-C", CLONE, "rev-parse", "HEAD:docs/install.ps1"], "git rev-parse HEAD:docs/install.ps1",
                 section="live"),
            ps_step("L05", "installer-live", "Live installer: the README.md:136 one-liner with a throwaway prefix and GARNET_NO_MODIFY_PATH=1, then --version", LIVE_RUN, section="live"),
            ps_step("L06", "user-path-after-live", "User PATH fingerprint after the live installer", USER_PATH_PROBE,
                    section="live"),
            step("L07", "fetch-sums", "Companion fetch of the v0.8.2 SHA256SUMS", curl(RELEASE + "/SHA256SUMS", sums),
                 f"curl.exe -sS -L --proto =https -o streams/L07-SHA256SUMS.v0.8.2 {RELEASE}/SHA256SUMS", section="live",
                 fetched=[sums]),
            step("L08", "fetch-zip", "Companion fetch of the v0.8.2 Windows zip (kept outside the bundle)",
                 curl(RELEASE + "/garnet-0.8.2-x86_64-pc-windows-msvc.zip", SCRATCH_ZIP),
                 f"curl.exe -sS -L --proto =https -o <scratch>/garnet-0.8.2-x86_64-pc-windows-msvc.zip {RELEASE}/garnet-0.8.2-x86_64-pc-windows-msvc.zip",
                 section="live", fetched=[SCRATCH_ZIP]),
            step("L09", "zip-verify", "Zip sha256 vs its SHA256SUMS line; zip entries; sha256 of the zip's garnet.exe",
                 pyc(zip_code(sums)), "python -I -c <hash zip, compare SHA256SUMS, list entries>", section="live"),
            ps_step("M01", "installer-main", "docs/install.ps1 from main, same steps (happy path; own prefix)",
                    main_run("garnet-nuc-install-test-main"), section="main"),
            ps_step("M02", "empty-release-dir", "The file:/// release dir used to make SHA256SUMS unreachable",
                    f"Get-ChildItem -Force -LiteralPath '{EMPTY_RELEASE_DIR}' | Measure-Object | "
                    "ForEach-Object { \"entries=$($_.Count)\" }\n", section="main"),
            ps_step("M03", "installer-main-hint", "docs/install.ps1 from main with no reachable release (SHA256SUMS unreachable)",
                    main_run("garnet-nuc-install-test-hint",
                             '$env:GARNET_BASE_URL = "file:///C:/gw1-stage/empty-release-dir"\n', version_line=False),
                    section="main", expect_exit=1),
            step("M04", "hint-decode", "Does M03's message contain --tag v0.8.2? (decoded view of its error text)",
                 pyc(decode_code(streams, "M03-installer-main-hint", "--tag v0.8.2")),
                 "python -I -c <decode M03 CLIXML error text; exit 1 unless it contains --tag v0.8.2>", section="main"),
            ps_step("M05", "installer-main-v081",
                    "docs/install.ps1 from main for v0.8.1, a published release with no Windows asset",
                    main_run("garnet-nuc-install-test-v081", '$env:GARNET_VERSION = "0.8.1"\n', version_line=False),
                    section="main", expect_exit=1),
            step("M06", "v081-decode", "Decoded view of M05's error text",
                 pyc(decode_code(streams, "M05-installer-main-v081", "lists no garnet-0.8.1-x86_64-pc-windows-msvc.zip")),
                 "python -I -c <decode M05 CLIXML error text>", section="main"),
            ps_step("M07", "installer-main-mode-source",
                    "docs/install.ps1 from main with GARNET_INSTALL_MODE=source (the variable FAQ.md:91 names)",
                    main_run("garnet-nuc-install-test-mode-source", '$env:GARNET_INSTALL_MODE = "source"\n'),
                    section="main"),
            ps_step("M08", "installed-exes", "Every throwaway prefix: installed garnet.exe sha256 and signature status",
                    EXE_PROBE, section="main"),
            ps_step("M09", "user-path-after", "User PATH fingerprint after all installer runs", USER_PATH_PROBE,
                    section="main"),
        ]
    if phase == "build":
        return [step("B01", "cargo-build", "Build garnet-cli", ["cargo", "build", "--locked", "-p", "garnet-cli"],
                     "cargo build --locked -p garnet-cli", timeout=5400, section="build")]
    if phase == "test":
        return [step("T01", "cargo-test", "Workspace tests (stdout+stderr merged into cargo-test.log)",
                     ["cargo", "test", "--locked", "--workspace", "--no-fail-fast"],
                     "cargo test --locked --workspace --no-fail-fast", merged_file="cargo-test.log",
                     timeout=4 * 3600, section="test")]
    if phase == "post":
        return [
            step("P01", "status-after", "Worktree status after every step", ["git", "-C", CLONE, "status", "--short",
                 "--branch"], "git status --short --branch", section="post"),
            step("P02", "remote-main-after", "Remote readback of main after every step (no credential)",
                 ["git", "-C", CLONE, "-c", "credential.helper=", "ls-remote", "origin", "refs/heads/main"],
                 "git -c credential.helper= ls-remote origin refs/heads/main", env=NO_PROMPT, section="post"),
            ps_step("P03", "processes-after", "Process count after every step", "\"process_count=$((Get-Process).Count)\"\n",
                    section="post"),
        ]
    raise SystemExit(f"unknown phase {phase}")


PHASES = ("clone", "env", "installers", "build", "test", "post")


def kill_tree(pid: int) -> None:
    subprocess.run(["taskkill", "/T", "/F", "/PID", str(pid)], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)


def run_step(bundle: Path, s: dict, phase: str) -> dict:
    env = dict(os.environ)
    env["PATH"] = PINNED_PATH if s["path"] == "pinned" else DEFAULT_PATH
    env.update(s["env"])
    exe = shutil.which(s["argv"][0], path=env["PATH"]) or s["argv"][0]
    argv = [exe] + s["argv"][1:]
    (bundle / "streams").mkdir(parents=True, exist_ok=True)
    if s["merged_file"]:
        outs = [(bundle / s["merged_file"], "stdout+stderr")]
    else:
        outs = [(bundle / "streams" / f"{s['id']}-{s['stem']}.stdout", "stdout"),
                (bundle / "streams" / f"{s['id']}-{s['stem']}.stderr", "stderr")]
    started, t0, timed_out = utcnow(), time.monotonic(), False
    handles = [open(p, "wb") for p, _ in outs]
    try:
        proc = subprocess.Popen(argv, cwd=s["cwd"], env=env, stdin=subprocess.DEVNULL, stdout=handles[0],
                                stderr=subprocess.STDOUT if s["merged_file"] else handles[1])
        try:
            rc = proc.wait(timeout=s["timeout"])
        except subprocess.TimeoutExpired:
            timed_out = True
            kill_tree(proc.pid)
            rc = proc.wait()
    finally:
        for h in handles:
            h.close()
    rec = {k: s[k] for k in ("id", "stem", "title", "display", "cwd", "section", "ps_script", "expect_exit")}
    rec.update(phase=phase, argv=argv, resolved_exe=exe, path_mode=s["path"], env_overrides=s["env"],
               merged=bool(s["merged_file"]), started_utc=started, ended_utc=utcnow(),
               duration_s=round(time.monotonic() - t0, 3), exit_code=rc, timed_out=timed_out, timeout_s=s["timeout"],
               harness_sha256=HARNESS_SHA256,
               fetched=[{"file": Path(f).relative_to(bundle).as_posix() if Path(f).is_relative_to(bundle) else f,
                         "in_bundle": Path(f).is_relative_to(bundle), "bytes": Path(f).stat().st_size,
                         "sha256": sha256_file(Path(f))} for f in s["fetched"] if Path(f).is_file()],
               streams=[{"file": p.relative_to(bundle).as_posix(), "kind": k, "bytes": p.stat().st_size,
                         "sha256": sha256_file(p)} for p, k in outs])
    return rec


def cmd_all(bundle: Path, log: Path) -> int:
    if log.exists() or (bundle.exists() and any(bundle.iterdir())):
        raise SystemExit(f"refusing to mix runs: {log} or {bundle} already has content")
    bundle.mkdir(parents=True, exist_ok=True)
    run = {"harness_sha256": HARNESS_SHA256, "python": sys.executable, "python_version": sys.version,
           "argv": sys.argv, "started_utc": utcnow(), "clone": CLONE, "stage": str(bundle), "host": HOST,
           "phases": list(PHASES), "stopped_at": None, "unexpected_exits": []}
    for phase in PHASES:
        for s in steps_for(phase, bundle):
            rec = run_step(bundle, s, phase)
            with open(log, "a", encoding="utf-8", newline="\n") as f:
                f.write(json.dumps(rec, ensure_ascii=False) + "\n")
            flag = "" if rec["exit_code"] == s["expect_exit"] else "  <-- UNEXPECTED EXIT"
            print(f"{rec['id']} {rec['stem']}: exit={rec['exit_code']} (expected {s['expect_exit']}) "
                  f"{rec['duration_s']}s{flag}", flush=True)
            if flag:
                run["unexpected_exits"].append(rec["id"])
                if s["gate"]:
                    run["stopped_at"] = rec["id"]
                    break
        if run["stopped_at"]:
            print(f"GATE FAILED at {run['stopped_at']}; stopping", flush=True)
            break
    run["ended_utc"] = utcnow()
    (STAGE_ROOT / f"run-{bundle.name}.json").write_text(json.dumps(run, indent=2, sort_keys=True) + "\n",
                                                        encoding="utf-8", newline="\n")
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
    if not data:
        endings = "empty"
    elif not (crlf or lf or cr):
        endings = "no-newline"
    elif crlf and not lf and not cr:
        endings = "CRLF"
    elif lf and not crlf and not cr:
        endings = "LF"
    else:
        endings = "MIXED"
    return dict(path=rel, bytes=len(data), sha256=hashlib.sha256(data).hexdigest(), utf8_valid=utf8, bom=bom,
                endings=endings, lf=lf, crlf=crlf, lone_cr=cr, non_ascii=sum(1 for b in data if b > 127),
                nul=data.count(0), final_newline=data.endswith(b"\n"))


def section_doc(title: str, recs: list[dict], bundle: Path, preamble: str) -> bytes:
    out = bytearray(f"# {title}\n{preamble}".encode())
    for r in recs:
        out += b"\n" + b"=" * 78 + b"\n"
        out += f"[{r['id']}] {r['title']}\n".encode()
        shown = r["display"].rstrip("\n").replace("\n", "\n  ")
        out += f"$ {shown}\n".encode()
        out += (f"exit_code={r['exit_code']} expected={r['expect_exit']} timed_out={r['timed_out']} "
                f"started={r['started_utc']} duration_s={r['duration_s']} path={r['path_mode']}\n").encode()
        for st in r["streams"]:
            if not st["file"].startswith("streams/"):
                out += (f"--- {st['kind']}: not repeated here; see {st['file']} ({st['bytes']} bytes, "
                        f"sha256 {st['sha256']})\n").encode()
                continue
            raw = (bundle / st["file"]).read_bytes()
            out += f"--- {st['kind']} ({st['file']}, {st['bytes']} bytes, sha256 {st['sha256']})\n".encode()
            out += raw
            if raw and not raw.endswith(b"\n"):
                out += b"\n[no trailing newline in the captured stream]\n"
            out += f"--- end {st['kind']}\n".encode()
    return bytes(out)


TEST_RESULT = re.compile(rb"^test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; "
                         rb"(\d+) filtered out")
RUNNING = re.compile(rb"^\s*(Running|Doc-tests) (.+?)\s*$")


def summarize_tests(log: bytes) -> bytes:
    lines = log.splitlines()
    current, targets, failures, in_fail_list, ignored = None, [], [], False, []
    for raw in lines:
        m = RUNNING.match(raw)
        if m:
            current = (m.group(1) + b" " + m.group(2)).decode("utf-8", "backslashreplace")
            in_fail_list = False
            continue
        m = TEST_RESULT.match(raw)
        if m:
            targets.append((current, m.group(1).decode(), *[int(x) for x in m.groups()[1:]]))
            in_fail_list = False
            continue
        if raw.startswith(b"test ") and raw.rstrip().endswith(b" ... ignored"):
            ignored.append((current, raw.decode("utf-8", "backslashreplace").strip()))
        if raw.rstrip() == b"failures:":
            in_fail_list = True
            continue
        if in_fail_list:
            s = raw.strip()
            if raw.startswith(b"    ") and s and not s.startswith(b"----"):
                failures.append((current, s.decode("utf-8", "backslashreplace")))
            elif s and not raw.startswith(b"    "):
                in_fail_list = False
    tail = [l.decode("utf-8", "backslashreplace") for l in lines
            if l.startswith(b"error:") or l.startswith(b"    `") or b"could not compile" in l]
    tot = [sum(t[i] for t in targets) for i in range(2, 7)]
    out = ["# cargo test summary (derived from cargo-test.log by nuc_capture.py; the log is authoritative)", ""]
    out.append(f"targets_started={sum(1 for l in lines if RUNNING.match(l))} targets_with_result_line={len(targets)} "
               f"failed_targets={sum(1 for t in targets if t[1] == 'FAILED')}")
    out.append(f"totals: passed={tot[0]} failed={tot[1]} ignored={tot[2]} measured={tot[3]} filtered_out={tot[4]}")
    out += ["", "## Failing tests by name (target, then test)"]
    out += [f"- {t} :: {n}" for t, n in sorted(set(failures))] or ["- none"]
    out += ["", "## Ignored tests (target, then libtest line)"]
    out += [f"- {t} :: {n}" for t, n in ignored] or ["- none"]
    out += ["", "## cargo error / failed-target lines (verbatim)"]
    out += tail or ["- none"]
    out += ["", "## Per-target result lines"]
    out += [f"- [{t[1]}] passed={t[2]} failed={t[3]} ignored={t[4]} measured={t[5]} filtered={t[6]} :: {t[0]}"
            for t in targets]
    return ("\n".join(out) + "\n").encode("utf-8")


def cmd_render(bundle: Path, log: Path, stamp: str) -> None:
    recs = [json.loads(l) for l in log.read_text(encoding="utf-8").splitlines() if l.strip()]
    run = json.loads((STAGE_ROOT / f"run-{bundle.name}.json").read_text(encoding="utf-8"))
    head = (bundle / "streams" / "E14-head.stdout").read_text(encoding="ascii").strip()
    by: dict[str, list] = {}
    for r in recs:
        by.setdefault(r["section"], []).append(r)
    pre = (f"bundle: proofs/windows/nuc-baseline/{bundle.name}/\n"
           f"host: {HOST} (%COMPUTERNAME%; `hostname` prints the DNS casing, see E01)\n"
           f"main: {head}\nstamp: {stamp} (UTC)\n"
           f"harness: harness/nuc_capture.py, sha256 {run['harness_sha256']}; every step below ran from this one\n"
           f"  invocation ({' '.join(Path(a).name if i == 0 else a for i, a in enumerate(run['argv']))}), "
           f"{run['started_utc']} to {run['ended_utc']}.\n"
           "python: every step except E09/E10 ran with Python312 first on PATH. E09/E10 deliberately use the\n"
           "  machine's default PATH, where python resolves to an unrelated 3.11.15 virtualenv. python3 is not\n"
           "  pinned; E11 shows where it resolves.\n"
           "powershell: PowerShell steps run powershell.exe (Windows PowerShell 5.1, see E23) with -NoProfile\n"
           "  -NonInteractive -ExecutionPolicy Bypass -EncodedCommand. With its output redirected, powershell.exe\n"
           "  writes error, progress and information records to stderr as CLIXML ('#< CLIXML'); M04 and M06 print\n"
           "  decoded views of the error text. The raw streams are authoritative.\n"
           "fetches: L01, L07 and L08 are companion fetches made next to the installer runs; they are not the\n"
           "  bytes the installers themselves downloaded.\n"
           "captures: stdout and stderr are the raw child-process bytes, unedited; see encoding-census.txt.\n")
    (bundle / "env.txt").write_bytes(section_doc("Clone and environment captures", by.get("env", []), bundle, pre))
    (bundle / "installer-live.txt").write_bytes(section_doc(
        "Installer, live (irm https://garnet-lang.org/install.ps1 | iex)", by.get("live", []), bundle, pre))
    (bundle / "installer-main.txt").write_bytes(section_doc(
        "Installer, from main (docs/install.ps1 at main)", by.get("main", []), bundle, pre))
    (bundle / "build-steps.txt").write_bytes(section_doc(
        "Build, test and post-run steps (the test log itself is cargo-test.log)",
        by.get("build", []) + by.get("test", []) + by.get("post", []), bundle, pre))
    lines = [f"# Commands run for proofs/windows/nuc-baseline/{bundle.name}/", pre.rstrip("\n"),
             "Each step: child process from argv, no shell, stdin closed. PowerShell steps show the script text",
             "exactly as PowerShell received it (the argv carries it base64-encoded).",
             f"run: stopped_at={run['stopped_at']} unexpected_exits={run['unexpected_exits']}", ""]
    for r in recs:
        lines.append("=" * 78)
        lines.append(f"[{r['id']}] {r['title']}   (phase {r['phase']})")
        if r["ps_script"]:
            lines.append("command (PowerShell script):")
            lines += ["    " + l for l in r["ps_script"].rstrip("\n").split("\n")]
            argv_shown = r["argv"][:-1] + ["<base64 UTF-16LE of the script above>"]
        else:
            lines.append(f"command: {r['display'].rstrip()}")
            argv_shown = r["argv"]
        lines.append("argv: " + json.dumps(argv_shown, ensure_ascii=False))
        lines.append(f"resolved_exe: {r['resolved_exe']}")
        lines.append(f"cwd: {r['cwd']}   path: {r['path_mode']}   env_overrides: {json.dumps(r['env_overrides'])}")
        lines.append(f"started_utc: {r['started_utc']}   ended_utc: {r['ended_utc']}   duration_s: {r['duration_s']}")
        lines.append(f"exit_code: {r['exit_code']}   expected: {r['expect_exit']}   timed_out: {r['timed_out']}   "
                     f"timeout_s: {r['timeout_s']}")
        for st in r["streams"]:
            lines.append(f"stream: {st['kind']:<13} {st['sha256']}  {st['bytes']:>9} B  {st['file']}")
        for fe in r.get("fetched", []):
            where = "" if fe["in_bundle"] else "  (kept outside the bundle)"
            lines.append(f"fetched: {'curl -o':<12} {fe['sha256']}  {fe['bytes']:>9} B  {fe['file']}{where}")
    (bundle / "commands.txt").write_bytes(("\n".join(lines) + "\n").encode("utf-8"))
    (bundle / "commands.json").write_bytes(
        (json.dumps({"run": run, "steps": recs}, ensure_ascii=False, indent=2, sort_keys=True) + "\n").encode())
    if (bundle / "cargo-test.log").exists():
        (bundle / "test-summary.txt").write_bytes(summarize_tests((bundle / "cargo-test.log").read_bytes()))
    rels = sorted(p.relative_to(bundle).as_posix() for p in bundle.rglob("*")
                  if p.is_file() and p.name not in ("MANIFEST.sha256", "encoding-census.txt"))
    rows = [census_row(bundle / rel, rel) for rel in rels]
    hdr = (f"{'path':<52} {'bytes':>9} {'utf8':>5} {'bom':>8} {'endings':>10} {'lf':>7} {'crlf':>6} {'cr':>4} "
           f"{'nonascii':>8} {'nul':>4} {'finalnl':>7}  sha256")
    cl = ["# Encoding census (UTF-8 validity, BOM, line endings) for every file in this bundle, sorted bytewise",
          "# excludes this file and MANIFEST.sha256 (both written by the harness as UTF-8, no BOM, LF).",
          "# lf = bare LF count, crlf = CRLF pairs, cr = lone CR; endings = LF | CRLF | MIXED | no-newline | empty.", "",
          hdr]
    for r in rows:
        cl.append(f"{r['path']:<52} {r['bytes']:>9} {str(r['utf8_valid']):>5} {r['bom']:>8} {r['endings']:>10} "
                  f"{r['lf']:>7} {r['crlf']:>6} {r['lone_cr']:>4} {r['non_ascii']:>8} {r['nul']:>4} "
                  f"{str(r['final_newline']):>7}  {r['sha256']}")
    (bundle / "encoding-census.txt").write_bytes(("\n".join(cl) + "\n").encode("utf-8"))
    allf = sorted(p.relative_to(bundle).as_posix() for p in bundle.rglob("*")
                  if p.is_file() and p.name != "MANIFEST.sha256")
    (bundle / "MANIFEST.sha256").write_bytes("".join(f"{sha256_file(bundle / rel)}  {rel}\n" for rel in allf)
                                             .encode("ascii"))
    print(f"rendered {len(allf)} files + MANIFEST.sha256 in {bundle}")


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("mode", choices=("all", "render"))
    ap.add_argument("--stamp", required=True)
    a = ap.parse_args()
    bundle = STAGE_ROOT / f"{a.stamp}-{HOST}"
    log = STAGE_ROOT / f"steps-{a.stamp}.jsonl"
    if a.mode == "render":
        cmd_render(bundle, log, a.stamp)
        return 0
    return cmd_all(bundle, log)


if __name__ == "__main__":
    sys.exit(main())
