# Contributing to Garnet

*"In the multitude of counsellors there is safety." — Proverbs 11:14*

Thank you for your interest in contributing to the Garnet programming language. This document explains how the project is organized, how to set up a development environment, and what we expect from contributors.

## Code of Conduct

Garnet's community is built on mutual respect, intellectual honesty, and a shared commitment to making a serious programming language. We expect all contributors to:

- Treat others with respect, even in heated technical debates
- Lead with evidence and reasoning, not appeals to authority
- Be generous with explanations — assume good questions, not malice
- Credit prior art and acknowledge influences
- Remember that the project was started by a small team with big ambitions

Harassment, gatekeeping, or dismissive behavior will not be tolerated.

## How the Project Works

### Architecture Overview

Garnet is a **dual-mode, agent-native programming language** that reconciles Rust's compile-time safety with Ruby's expressive ergonomics, designed for long-horizon agentic systems. Key concepts:

- **Managed mode** (default) — ARC memory, dynamic-ish types, Ruby-like surface syntax
- **Safe mode** (`@safe`) — ownership + borrowing, static types, Rust-level performance
- **First-class memory abstractions** — working, episodic, semantic, and procedural memory units
- **Typed actors** — compiler-enforced message protocols for concurrent systems
- **Recursive execution guardrails** — `@max_depth`, `@fan_out` annotations for agent spawn safety
- **Compiler-as-agent cache** — learns from compilation history to optimize future builds

### Workspace Structure

The repository is organized as a Rust Cargo workspace:

```
.
├── Cargo.toml                          # workspace manifest
├── garnet-parser-v0.3/                 # Rung 2.1: lexer + recursive-descent parser (213 tests)
├── garnet-interp-v0.3/                 # Rung 3: tree-walk interpreter + REPL (372 tests)
├── garnet-check-v0.3/                  # Rung 4: safe-mode validator + borrow checker (35 tests)
├── garnet-memory-v0.3/                 # Rung 5: four memory primitives reference (41 tests)
├── garnet-actor-runtime/               # Rung 6: concurrent actor runtime (33+5 reload tests)
├── garnet-stdlib/                      # standard library (74 tests)
├── garnet-convert/                     # multi-source converter (85 tests)
├── garnet-cli/                         # CLI binary: garnet parse/check/run/eval/repl (12 smoke)
└── xtask/                              # meta: determinism harness (7× consistency)
```

Total: ~22K LOC Rust, 857+ passing tests, zero clippy warnings with `-D warnings`.

### The Language Spec

The normative specification is `spec/GARNET_v1_0_Mini_Spec.md`. This is the source of truth for the grammar, semantics, mode boundaries, actor protocols, and type system. All crates implement against this spec.

### Research Papers

The foundational research lives in `research/`:
- Paper I — Rust deep dive
- Paper II — Ruby deep dive
- Paper III — Garnet synthesis (v2.1)
- Paper IV — Garnet for agentic systems (v2.1.1)
- Paper V — Formal grounding (affine type theory, RustBelt)
- Paper VI — Empirical validation protocol
- Paper VII — Implementation ladder and tooling

## Setting Up a Development Environment

### Prerequisites

- **Rust** 1.75+ (stable) — install via `rustup`
- **Python 3.10+** for test fixtures and converter tests (optional)
- **SQLite3** CLI for inspecting `.garnet-cache/` databases (optional)

### Build

```bash
cargo build --workspace              # build all crates
cargo test  --workspace --no-fail-fast  # run full test suite (~857 tests)
```

### Run the CLI

```bash
cargo run -p garnet-cli -- --help
cargo run -p garnet-cli -- repl      # interactive REPL
cargo run -p garnet-cli -- check examples/greeter_actor.garnet
```

### Determinism Verification

```bash
cargo run -p xtask -- seven-run      # runs test suite 7×, asserts identical pass/fail
```

### Stress Tests (opt-in)

```bash
cargo test --workspace -- --ignored  # 6 stress tests at 100K+ scale
```

## Contribution Workflow

1. **Open an issue** describing the bug, feature, or RFC. Describe the problem, not just the solution.
2. **Discuss** — maintainers will triage and label. For large features, we may ask for a brief design note.
3. **Fork and branch** — create a branch off `main`. Name it `feat/`, `fix/`, or `doc/` prefixed.
4. **Write tests** — all new code must have tests. No exceptions.
5. **Pass CI** — your PR must pass `cargo test --workspace`, `cargo clippy --workspace -- -D warnings`, and the 7× determinism harness.
6. **Submit a PR** — link the issue, describe what changed, note any breaking changes.
7. **Review** — at least one maintainer review is required before merge.

### Testing Expectations

| Crate | Minimum coverage expectation |
|---|---|
| garnet-parser-v0.3 | All grammar productions + negative tests for malformed input |
| garnet-interp-v0.3 | Every AST node type + end-to-end programs |
| garnet-check-v0.3 | All violation types + annotation bounds |
| garnet-memory-v0.3 | Property-based tests for R+R+I decay |
| garnet-actor-runtime | Tell, ask, lifecycle + hot-reload migration |
| garnet-convert | Corpus tests: Rust → Garnet → output parity |
| garnet-cli | Subprocess smoke tests for every subcommand |

### Commit Message Convention

We use a lightweight convention:

```
type(scope): brief description

type: feat | fix | doc | refactor | test | chore | perf
scope: parser, interp, check, memory, actor, stdlib, convert, cli, xtask, spec
```

Examples:
- `feat(interp): add pattern matching for enum destructuring`
- `fix(check): reject @safe fn with implicit allocation on hot path`
- `doc(spec): clarify actor protocol semantics in §9.3`

### Pull Request Checklist

- [ ] Tests added or updated
- [ ] `cargo test --workspace` passes
- [ ] `cargo clippy --workspace -- -D warnings` passes clean
- [ ] Commit messages follow the convention above
- [ ] Breaking changes documented in PR description
- [ ] If the spec changed, `spec/GARNET_v1_0_Mini_Spec.md` is updated

## Good First Contributions

Look for issues labeled `good-first-issue`. These are typically:

- Adding tests for uncovered grammar productions
- Improving error messages in the parser or checker
- Adding examples to `examples/`
- Writing documentation or fixing typos
- Extending the standard library with safe utility functions

### How to Add an Example

1. Create a `.garnet` file in `examples/` with a descriptive name
2. The file should compile and run via `garnet run examples/<name>.garnet`
3. Add a `//` comment at the top explaining what it demonstrates
4. Submit a PR with the example file

## Reporting Bugs

When reporting a bug, include:

1. The `.garnet` source that triggers the issue
2. Expected behavior (what the spec says should happen)
3. Actual behavior (what the compiler/interpreter did)
4. Garnet version (`garnet --version`)
5. Platform (OS, Rust version)

For **spec bugs** (where the spec itself is wrong or contradictory), reference the spec section number.

## Requesting Features

Feature requests should include:

1. What problem the feature solves
2. Why this approach over alternatives
3. Whether it changes existing semantics (breaking or additive)
4. A sketch of the surface syntax, if applicable

## License

By contributing to Garnet, you agree that your contributions will be licensed under both the Apache License, Version 2.0 and the MIT License (the same dual license as the project itself). All contributions are made under the DCO 1.1 (Developer Certificate of Origin).

---

*"A man that hath friends must shew himself friendly." — Proverbs 18:24 (KJV)*

Garnet is dual-licensed under Apache-2.0 and MIT. See `LICENSE` for details.
