# AGENTS.md — Language Specification Contracts

## Scope

This folder owns Garnet's normative language and architecture specifications: grammar, memory manager design, compiler architecture, installer/release contracts, benchmarking plans, and future-facing design notes.

## Stable Contracts

- Treat Mini-Spec v1.0 and formal grammar documents as semantic memory for the language surface.
- Do not change language semantics in implementation crates without either citing an existing spec rule or updating the relevant spec in the same change.
- Keep experimental ideas clearly labeled as proposals, ADRs, or roadmap items until implemented and tested.
- Preserve the distinction between language core, runtime concerns, and agent-harness concerns.

## Documentation Updates

Update this contract when a new normative spec area is added or an existing one changes ownership. If a project-management handoff captures a durable language rule, promote it into this folder instead of leaving it only in `F_Project_Management/`.

## Agent Documentation Runtime Contracts

`GARNET_Agent_Documentation_Runtime_Contracts.md` records the Space-Agent-inspired design note that repo-local markdown contracts are procedural memory. Treat it as a design bridge: tooling may implement it before the grammar does.
