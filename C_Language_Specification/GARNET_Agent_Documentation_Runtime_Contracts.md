# Garnet Agent Documentation Runtime Contracts

## Status

Design note / ADR. This is not yet a grammar change.

## Thesis

Repo-local markdown contracts are procedural memory for agent-native software engineering.

Space Agent demonstrated a practical pattern: a root `AGENTS.md` plus subsystem-local `AGENTS.md` files let long-horizon coding agents keep local intent, boundaries, and "what not to do" near the code. This reduces redundant rewrites, context-window pressure, and architecture drift.

Garnet already models memory as four kinds. The documentation hierarchy maps cleanly onto that model:

- Working memory: active task plans and run ledgers.
- Episodic memory: dated handoffs and verification logs.
- Semantic memory: specs, research papers, architecture docs, and factual project knowledge.
- Procedural memory: `AGENTS.md`, reusable workflows, command ladders, safety rules, and tool contracts.

## Decision

Garnet should dogfood documentation-as-procedural-memory before adding syntax. The initial implementation is repository and tooling level:

1. Maintain a root `AGENTS.md` contract index.
2. Maintain subsystem-local `AGENTS.md` contracts for crates, templates, specs, examples, and project management.
3. Validate the hierarchy with `scripts/check-agent-contracts.py`.
4. Later, compile the hierarchy into a machine-readable contract map for agents and CI.
5. Only consider grammar support after tooling proves the shape.

## Future Tooling Candidates

- `garnet agent doctor` — inspect agent-readiness of a project.
- `garnet contract check` — validate local documentation contracts.
- `garnet contract build` — emit `.garnet-contract/contract-map.json` and procedural-memory indexes.
- `garnet new --agent-docs` — scaffold project-local `AGENTS.md` files and memory/run ledgers.
- `garnet doc --json-context` — expose a structured Context Window-style project/action map.

## Non-Goals For Now

- No new Garnet keyword yet.
- No parser or grammar change yet.
- No hard requirement that every project outside this repository use `AGENTS.md`.
- No replacement for README, public docs, or formal specs.

## Rationale

This lets Garnet improve its own agentic build process immediately while preserving grammar discipline. If the pattern proves valuable across real agent builds, the compiled-contract layer can become a stronger language/tooling contribution.
