# Windows MSI (wix)

This directory holds the wix/cargo-wix configuration for Garnet's Windows
installer.

## Build

Prerequisites:

1. [WiX Toolset 3.11+](https://wixtoolset.org/) on PATH (`candle.exe`,
   `light.exe`).
2. [`cargo-wix`](https://github.com/volks73/cargo-wix) (`cargo install cargo-wix`).
3. The Garnet monorepo checked out at a tagged release.

```powershell
# One-time: ensure the MSVC target is installed (not MinGW).
rustup target add x86_64-pc-windows-msvc

# Build the release binary + MSI. Output lands at
#   target\wix\garnet-<version>-x86_64.msi
cd garnet-cli
cargo build --release --target x86_64-pc-windows-msvc
cargo wix --nocapture
```

## Sign

Sign the produced MSI with the user's Authenticode cert. Timestamping is
required so the signature remains trusted after the cert expires.

```powershell
signtool sign ^
  /f C:\path\to\codesign.pfx ^
  /p <password> ^
  /fd SHA256 ^
  /tr http://timestamp.digicert.com ^
  /td SHA256 ^
  target\wix\garnet-0.4.2-x86_64.msi
```

Verify the signature:

```powershell
signtool verify /pa /v target\wix\garnet-0.4.2-x86_64.msi
```

## Test

Install on a clean Windows 11 26H1 VM. The MSI:

- Installs to `C:\Program Files\Garnet\`
- Adds `C:\Program Files\Garnet\bin` to the system PATH (HKLM)
- Creates Start Menu shortcut "Garnet Shell"
- Runs `garnet --version` once post-install as a smoke test
- Registers an Add/Remove Programs entry linking to `https://garnet-lang.org`

On uninstall: binary, LICENSE.txt, PATH entry, Start Menu shortcut, and
registry values are all removed.

## Assets to supply

The following placeholder paths are referenced in `main.wxs` and must
resolve to real files at build time:

| Path | Purpose |
|------|---------|
| `wix\License.rtf`  | RTF-formatted license shown during the WiX UI license-accept step. |
| `wix\Banner.bmp`   | 493×58 banner at the top of wizard pages. |
| `wix\Dialog.bmp`   | 493×312 dialog background for welcome/exit pages. |
| `wix\Garnet.ico`   | Multi-resolution icon (16/32/48/256 px). Also appears in ARP. |
| `LICENSE.txt`      | Plain-text license dropped into the install root. |

These are assets the user provides; the build step fails loudly if any
is missing (WiX: `LGHT0103 : file not found`).
