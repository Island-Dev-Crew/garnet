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

## Required Contract Index

Every path below is part of the current contract surface and must remain present unless the owning scope is removed or renamed.

- `/AGENTS.md`
- `/C_Language_Specification/AGENTS.md`
- `/F_Project_Management/AGENTS.md`
- `/garnet-parser-v0.3/AGENTS.md`
- `/garnet-interp-v0.3/AGENTS.md`
- `/garnet-check-v0.3/AGENTS.md`
- `/garnet-memory-v0.3/AGENTS.md`
- `/garnet-actor-runtime/AGENTS.md`
- `/garnet-stdlib/AGENTS.md`
- `/garnet-cli/AGENTS.md`
- `/garnet-cli/templates/AGENTS.md`
- `/garnet-convert/AGENTS.md`
- `/examples/AGENTS.md`
- `/xtask/AGENTS.md`

Run `python3 scripts/check-agent-contracts.py` after changing this index or any `AGENTS.md` file.

## Change Rules

Before editing a subsystem, read the closest owning `AGENTS.md` plus this root file.

When a code change alters behavior, ownership, invariants, public commands, template shape, or required tests, update the closest owning `AGENTS.md` in the same change. Update parent docs too when the higher-level architecture or boundary changes.

Do not let handoff files become the only source of current truth. If a handoff records a durable rule, promote that rule into the relevant spec or `AGENTS.md` file.

Do not add hidden compatibility seams, generated artifacts, or ad hoc scratch directories as tracked content unless the owning contract says they are durable project state.

## Verification Ladder

For documentation-contract changes, run:

1. `python3 scripts/check-agent-contracts.py`
2. `cargo fmt --all -- --check`
3. `cargo test -p garnet-cli new_cmd`
4. `cargo test --workspace --no-fail-fast` when Rust behavior changed.

For release-impacting work, follow the latest verification ladder in `F_Project_Management/`.
