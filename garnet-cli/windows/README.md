# Windows Sandbox smoke-test harness

This directory holds two files that together give you a **one-double-click** smoke test for the MSI after you sign it.

## Prerequisites

- **Windows 11 Pro / Enterprise** (Windows Sandbox is not available on Home)
- **Windows Sandbox feature enabled** — one-time: `Control Panel > Programs > Turn Windows features on or off > Windows Sandbox`, tick it, reboot
- A **signed MSI** at `dist/windows/garnet-0.4.2-x86_64.msi` (unsigned MSIs will trigger a SmartScreen warning; the smoke still completes, but Gatekeeper-style friction is part of what you're testing)

## Usage

1. **Adjust the host path** in `sandbox-smoke.wsb` if your clone isn't at the default location. Edit this line:

   ```xml
   <HostFolder>D:\Projects\New folder\Garnet (1)\GARNET\Garnet\Opus-Gpt-Xai-Opus-Gemini-Opus\Garnet_Final\E_Engineering_Artifacts</HostFolder>
   ```

   Point it at your clone's `E_Engineering_Artifacts/` directory. Leave `<SandboxFolder>` alone — it's the Sandbox-side path.

2. **Double-click `sandbox-smoke.wsb`**.

   Windows Sandbox launches. The workspace auto-mounts read-only inside the Sandbox. The LogonCommand runs `smoke-test.cmd` in a fresh cmd window.

3. **Observe the 8 gates**:

   | Gate | What's verified |
   |---|---|
   | 1 | `msiexec /i garnet-0.4.2-x86_64.msi /qn` silent install completes |
   | 2 | `garnet --version` renders the wordmark + version table |
   | 3 | `garnet --help` shows all 13 subcommands |
   | 4 | `garnet new --template cli C:\test-project` scaffolds 5 files |
   | 5 | `garnet test C:\test-project` passes 2 starter tests |
   | 6 | `garnet keygen C:\test.key` generates Ed25519 keypair |
   | 7 | `garnet build --deterministic --sign` produces signed manifest |
   | 8 | `garnet verify --signature` confirms cryptographic round-trip |

   Expected final line: `ALL 8 GATES PASSED - Phase 6D Windows fully verified`.

4. **Close the Sandbox window** when done. The entire ephemeral VM — install log, generated keys, test project, everything — is discarded. No residue on your host machine.

## If a gate fails

Most likely cause: the MSI isn't where the script expects, or the signed MSI's cert isn't trusted in the Sandbox (Sandbox starts with a fresh Windows cert store; your publisher cert chain should chain to a root that Windows trusts — most Authenticode CAs do by default).

Logs:

- MSI install: `%TEMP%\garnet-install.log` inside the Sandbox
- Everything else: inline in the cmd window; re-run manually with `set PATH=%PATH%;C:\Program Files\Garnet\bin` then `garnet <subcommand>` to isolate the failing step

## Without Windows Sandbox

If you're on Windows 11 Home or don't want the Sandbox dance, `smoke-test.cmd` works standalone — just run it from an admin cmd on your host. It'll install Garnet natively on your host machine, run the tests, and leave Garnet installed (you can uninstall via Add/Remove Programs afterward).

## What's NOT covered here

- **Uninstall cleanliness**: the script doesn't test that Add/Remove Programs removes everything correctly. Do that manually: after the 8 gates pass, run `Control Panel > Programs > Uninstall`, pick Garnet, let it uninstall, then check `C:\Program Files\Garnet\` is gone + the HKLM PATH entry is cleaned + the Start Menu shortcut is removed. A v4.2.1 iteration can script this via a second `.wsb` that invokes `msiexec /x`.
- **Upgrade path**: a future v4.3 MSI should install over this one via the stable UpgradeCode. Validate once v4.3 is cut.
- **Signed-release fetch from `sh.garnet-lang.org`**: orthogonal to this local smoke — covered by the universal installer's `SHA256SUMS` check.

## Files in this directory

| File | Purpose |
|------|---------|
| `sandbox-smoke.wsb` | Windows Sandbox config: mounts workspace + runs smoke-test.cmd |
| `smoke-test.cmd` | The 8-gate smoke test; installs MSI + exercises every v4.2 feature |
| `README.md` | This file |
