# Garnet Windows Studio Clean-VM Smoke Runbook

Date: 2026-05-21
Scope: Windows-side Garnet Studio installer/runtime proof after PR #217.

## Current Truth

- `apps/garnet-studio` is the Windows/Linux Tauri v2 shell.
- Windows source proof exists for frontend build, Rust backend tests, release
  executable, unsigned NSIS bundle, and `--studio-smoke`.
- Clean-machine Windows installer completion is still open until a fresh VM
  installs the unsigned NSIS artifact, launches the installed app, and preserves
  logs/screenshots.
- Signed MSI, Authenticode, winget, Linux desktop package proof, and macOS
  notarization remain separate gates.

## Target Order

1. Windows x64: first clean-VM installer proof target
   (`x86_64-pc-windows-msvc`).
2. Windows ARM64: next target after x64 proof
   (`aarch64-pc-windows-msvc`), requiring its own target install/build/smoke.
3. Windows 32-bit: deferred until demand justifies the extra WebView2 and
   installer QA surface (`i686-pc-windows-msvc`).

Linux Studio package format should remain open until the shell launches in a
real Linux desktop session. The macOS SwiftUI Studio remains the native Apple
reference app.

## Required Clean-VM Evidence

Capture these from the VM and record them with
`scripts/garnet_windows_clean_vm_installer_status.py`:

- unsigned NSIS setup executable path and SHA-256
- VM name, guest OS, and guest architecture
- installer run log or transcript from inside the guest
- installed Studio `--studio-smoke` JSON with `status=passed`
- installed app launch screenshot
- preserved claim boundary: unsigned NSIS proof is not signed MSI, winget, or
  Linux package proof

## Recording Command

```powershell
$out = "$env:USERPROFILE\Desktop\dogfood\garnet-studio-windows-clean-vm\garnet-studio-windows-clean-vm-$(Get-Date -Format yyyyMMdd-HHmmss)"
python scripts\garnet_windows_clean_vm_installer_status.py `
  --record-proof `
  --mode clean-vm `
  --vm-name "garnet-win11-clean" `
  --guest-os "Windows 11" `
  --guest-arch "x64" `
  --installer "C:\path\to\Garnet Studio_*_x64-setup.exe" `
  --install-log "C:\path\to\install.log" `
  --studio-smoke-json "C:\path\to\studio-smoke.json" `
  --screenshot "C:\path\to\launch.png" `
  --output-dir $out
```

Then run:

```powershell
python scripts\garnet_windows_clean_vm_installer_status.py --format markdown
python scripts\garnet_windows_linux_studio_status.py --format markdown
python scripts\garnet_mit_readiness_status.py --format markdown
```

## Claim Boundary

Do not update README, website, or release notes to say Windows installer
completion until the clean-VM bundle exists and verifies. Do not say signed MSI
or winget until the certificate, signed artifact, public release asset, and
clean-machine install evidence all exist.
