# mac-codex S19 Plan - Compiler-as-agent LLM tier

Date: 2026-05-24
Updated: 2026-05-26 after syncing `origin/main` through S17 PR #231
Branch: `agent-mac-codex/s19-suggest-llm`
Base: `origin/main` at `2d655a0` after fast-forwarding over S17 merge PR #231
PRD: `F_Project_Management/PRD_D_S18_S19_PACKAGES_LLM.md`, sections "Implementation Plan - S19", "Dogfood block (verification) / S19", "Out of scope", "Coordination", and "Honest accounting hooks".
Slice contract: `F_Project_Management/GARNET_v0_7_SLICE_DOGFOOD.md`, section "S19 - Compiler-as-agent LLM tier (`garnet-suggest-llm`)".

## Current truth

- Pre-edit and post-sync baselines on current `origin/main` passed:
  - Initial S19 start on `09d6703` reported 80.6% before S16 merged.
  - After syncing to `d103525`, `python3 scripts/garnet_mit_readiness_status.py` reported 81.4% with S16 and the local S19 lane present.
  - After syncing to `2d655a0`, live readiness reports 82.1% with S17 and the local S19 lane present. The committed no-regression baseline is regenerated as the source-only floor (80.5%) because local promo and Windows clean-VM evidence are evidence-root dependent.
  - `cargo test --workspace --no-fail-fast` exits 0.
  - `cargo clippy --workspace --all-targets -- -D warnings` exits 0.
- Ledger status at start:
  - S15 merged and S15-Compare chose rowan `garnet-cst`.
  - S16 has since merged on origin/main via PR #230.
  - S17 has merged on origin/main via PR #231, so S19 can use the `@stability` vocabulary. S18 is now unblocked but remains a separate slice/PR.
- S19 has a soft S18 dependency for the shared `garnet-lang/llm` package trait. Until S18 lands, S19 keeps a local Rust trait boundary and labels the shared package trait as deferred.
- The exact public CLI command is a read-only `garnet-cli` surface. This plan
  therefore lands the feature-gated crate and files a handoff before any
  `garnet-cli` or `garnet-check-v0.3` modification.

## Writable scope

- Primary owned surface: new `garnet-suggest-llm/` crate.
- Cross-cutting files allowed by ledger rules:
  - `Cargo.toml`: append-sort workspace member only.
  - `CHANGELOG.md`: append S19 bullet under `[Unreleased]`.
  - `CURRENT_STATE.md`: update only the S19/current-state section or add a clearly bounded S19 note if absent.
  - `F_Project_Management/GARNET_v0_7_SLICE_DOGFOOD.md`: update only the S19 contract block.
  - `scripts/garnet_mit_readiness_status.py`: add only the `compiler_agent_llm_tier` lane definition and tests.
- Read-only unless a ledger handoff is accepted: `garnet-check-v0.3`, `garnet-stdlib`, `README.md`, `C_Language_Specification/GARNET_v1_0_Mini_Spec.md`.

## Implementation sequence

1. Confirm the public surface available from `garnet-check-v0.3::suggest`.
   - If `Suggestion` or the module data needed by PRD D is not public, do not edit `garnet-check-v0.3`.
   - File a Handoff Request to the owning slot instead, then keep S19 to mockable/local types where honest.

2. Add the `garnet-suggest-llm` crate behind `default = []` and `llm = []`.
   - Keep provider-backed code opt-in.
   - Define a local Rust `LlmClient` trait only as a pending bridge while S18 is still a separate PR, with comments stating the Garnet Layer-2 `garnet-lang/llm` trait is the intended source of truth.
   - Implement Anthropic, OpenAI, and Ollama clients using the least new dependency surface available. Prefer existing workspace dependency patterns; add no new dependency unless the provider implementation cannot be made useful without it.

3. Preserve deterministic-first behavior.
   - Provide an additive `suggest_for_module_with_llm`-style API that first consumes deterministic suggestions as ground truth, then appends clearly labeled non-deterministic suggestions.
   - Never replace or weaken the existing deterministic `garnet check --suggest` behavior.
   - Tag LLM output as `non-deterministic` in the S19 type/output layer.

4. Add reproducibility logging.
   - Write `.garnet-cache/llm-suggest-log.jsonl`.
   - Include prompt hash, provider/model, temperature or deterministic placeholder, response text or error envelope, emitted suggestions, and timestamp.
   - Keep logs local and project-scoped. Do not embed secrets.

5. Add the determinism guard.
   - Add a script/test that fails if a determinism workflow command includes `--llm`.
   - Do not edit unrelated workflow behavior.

6. Add Paper VI Exp 3 harness scaffold.
   - Create `benchmarks/paper_vi_exp3_compiler_as_agent/` with the PRD-required scripts and placeholder snapshots.
   - Label it as harness-only. Running the experiment for h3 results remains v0.7.1.

7. Update honest accounting surfaces.
   - Add `compiler_agent_llm_tier` to `scripts/garnet_mit_readiness_status.py` with status no stronger than the evidence supports.
   - Update `CHANGELOG.md`, `CURRENT_STATE.md`, and only the S19 dogfood block.
   - Keep README and Mini-Spec untouched unless a Handoff Request is approved.

8. Verify locally before PR.
   - `cargo fmt --all -- --check`
   - `cargo build --features llm -p garnet-suggest-llm --release`
   - `cargo test -p garnet-suggest-llm --features llm --no-fail-fast`
   - `cargo clippy -p garnet-suggest-llm --all-targets --features llm -- -D warnings`
   - `cargo test --workspace --no-fail-fast`
   - `cargo clippy --workspace --all-targets -- -D warnings`
   - `python3 scripts/garnet_mit_readiness_status.py`
   - `python3 scripts/garnet_mit_readiness_status.py --check-no-regression`
   - Any new Python unit tests for the determinism guard/readiness lane.

## Dogfood stance

- PR title when approved: `S19: compiler-as-agent LLM tier`.
- The PR body will state:
  - LLM suggestions are non-deterministic and excluded from determinism CI by design.
  - Deterministic suggestions remain the ground truth tier.
  - Streaming, function calling, tools, vision, vector-store integration, and h3 experiment results are out of scope.
  - Provider-backed execution requires explicit env configuration and is not part of clean deterministic CI.

## Stop conditions

- If `garnet-check-v0.3` must change to expose required types, stop and append a Handoff Request.
- If S17 lands during the work, re-read the ledger and PRD C output before finalizing stability wording.
- Do not merge without Jon approval. PR-open is allowed as `feature-gated-source-ready` after local dogfood passes; the public CLI line remains pending the read-only `garnet-cli` handoff.
