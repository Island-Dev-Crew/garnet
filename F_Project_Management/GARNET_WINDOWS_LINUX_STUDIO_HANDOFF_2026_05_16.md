# Garnet Windows/Linux Studio Handoff

Status: executable handoff packet.
Date: 2026-05-16.
Base truth: PR #140 merged at `e944cf5e52cdcbcf3dbd77f2c1f3dce17411824e`.

## Objective

Build the first cross-platform Garnet Studio MVP for Windows and Linux without weakening current truth:

- Reuse the existing Garnet CLI, docs/PWA, converter advisory pipeline, and dogfood readiness gates.
- Do not port SwiftUI directly to Windows/Linux.
- Prefer a Tauri/PWA shell or equivalent thin desktop wrapper around the verified CLI and web surfaces.
- Keep macOS SwiftUI Studio as the native Apple reference app.
- Preserve the same language taxonomy: active conversion, advisory planning, native boundary, and future backend lowering.

## Source Setup

```sh
git clone https://github.com/Island-Dev-Crew/garnet.git
cd garnet
git fetch --prune origin
git checkout main
git log --oneline --decorate --max-count=8
```

Expected current or newer baseline:

```text
e944cf5 Merge pull request #140 from Navigata1/codex/studio-advisory-handoff-ux
```

If the branch is newer, treat live `origin/main` as source of truth and re-run the status scripts below.

## Required First Verification

```sh
cargo fmt --all -- --check
git diff --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --no-fail-fast
python3 scripts/test_garnet_converter_status.py
python3 scripts/test_garnet_adoption_surface_status.py
python3 scripts/test_run_agentic_dogfood_matrix.py
python3 scripts/garnet_adoption_surface_status.py
python3 scripts/garnet_mit_readiness_status.py
```

If Swift tooling is unavailable on Windows/Linux, do not mark SwiftUI Studio tests as failed product blockers. The cross-platform MVP should prove its own surface while preserving macOS SwiftUI verification on macOS.

## Codex Desktop on Windows Lane

Recommended owner: GPT-5.5 xhigh/fast when available.

2026-05-17 update: live `origin/main` is newer than this handoff baseline. The
first Windows-side slice is now captured in
`F_Project_Management/GARNET_WINDOWS_LINUX_STUDIO_MVP_ARCHITECTURE_2026_05_17.md`
and `scripts/garnet_windows_linux_studio_status.py`. Treat the zip/handoff
packet as input, not as source truth. The current CLI has no `garnet health`
subcommand, so the Studio `CLI Health` action maps to `garnet version` unless a
real health command is added later.

Primary work:

1. Create the cross-platform Studio shell plan.
2. Choose the least-new-dependency MVP path. Tauri is preferred only if the repo can accept the dependency and the build can be verified on Windows and Linux.
3. Expose the following actions:
   - CLI Health
   - Parse
   - Check
   - Run
   - Convert for Rust/Ruby/Python/Go only
   - Assist Plan for advisory languages
   - Advisory Bundle
   - Advisory Review
   - Advisory Handoff
   - Objective Pulse
   - Agentic Dogfood Matrix
4. Save evidence to a local dogfood folder equivalent to `~/Desktop/dogfood`.
5. Add tests for command construction, evidence directory creation, and copy truth.
6. Preserve line-of-sight to existing scripts instead of duplicating converter logic.

Suggested branch:

```sh
git checkout -b codex/windows-linux-studio-mvp
```

## Claude Code on Windows Lane

Recommended owner: Opus 4.7 max thinking after reset.

Primary work:

1. Review the cross-platform shell plan against current docs and prior Windows release verification.
2. Build Windows packaging/release gates around the CLI and future Studio shell:
   - MSVC build
   - MSI packaging plan
   - Authenticode signing plan
   - winget manifest plan
   - clean-machine smoke checklist
3. Produce a PR-ready release runbook update that separates verified source installs from future signed MSI claims.
4. Run adversarial copy review on website and README so they do not claim Windows package completion before evidence exists.

Suggested branch:

```sh
git checkout -b claude/windows-release-productization-gates
```

## Linux Lane

Primary work:

1. Verify existing `.deb` and `.rpm` smoke tests still pass.
2. Decide whether the first Linux Studio package is AppImage, `.deb`, `.rpm`, Flatpak, or source/PWA shell only.
3. Keep the first MVP simple: installable shell plus CLI smoke is enough; do not block on every packaging format at once.
4. Add Linux evidence bundle commands and update the dogfood matrix only when the package can be exercised.

## Shared Truth Contract

The cross-platform Studio must preserve these labels:

| Menu group | Languages |
| --- | --- |
| Active conversion | Rust, Ruby, Python, Go |
| Advisory planning | JavaScript, TypeScript, Swift, Java, C, C++, C#, Perl, Kotlin, Shell, SQL, Other |
| Native boundary recommended | C, C++, Objective-C, Assembly, CUDA, platform-specific code |
| Future backend lowering | Wasm, LLVM-style native targets, native package toolchains |

The UI must not call provider APIs, execute source code, include source in provider packets by default, or mark advisory output safe.

## PR Shape

Keep PRs sliced:

1. Cross-platform Studio architecture doc and tests.
2. Minimal shell scaffold.
3. CLI health + parse/check/run actions.
4. Converter active/advisory UI.
5. Evidence directory + dogfood matrix integration.
6. Windows package smoke.
7. Linux package smoke.
8. Website readiness copy sync.

Each PR body must include:

- Current truth.
- Local verification.
- Remote verification after checks complete.
- Desktop/local dogfood bundle path.
- Deferred/out-of-scope claims.

## Completion Criteria

The first Windows/Linux Studio MVP is not complete until:

- It launches on at least one Windows machine and one Linux environment.
- It can locate or bundle `garnet`.
- It can run parse/check/run against local examples.
- It can produce advisory handoff evidence without source inclusion by default.
- It can run or invoke the agentic dogfood matrix.
- It has screenshots or logs in the dogfood archive.
- README and site copy distinguish verified install paths from future signed package distribution.
