# Garnet Windows/Linux Studio MVP Architecture

Status: slice 1 contract and architecture.
Date: 2026-05-17.
Source truth: live `origin/main`, newer than PR #140. PR #140 remains the
historic baseline for the original Studio advisory handoff, not the current tip.

## Objective

Build the first Windows/Linux Garnet Studio as a thin desktop shell around the
verified CLI, docs/PWA, converter advisory pipeline, and dogfood readiness
gates. Do not port SwiftUI to Windows/Linux. Keep the macOS SwiftUI Studio as
the native Apple reference app.

This slice intentionally adds no Tauri/Electron/GUI dependency. It records the
contract a future shell must implement and verifies that contract with
`scripts/garnet_windows_linux_studio_status.py` and its regression tests.

## Shell Strategy

Preferred path after this slice:

1. Tauri v2 shell if dependency review and Windows/Linux builds pass.
2. PWA-first wrapper if Tauri is not accepted or cannot be verified quickly.
3. Simple Rust GUI only if the web/PWA shell path blocks MVP progress.

Rejected for the MVP: SwiftUI port, converter logic duplication, provider API
calls from the shell, Electron-by-default, or any package claim without target
machine evidence.

## Current Action Contract

| Studio action | Current implementation surface | Notes |
| --- | --- | --- |
| CLI Health | `garnet version` | The current CLI has no `health` subcommand; use the version probe unless a real health command is added. |
| Parse | `garnet parse <file.garnet>` | Garnet source only. |
| Check | `garnet check <file.garnet>` | Garnet source only. |
| Run | `garnet run <file.garnet>` | Executes Garnet source; keep distinct from advisory source inspection. |
| Convert | `garnet convert <rust|ruby|python|go> <source> --out <evidence-dir>` | Active conversion only for Rust, Ruby, Python, and Go. |
| Assist Plan | `scripts/garnet_converter_assist_plan.py` | Active or advisory source inspection; does not enable conversion. |
| Advisory Bundle | `scripts/garnet_converter_advisory_bundle.py` | Omits source by default; no provider call. |
| Advisory Review | `scripts/garnet_converter_advisory_review.py` | Refuses unsafe/source-included bundles unless explicitly approved by the existing script. |
| Advisory Handoff | `scripts/garnet_converter_advisory_handoff.py` | Produces provider-neutral handoff context only. |
| Objective Pulse | `scripts/garnet_mit_readiness_status.py` | Current objective status; not a completion claim. |
| Agentic Dogfood Matrix | `scripts/run_agentic_dogfood_matrix.py --copy-to-desktop --strict` | Evidence gate; shell integration must preserve generated Desktop dogfood bundles. |

## Language Taxonomy

| Menu group | Languages |
| --- | --- |
| Active conversion | Rust, Ruby, Python, Go |
| Advisory planning | JavaScript, TypeScript, Swift, Java, C, C++, C#, Perl, Kotlin, Shell, SQL, Other |
| Native boundary recommended | C, C++, Objective-C, Assembly, CUDA, platform-specific code |
| Future backend lowering | Wasm, LLVM-style native targets, native package toolchains |

## Evidence Contract

Default evidence root:

```text
~/Desktop/dogfood/garnet-studio-windows-linux/
```

Each run should write a timestamped bundle named
`garnet-studio-windows-linux-<stamp>` with command stdout/stderr logs,
manifested JSON/Markdown status, and screenshots once the shell exists. Source
is not included by default. Advisory output is never marked safe.

## Packaging Gates

Windows remains open until target evidence exists for:

- MSVC release build of the CLI and future shell.
- MSI plan and clean-machine smoke.
- Authenticode signing and `signtool verify`.
- winget manifest only after a signed public artifact exists.

Linux remains open until target evidence exists for:

- Existing `.deb` and `.rpm` CLI package smoke.
- First Studio package decision. AppImage-first is the simplest candidate, but
  it is not selected until a target smoke can exercise the shell.
- Desktop dogfood bundle capture from the Linux shell.

README and site copy must keep verified source installs separate from future
signed/package distribution claims.

## Verification

Focused slice verification:

```sh
python3 scripts/test_garnet_windows_linux_studio_status.py
python3 scripts/garnet_windows_linux_studio_status.py
python3 scripts/garnet_windows_linux_studio_status.py --output-dir <bundle>
cargo fmt --all -- --check
git diff --check
```

Full PR verification should then include the usual workspace and dogfood gates.

## Open User-Supplied Inputs

- A real Windows machine or Windows VM for runtime launch evidence.
- A Linux GUI VM/container for runtime launch evidence.
- Signing credentials only when the project is ready to verify signed MSI
  claims.
- A final dependency decision after the no-new-dependency contract passes:
  Tauri v2, PWA-first, or another thin shell.
