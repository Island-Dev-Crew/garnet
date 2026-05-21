# Garnet Studio (Windows/Linux)

This is the first Tauri v2 shell for the Windows/Linux Garnet Studio MVP. It is
a thin desktop wrapper around the existing Garnet CLI, converter advisory
scripts, docs/PWA truth, and dogfood readiness matrix.

## Current Truth

- Active conversion is source-to-Garnet for Rust, Ruby, Python, and Go only.
- Advisory planning is separate: JavaScript, TypeScript, Swift, Java, C, C++,
  C#, Perl, Kotlin, Shell, SQL, and Other.
- The shell does not call provider APIs, does not include source in provider
  packets by default, and does not mark advisory output safe.
- macOS SwiftUI Studio remains the native Apple reference app.
- Windows source-build proof exists for the release executable, unsigned NSIS
  bundle, and `--studio-smoke` evidence. Signed MSI, winget, Linux runtime, and
  clean-machine installer proof are still separate gates.
- The Release / Readiness panel calls the same repo-native reporters used by the
  macOS reference surface: Windows/Linux Studio status, converter fit/provider
  options, MIT objective/demo/deck evidence, Mac continuation boundaries, proof
  and benchmark status, benchmark no-run compile evidence, and notarization
  preflight status.

## Local Verification

From `apps/garnet-studio`:

```sh
npm run build
npm run tauri build
```

From the repository root:

```sh
cargo test --manifest-path apps/garnet-studio/src-tauri/Cargo.toml
target/release/garnet-studio --studio-smoke
python scripts/test_garnet_windows_linux_studio_shell.py
python scripts/test_garnet_windows_linux_studio_status.py
```

On Windows, the smoke bundle is written under:

```text
%USERPROFILE%\Desktop\dogfood\garnet-studio-windows-linux
```
