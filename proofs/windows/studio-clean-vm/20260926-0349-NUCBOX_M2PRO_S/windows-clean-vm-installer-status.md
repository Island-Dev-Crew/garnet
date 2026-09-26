# Garnet Windows Studio Clean-VM Installer Status

Source: `C:\garnet-w2`
Status: `clean-vm-proof-verified`
Default evidence root: `proofs\windows\studio-clean-vm`
Clean VM verified: `true`
Proof source: `local:proofs\windows\studio-clean-vm\20260926-0349-NUCBOX_M2PRO_S`

## Current Truth

- Windows x64 is the first Studio installer proof target because current Tauri NSIS evidence is x64-local.
- Windows ARM64 is a reasonable follow-up target, but it needs its own Rust/MSVC target install, build, and clean-machine smoke.
- Windows 32-bit remains deferred until user demand justifies separate WebView2 and installer QA.
- Linux Studio package format remains open until a Linux desktop launch proves the shell runtime.
- macOS Studio remains the separate SwiftUI Apple reference lane, not a Tauri port claim.
- This script records or reports installer proof; it does not run installers on the current host.

## Package Target Posture

| Target | Platform | Architecture | Rust target | Surface | Status |
| --- | --- | --- | --- | --- | --- |
| studio-windows-x64-nsis | Windows | x64 | `x86_64-pc-windows-msvc` | Tauri NSIS setup executable | `first-clean-vm-target` |
| studio-windows-arm64-nsis | Windows | ARM64 | `aarch64-pc-windows-msvc` | Tauri NSIS setup executable | `planned-after-x64-proof` |
| studio-windows-x86-nsis | Windows | 32-bit x86 | `i686-pc-windows-msvc` | Tauri NSIS setup executable | `deferred-until-user-demand` |
| studio-linux-x64 | Linux | x64 | `x86_64-unknown-linux-gnu` | AppImage, .deb, or .rpm decision pending | `runtime-open` |
| studio-linux-arm64 | Linux | ARM64 | `aarch64-unknown-linux-gnu` | source/PWA shell first; package later | `planned-after-x64-linux-proof` |
| studio-macos-reference | macOS | Apple Silicon and Intel | `aarch64-apple-darwin / x86_64-apple-darwin` | SwiftUI Studio reference app, not the Windows/Linux Tauri shell | `separate-apple-lane` |

## Required Gates

| Gate | Status | Evidence |
| --- | --- | --- |
| Installer Artifact | `pass` | target\release\bundle\nsis\Garnet Studio_0.8.2_x64-setup.exe |
| Fresh Guest | `pass` | mode=clean-vm; vm=Windows Sandbox; os=Microsoft Windows 11 Enterprise 10.0.26100; arch=x64 |
| Install Log | `pass` | proofs\windows\studio-clean-vm\20260926-0349-NUCBOX_M2PRO_S\install.log |
| Studio Smoke | `pass` | proofs\windows\studio-clean-vm\20260926-0349-NUCBOX_M2PRO_S\studio-smoke.json |
| Launch Screenshot | `pass` | proofs\windows\studio-clean-vm\20260926-0349-NUCBOX_M2PRO_S\launch.png |
| Claim Boundary | `pass` | signed MSI, winget, Linux package, and provider-backed conversion remain forbidden claims. |

## Latest Proof

- Mode: `clean-vm`
- VM: `Windows Sandbox`
- Guest: `Microsoft Windows 11 Enterprise 10.0.26100` / `x64`
- Installer SHA-256: `b288930ee8412be3aee3876ab49177d6c73d7d2fe10c2c4d0caa5187cfeb1d8e`

## Blocked By

- None

## Forbidden Claims

- signed Windows MSI is available
- winget install path is verified
- Windows clean-machine proof exists without clean-VM evidence
- Linux Studio package is verified
- provider-backed conversion is active
