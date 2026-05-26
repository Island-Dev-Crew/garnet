# mac-codex S18 Plan - First five Layer-2 packages

Date: 2026-05-26
Branch: `agent-mac-codex/s18-llm-package`
PRD: `F_Project_Management/PRD_D_S18_S19_PACKAGES_LLM.md`, sections "Implementation Plan - S18", "Dogfood block (verification) / S18", "Out of scope", and "Coordination".
Slice contract: `F_Project_Management/GARNET_v0_7_SLICE_DOGFOOD.md`, section "S18 - First five Layer-2 packages (`garnet-lang/*`)".

## Current gate

S18 is unblocked by the S17 MERGED ledger entry and S19 is already merged. Fresh baseline on `origin/main` `b5cfcbf` passed:

- `python3 scripts/garnet_mit_readiness_status.py` -> 82.1%
- `cargo test --workspace --no-fail-fast`
- `cargo clippy --workspace --all-targets -- -D warnings`

The external publication lane is not fully available yet: `gh api orgs/garnet-lang` returns 404 / missing `admin:org`, and `gh repo list garnet-lang` reports the owner handle is not recognized. Per the S18 contract, creating the `garnet-lang` org is Jon's manual step.

## Scope split

Proceed now:

- Add `tools/garnet-lang-template/` with calibrated-honesty README, dual licenses, CHANGELOG, `Garnet.toml`, `garnet/lib.garnet`, and `tests/smoke.garnet`.
- Add a repo-local registry-seed fixture for the five Layer-2 packages so package content, registry integrity, vendoring, and `garnet run` can be dogfooded without claiming external publication.
- Add `examples/mvp_18_all_official_packages.garnet` and a matching smoke script/test that uses the actual S13 CLI shape: `garnet add --registry <filesystem-registry> <name>@0.1.0`.
- Add an `official_packages_seed` readiness lane labeled `local-registry-seed` or stronger only if evidence supports it.
- Update S18 sections of `CHANGELOG.md`, `CURRENT_STATE.md`, and `F_Project_Management/GARNET_v0_7_SLICE_DOGFOOD.md` with the external-org blocker explicit.

Do not claim yet:

- Five external repos created under `github.com/garnet-lang/`.
- Public registry publication.
- Source-level `@stability(...)` enforcement on package functions. The S17 policy explicitly says source-level annotations still wait on the parser annotation handoff, so S18 can declare package stability in manifest/docs and keep source syntax runnable.

## Implementation sequence

1. Template: create the reusable package scaffold under `tools/garnet-lang-template/`.
2. Local package seed: create five registry packages (`http-client`, `llm`, `cli`, `test-property`, `log`) with v0.1.0 manifests/docs and runnable Garnet functions.
3. Registry index: build and verify a filesystem-backed `index.json` for the local seed.
4. Example project: add `examples/mvp_18_all_official_packages.garnet` and a smoke path that vendors all five packages into a temp project before running it.
5. Status surfaces: update S18 dogfood/readiness/current-state/changelog with calibrated labels.
6. Verification: run the S18 dogfood block, relevant example checks, script tests, readiness no-regression, workspace tests, clippy, and diff checks before PR-open.

## Dogfood target

Until the external org exists, the S18 reproducible block is local-registry source proof:

```bash
cargo run -p garnet-registry-stub -- build examples/garnet_lang_registry_seed
cargo run -p garnet-registry-stub -- verify examples/garnet_lang_registry_seed
tmpdir=$(mktemp -d)
cp -R examples/mvp_18_all_official_packages "$tmpdir/project"
(cd "$tmpdir/project" && \
  cargo run --manifest-path "$OLDPWD/Cargo.toml" -p garnet-cli -- add --registry "$OLDPWD/examples/garnet_lang_registry_seed" http-client@0.1.0 && \
  cargo run --manifest-path "$OLDPWD/Cargo.toml" -p garnet-cli -- add --registry "$OLDPWD/examples/garnet_lang_registry_seed" llm@0.1.0 && \
  cargo run --manifest-path "$OLDPWD/Cargo.toml" -p garnet-cli -- add --registry "$OLDPWD/examples/garnet_lang_registry_seed" cli@0.1.0 && \
  cargo run --manifest-path "$OLDPWD/Cargo.toml" -p garnet-cli -- add --registry "$OLDPWD/examples/garnet_lang_registry_seed" test-property@0.1.0 && \
  cargo run --manifest-path "$OLDPWD/Cargo.toml" -p garnet-cli -- add --registry "$OLDPWD/examples/garnet_lang_registry_seed" log@0.1.0 && \
  cargo run --manifest-path "$OLDPWD/Cargo.toml" -p garnet-cli -- run src/main.garnet)
```

## Stop conditions

- If the package content requires parser or checker changes outside mac-codex ownership, keep it as manifest/docs metadata and file/point to the existing handoff.
- If external repo creation remains unavailable, land only the local registry seed with `pending-external-org` labels and do not mark S18 MERGED.
- If registry CLI behavior prevents the dogfood flow, update only the S18 block to match the actual implemented CLI and keep the release-gate claim below shipped.
