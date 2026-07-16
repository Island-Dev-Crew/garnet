# AGENTS.md — Garnet Runtime Documentation Contract

## Documentation First

Treat every `AGENTS.md` file as part of Garnet's runtime documentation contract, not as optional contributor notes. Garnet is an agent-native language platform; long-horizon agents must be able to recover local intent, invariants, and "what not to break" from files that live beside the code they govern.

This repo uses a documentation hierarchy:

- `/AGENTS.md` owns repo-wide rules, the contract index, and cross-cutting architecture.
- Crate-level `AGENTS.md` files own implementation contracts for each Rust crate.
- Spec and project-management `AGENTS.md` files distinguish normative language truth from episodic handoff history.
- Template docs under `garnet-cli/templates/` define what new Garnet projects should teach agents by default.

The closer the doc is to the code, the more concrete it should be. Parent docs explain boundaries and stable seams; child docs explain local behavior, invariants, tests, and update rules.

## Memory-Kind Mapping

Garnet's own memory taxonomy applies to the repository:

- Working memory: current task plans, local run notes, and active PR descriptions.
- Episodic memory: handoffs, verification logs, release notes, and dated project-state files.
- Semantic memory: language specs, architecture docs, research papers, and public README/FAQ material.
- Procedural memory: `AGENTS.md`, contribution rules, test ladders, commands, and repeatable workflows.

A stable workflow that changes agent behavior belongs in procedural memory, not only in a chat transcript or a maintainer's head.

## Lane 0 Truth-Freeze Gate

`python3 -I scripts/garnet_lane0_truth_freeze_status.py --gate` is the machine
authority for the Lane 0 first-parent archive and U-18 resume contract. It
derives the archived PR order and full squash-main SHAs from local Git history,
then checks the materialized P7/P7-T1..P7-T4 references and rejects stale
P8/P9/P10 resume pointers. Run
`python3 -I scripts/test_garnet_lane0_truth_freeze_status.py` after changing the
checkpoint, its locked P0 gates, or the mission SOTU renderer. This gate reads
only the local checkout; it must not read a fork's main branch or ambient
credentials.

## Rust MSRV Contract

Cargo `rust-version = "1.95"` is the single workspace MSRV. Every active
workspace member inherits that value; the excluded Studio backend and parser
fuzz workspace declare it directly. Ordinary CI continues to track moving
stable, while the existing required CI and Studio contexts also compile under
exact Rust 1.95.0. Do not add a `rust-toolchain.toml` pin or raise the floor
without updating every active manifest, current public/contributor surface,
the existing required workflow checks, and this contract in one Jon-reviewed
change.

Run `python3 -I scripts/test_garnet_msrv_status.py` and
`python3 -I scripts/garnet_msrv_status.py --gate` after changing a Rust
manifest, current MSRV wording, or the exact-floor workflow wiring. The gate
uses the pinned typed workflow-YAML boundary from
`scripts/garnet_workflow_yaml_requirements.txt`; comments, disabled steps, or
commands in another job do not satisfy it.

## WV-6 / WV-7 Acceptance Gates

`F_Project_Management/LAUNCH/WV6_WV7_ACCEPTANCE_CONTRACTS.json` preserves the
established meanings: WV-6 is the native-Windows Core Ring Tier 1 + Minimum
Shelf/MCP proof, and WV-7 is the winget/Scoop dry-run + devcontainer/Docker +
installer happy-path distribution proof. Run
`python3 -I scripts/test_garnet_wv_acceptance_status.py` after changing either
contract or reporter. The two `--gate` commands are expected to exit nonzero
with `state=pending` until their exact-candidate, hash-verified Windows evidence
manifests exist; never turn absence into acceptance or perform a Jon-only
action from the reporter.

## Required Contract Index

Every path below is part of the current contract surface and must remain present unless the owning scope is removed or renamed.

- `/AGENTS.md`
- `/C_Language_Specification/AGENTS.md`
- `/F_Project_Management/AGENTS.md`
- `/garnet-parser-v0.3/AGENTS.md`
- `/garnet-parser-v0.3/fuzz/AGENTS.md`
- `/garnet-interp-v0.3/AGENTS.md`
- `/garnet-check-v0.3/AGENTS.md`
- `/garnet-memory-v0.3/AGENTS.md`
- `/garnet-actor-runtime/AGENTS.md`
- `/garnet-stdlib/AGENTS.md`
- `/garnet-cli/AGENTS.md`
- `/garnet-cli/templates/AGENTS.md`
- `/garnet-convert/AGENTS.md`
- `/garnet-cst/AGENTS.md`
- `/garnet-prim-macros/AGENTS.md`
- `/garnet-lsp/AGENTS.md`
- `/garnet-suggest-llm/AGENTS.md`
- `/garnet-vm/AGENTS.md`
- `/garnet-wasm/AGENTS.md`
- `/garnet-registry-stub/AGENTS.md`
- `/apps/garnet-studio/src-tauri/AGENTS.md`
- `/apps/garnet-studio-macos/AGENTS.md`
- `/examples/AGENTS.md`
- `/xtask/AGENTS.md`

Run `python3 scripts/check-agent-contracts.py` after changing this index or any `AGENTS.md` file.

## Change Rules

Before editing a subsystem, read the closest owning `AGENTS.md` plus this root file.

When a code change alters behavior, ownership, invariants, public commands, template shape, or required tests, update the closest owning `AGENTS.md` in the same change. Update parent docs too when the higher-level architecture or boundary changes.

Do not let handoff files become the only source of current truth. If a handoff records a durable rule, promote that rule into the relevant spec or `AGENTS.md` file.

Do not add hidden compatibility seams, generated artifacts, or ad hoc scratch directories as tracked content unless the owning contract says they are durable project state.

## Phase ID Allocation

Phase identifiers (e.g. `Phase 6BT`) are a single shared global counter. With
multiple concurrent agents, hand-picking a letter causes collisions (PR #74 and
PR #75 both shipped as "Phase 4BI"). Before choosing a phase id:

1. Run `python3 scripts/garnet_phase_id.py` and use the id it prints.
2. Never hand-pick or reuse a letter from memory or a stale handoff.
3. CI and agents may run `python3 scripts/garnet_phase_id.py --check <ID>`;
   it exits non-zero if `<ID>` already appears in the implementation plan,
   the phase ownership register, or recent git history.

This rule is procedural memory: it changes agent behavior, so it lives here,
not only in a transcript.

## Verification Ladder

For documentation-contract changes, run:

1. `python3 scripts/check-agent-contracts.py`
2. `python3 scripts/test_check_agent_contracts.py`
3. `cargo fmt --all -- --check`
4. `cargo test -p garnet-cli new_cmd`
5. `cargo test --workspace --no-fail-fast` when Rust behavior changed.

For release-impacting work, follow the latest verification ladder in `F_Project_Management/`.
