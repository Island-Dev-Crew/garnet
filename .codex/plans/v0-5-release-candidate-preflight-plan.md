# v0.5 Release Candidate Preflight Plan

Date: 2026-05-20
Branch: `codex/v0-5-release-candidate-preflight`
Contract: `F_Project_Management/GARNET_v0_5_SLICE_DOGFOOD.md` §"v0.5.0 Release Gate"

## Goal

Prepare the repository for the v0.5.0 release-gate reproduction without tagging
v0.5.0 early. This is a release-candidate proof slice, not one of the S1-S10
feature slices and not a public release announcement.

## Scope

1. Move the canonical workspace and `garnet` CLI version surfaces from `0.4.2`
   to `0.5.0`.
2. Retarget installer defaults and release-smoke defaults to `0.5.0`, while
   preserving source fallback and avoiding any claim that release assets already
   exist.
3. Keep the public installer reproducible before and after the tag by making the
   source fallback prefer the requested release tag when it exists and fall back
   to `main` only when the tag is not available.
4. Remove or rewrite stale v0.5 draft language that says the tag already exists.
5. Update the release-gate ledger/current-state text with exact remaining
   blockers: clean-machine installer proof, VSIX/editor proof, and release asset
   publication.

## Out of Scope

- Creating or pushing the `v0.5.0` tag.
- Claiming Apple notarization, Windows/Linux runtime proof, or provider-backed
  LLM compiler assistance.
- Publishing VSIX marketplace assets; local/package artifact proof is acceptable
  only if recorded as local artifact proof.

## Verification

Run the focused release-candidate ladder:

```bash
cargo metadata --format-version 1 --no-deps
GARNET_INSTALL_MODE=source GARNET_SOURCE_REPO_URL=file://$PWD GARNET_SOURCE_REF=codex/v0-5-release-candidate-preflight GARNET_PREFIX=/tmp/garnet-v0-5-rc-source sh installer/sh.garnet-lang.org/install.sh
/tmp/garnet-v0-5-rc-source/bin/garnet --version
GARNET_INSTALL_MODE=auto GARNET_SOURCE_REPO_URL=file://$PWD GARNET_SOURCE_FALLBACK=1 GARNET_PREFIX=/tmp/garnet-v0-5-rc-auto sh installer/sh.garnet-lang.org/install.sh
/tmp/garnet-v0-5-rc-auto/bin/garnet new --template cli /tmp/garnet-v0-5-demo
/tmp/garnet-v0-5-rc-auto/bin/garnet test /tmp/garnet-v0-5-demo
/tmp/garnet-v0-5-rc-auto/bin/garnet run /tmp/garnet-v0-5-demo/src/main.garnet
(cd editors/vscode && npm run package)
python3 scripts/garnet_mit_readiness_status.py --format json
cargo fmt --all -- --check
cargo test --workspace --no-fail-fast
cargo clippy --workspace --all-targets -- -D warnings
cargo deny check
```

If the auto installer cannot prove the local branch before the tag, record that
as a blocker rather than weakening the release gate.
