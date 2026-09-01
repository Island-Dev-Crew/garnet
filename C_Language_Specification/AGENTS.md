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

`GARNET_TRUST_KERNEL_ROLLING_REVIEW.md` is a procedural governance contract as
well as a normative policy. Its v2 gate must fail closed on incomplete Git
enumeration and accept trust-kernel review only from a canonical, exact-path,
content-bound W_TRUST record whose GitHub review and author identities are
authenticated through the explicit bounded transport. Preserve the two
evidence-bounded states: premerge records bind `reviewed_head` without
inventing a landed commit; registered postmerge markers prove squash-landed
content from upstream main's first-parent history without requiring pre-squash
ancestry.

`GARNET_WV_ACCEPTANCE_SUCCESSION_CONTRACT.md` owns the adopted but inactive WV
succession, event, effectiveness, eligibility, registry, and conservation
contract shapes. Its `OPEN-UNTIL-IMPLEMENTED` markers are activation blockers,
not acceptance evidence or claim upgrades; later code must trap every named
mechanical test before any instance can become effective.

## Agent Documentation Runtime Contracts

`GARNET_Agent_Documentation_Runtime_Contracts.md` records the Space-Agent-inspired design note that repo-local markdown contracts are procedural memory. Treat it as a design bridge: tooling may implement it before the grammar does.
