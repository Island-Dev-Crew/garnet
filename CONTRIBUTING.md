# Contributing to Garnet

*"In the multitude of counsellors there is safety." — Proverbs 11:14*

Thank you for your interest in contributing to the Garnet programming language. This document explains Garnet's design North Star, how the repository is organized, how to choose between the **Contributor Fast Lane** and the **Trust-Kernel Review Lane**, how to set up a local environment, and what checks every pull request runs.

## North Star: Best of All Worlds

Garnet aims for a Best-of-All-Worlds synthesis:

- **Ruby clarity** — expressive managed-mode ergonomics (`def`, `unless`/`until`, trailing `do ... end` blocks, `|>`, readable defaults) so orchestration code reads cleanly.
- **Rust capability rigor** — explicit `@caps(...)` authority budgets, `@safe fn` ownership discipline, deterministic builds, and machine-recomputable evidence so humans can accept agent-authored and human-authored code on proof rather than guesswork.
- **Go simplicity** — one coherent toolchain (`garnet`), fast predictable workflows, minimal ceremony, and straightforward workspace boundaries.

## Code of Conduct

Garnet's community is built on mutual respect, intellectual rigor, and a shared commitment to making a serious programming language. We expect all contributors to:

- Treat others with respect, even in heated technical debates
- Lead with evidence and reasoning, not appeals to authority
- Be generous with explanations — assume good questions, not malice
- Credit prior art and acknowledge influences
- Remember that the project is built in the open with high verification standards

Harassment, gatekeeping, or dismissive behavior will not be tolerated. See [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md). For public inquiries, reach us at `hello@garnet-lang.org`; for private security reports, follow [SECURITY.md](SECURITY.md).

## Contributor Fast Lane vs. Trust-Kernel Review Lane

Garnet separates everyday language/tooling contributions from changes to the capability and release enforcement boundary. This keeps first-time and feature contributions fast while preserving strict review over the trust kernel.

### Fast Lane (non-trust-kernel)

Ideal for first contributions, parser ergonomics, CST/LSP tooling, migration frontends, examples, and documentation:

- `garnet-parser-v0.3/` — lexer, recursive-descent parser, AST, grammar tests
- `garnet-cst/` — rowan trivia-preserving CST, typed syntax wrappers, `cst_to_ast`
- `garnet-convert/` — Rust, Ruby, Python, and Go migration frontends and idioms
- `garnet-lsp/` — Language Server Protocol handlers, diagnostics, semantic tokens
- `examples/` — runnable Garnet programs and dogfood examples
- `docs/` — public documentation and guides (excluding `docs/why.html`)
- `CONTRIBUTING.md`, `README.md`, `FAQ.md`, and other contributor docs

**What to expect in the Fast Lane:**

- Normal PR review and standard CI — **no `F_Project_Management/W_TRUST/*.review.json` record is required** (`trust_kernel_touched: false`).
- `PR dogfood evidence` auto-skips on non-readiness paths (`garnet-cst/`, `garnet-convert/`, `garnet-lsp/`, `docs/`, `CONTRIBUTING.md`).
- Readiness-sensitive Fast-Lane paths (`garnet-parser-v0.3/`, `examples/`) still skip `W_TRUST`, and simply fill in the `## Dogfood Readiness` checklist from the PR template with recomputable local check output.

### Trust-Kernel Review Lane

Changes that touch capability enforcement, execution semantics, supply-chain dependencies, or CI/governance checks enter the Trust-Kernel Review Lane:

- `garnet-cli/src/` and `garnet-cli/Cargo.toml`
- `garnet-check-v0.3/src/`
- `garnet-interp-v0.3/src/`
- `garnet-vm/src/`
- `garnet-stdlib/src/`
- `garnet-wasm/src/`
- `.github/workflows/`, `.github/actions/`, `.github/rulesets/`, `.github/CODEOWNERS`
- `Cargo.lock`
- `docs/why.html`, `C_Language_Specification/GARNET_CAPABILITY_ENFORCEMENT_SCOPE.md`, `F_Project_Management/W_TRUST/`, and required-context producer scripts (`scripts/garnet_*`, `scripts/test_garnet_*`, `scripts/smoke_garnet_*`, `scripts/check_dogfood_pr_body.py`, `scripts/check-agent-contracts.py`, etc.)

**What to expect in the Trust-Kernel Lane:**

- Requires an independent cross-family review recorded in `F_Project_Management/W_TRUST/*.review.json` bound to the exact candidate HEAD SHA, plus maintainer sign-off ([GOVERNANCE.md](GOVERNANCE.md)).
- Verify whether your branch touches the trust kernel at any time with:

```bash
python3 -I scripts/garnet_trust_kernel_review_status.py --base origin/main --head HEAD
```

## How the Project Works

### Architecture Overview

- Language overview: [README.md](README.md) and the [Mini-Spec v1.0](C_Language_Specification/GARNET_v1_0_Mini_Spec.md).
- Enforced vs. declared authority: [capability enforcement scope table](C_Language_Specification/GARNET_CAPABILITY_ENFORCEMENT_SCOPE.md).
- Subsystem invariants and local contracts: read root [AGENTS.md](AGENTS.md) and the closest crate-level `AGENTS.md` before editing a subsystem.

### Workspace Structure

The repository is organized as a Rust Cargo workspace with **16 workspace crates** (plus **2 paused Studio apps** held outside the active release path), totaling **~110.5K Rust LOC** and **2,483+ passing Rust tests** under MSRV `1.95.0`:

```text
.
├── Cargo.toml                  # workspace manifest (public release v0.8.4, MSRV 1.95)
├── garnet-parser-v0.3/         # lexer, recursive-descent parser, AST, editions (Fast Lane)
├── garnet-cst/                 # rowan trivia-preserving CST + cst_to_ast (Fast Lane)
├── garnet-check-v0.3/          # @safe validator, borrow/mode checks, @caps propagator (Trust Kernel)
├── garnet-interp-v0.3/         # tree-walk interpreter + REPL + runtime gates (Trust Kernel)
├── garnet-vm/                  # bytecode compiler & VM with trap parity (Trust Kernel)
├── garnet-memory-v0.3/         # four-tier memory primitives reference (working/episodic/semantic/procedural)
├── garnet-stdlib/              # 84 capability- and stability-tagged core/std primitives (Trust Kernel)
├── garnet-actor-runtime/       # concurrent actor runtime + signed hot-reload staging crate
├── garnet-convert/             # Rust/Ruby/Python/Go → Garnet migration frontends (Fast Lane)
├── garnet-lsp/                 # Language Server Protocol server (Fast Lane)
├── garnet-wasm/                # WebAssembly playground & browser runtime bridge (Trust Kernel)
├── garnet-cli/                 # `garnet` binary: parse/check/run/eval/test/build/verify/diff-caps (Trust Kernel)
├── garnet-prim-macros/         # proc-macros for primitive metadata & registration
├── garnet-registry-stub/       # local package registry stub & fixtures
├── xtask/                      # repo automation: truth manifest checks & determinism harness
├── garnet-suggest-llm/         # slated for ADR-0016 D-42 retirement (models stay out of the trust path)
└── apps/
    ├── garnet-studio/          # paused/held desktop Studio app (excluded from root workspace)
    └── garnet-studio-macos/    # paused/held macOS Studio wrapper
```

### Versioning Policy

Garnet uses **one public release version, plus independent crate semver**:

- The **public release version** is `garnet-cli`, which inherits `[workspace.package].version` (currently **0.8.4**) via `version.workspace = true`. This is what `garnet --version`, the git tag (`v0.8.4`), and the signed release assets carry.
- The **internal library crates** (`garnet-parser`, `garnet-cst`, `garnet-check`, `garnet-interp`, `garnet-memory`, `garnet-vm`, `garnet-stdlib`, `garnet-convert`, `garnet-lsp`, …) carry their **own** `[package] version` reflecting their actual API evolution (for example, `garnet-parser` at `0.3.x`). They are **not** force-bumped to match the CLI release. Each inter-crate path dependency keeps an explicit `version = "x.y.z"` pin matching that crate's real version because workspace `cargo deny` rules ban wildcard (`*`) dependencies.

### The Language Spec & Research

- Normative specification: `C_Language_Specification/GARNET_v1_0_Mini_Spec.md`.
- Research corpus: `research/README.md` and `A_Research_Papers/` (Papers I–VII).

## Setting Up a Development Environment

### Prerequisites

- **Rust** 1.95+ (exact workspace MSRV `1.95.0`; install via `rustup`; CI builds on both exact Rust `1.95.0` and moving stable, with `clippy` pinned per [AGENTS.md](AGENTS.md))
- **Python 3.10+** for repo verification gates and LSP/converter checks
- **SQLite3** CLI for inspecting `.garnet-cache/` databases (optional)

### Build & Local Verification

Run targeted checks while iterating, and the full verification ladder before opening a PR:

```bash
# Formatting and lint (must pass with zero warnings)
cargo fmt --check
cargo clippy --workspace --all-targets --locked -- -D warnings

# Fast inner loop on a single crate
cargo test -p garnet-parser --locked
cargo test -p garnet-cst --locked
cargo test -p garnet-convert --locked
cargo test -p garnet-lsp --locked

# Full workspace test suite (2,483+ passing Rust tests)
cargo test --workspace --locked --no-fail-fast

# Repo governance and hygiene checks
python3 -I scripts/garnet_trust_kernel_review_status.py --base origin/main --head HEAD
python3 -I scripts/garnet_text_byte_policy_status.py --gate
python3 -I scripts/check-agent-contracts.py
cargo run -p xtask --locked -- truth --check
```

### Run the CLI

```bash
cargo run -p garnet-cli --locked -- --help
cargo run -p garnet-cli --locked -- repl
cargo run -p garnet-cli --locked -- check examples/mvp_01_os_simulator.garnet
cargo run -p garnet-cli --locked -- run examples/mvp_01_os_simulator.garnet
```

### Determinism & Opt-In Stress Checks

```bash
cargo run -p xtask --locked -- seven-run     # runs test suite 7×, asserts identical pass/fail
cargo test --workspace --locked -- --ignored # opt-in stress tests
```

## Contribution Workflow

1. **Pick or open an issue** — browse [`good first issue`](https://github.com/Island-Dev-Crew/garnet/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22) for scoped Fast-Lane tasks, or open an issue describing the problem you want to solve.
2. **Read local contracts** — check root [AGENTS.md](AGENTS.md) and the owning crate's `AGENTS.md` before editing.
3. **Fork and branch** — create a clean branch off `main` (`feat/`, `fix/`, `docs/`, or `chore/`).
4. **Write tests** — all behavior changes require focused tests (positive and negative cases where applicable).
5. **Run local verification** — `cargo fmt --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo test -p <crate> --locked`, and the repo gate commands above.
6. **Submit a PR** — link the issue, note whether the PR is Fast Lane or Trust-Kernel Lane, and include the exact local commands you ran.
7. **Review & merge** — maintainers review and squash-merge once required CI checks pass ([GOVERNANCE.md](GOVERNANCE.md)).

### Testing Expectations

| Crate | Minimum coverage expectation |
|---|---|
| `garnet-parser-v0.3` | Grammar productions + negative tests for malformed input |
| `garnet-cst` | Byte-exact round-trip + `cst_to_ast` span parity (`substrate_fidelity`) |
| `garnet-interp-v0.3` | AST node semantics + end-to-end programs + runtime trap parity |
| `garnet-vm` | Bytecode lowering + cross-backend trap parity with interpreter |
| `garnet-check-v0.3` | Violation types + transitive `@caps` coverage + annotation bounds |
| `garnet-memory-v0.3` | Property-based tests for four-tier memory primitives |
| `garnet-stdlib` | Capability-tag and stability-tag assertions + behavior tests |
| `garnet-actor-runtime` | Tell, ask, lifecycle + signed hot-reload migration |
| `garnet-convert` | Parse-verified emission (`tests/parses.rs`) + corpus parity (`tests/corpus.rs`) |
| `garnet-lsp` | Unit tests + stdio protocol smoke (`scripts/smoke_garnet_lsp_protocol.py`) |
| `garnet-wasm` | WASM unit tests + playground artifact checks |
| `garnet-cli` | Subprocess smoke tests for subcommands and manifest verification |

### Commit Message Convention

```text
type(scope): brief description

type:  feat | fix | docs | refactor | test | chore | perf | security
scope: parser | cst | interp | vm | check | memory | actor | stdlib | convert | lsp | wasm | cli | xtask | spec | examples | docs
```

Examples:
- `feat(parser): support Ruby unless and until control-flow sugar`
- `feat(cst): add typed wrappers for ActorDef and TraitDef`
- `fix(convert): preserve operator precedence in Rust binary expressions`
- `docs(contributing): document v0.8.4 workspace census and Fast Lane`

### Pull Request Checklist

- [ ] Read root `AGENTS.md` and the closest crate `AGENTS.md`
- [ ] Tests added or updated for the changed crate
- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy --workspace --all-targets --locked -- -D warnings` passes
- [ ] `cargo test -p <crate> --locked` (and `--workspace --locked` for cross-crate changes) passes
- [ ] `python3 -I scripts/garnet_trust_kernel_review_status.py --base origin/main --head HEAD` confirms expected lane
- [ ] `python3 -I scripts/garnet_text_byte_policy_status.py --gate` passes (LF-only text)
- [ ] If the spec changed, `C_Language_Specification/GARNET_v1_0_Mini_Spec.md` is updated

## Good First Contributions

Look for open issues labeled `good first issue` and `help wanted`. Great Fast-Lane entry points include:

- Adding negative parser tests and diagnostic span coverage in `garnet-parser-v0.3`
- Adding typed rowan CST node wrappers in `garnet-cst`
- Extending Rust, Ruby, Python, or Go statement lowering in `garnet-convert`
- Adding CST-backed editor capabilities (such as folding ranges) in `garnet-lsp`
- Writing compact runnable `.garnet` examples in `examples/`
- Updating guides and tutorials in `docs/` (outside `docs/why.html`)

### How to Add an Example

1. Create a `.garnet` file in `examples/` with a descriptive name and a top `//` comment explaining what it teaches.
2. Verify it parses, checks, and runs cleanly:
   ```bash
   cargo run -p garnet-cli --locked -- parse examples/<name>.garnet
   cargo run -p garnet-cli --locked -- check examples/<name>.garnet
   cargo run -p garnet-cli --locked -- run examples/<name>.garnet
   ```
3. Add a one-line entry in `examples/README.md` and open a PR with the `## Dogfood Readiness` checklist filled in.

## Reporting Bugs & Requesting Features

When reporting a bug, include:

1. Minimal `.garnet` source that reproduces the issue
2. Expected behavior (citing the Mini-Spec section for spec bugs)
3. Actual behavior (full error or command output)
4. Garnet version (`garnet --version`), OS, and Rust version (`rustc --version`)

Feature requests should describe the problem, why the proposed direction fits Garnet's Ruby × Rust × Go North Star, whether it is additive or breaking, and a short syntax sketch.

## License

By contributing to Garnet, you agree that your contributions will be licensed under both the Apache License, Version 2.0 and the MIT License (the same dual license as the project itself).

---

*"A man that hath friends must shew himself friendly." — Proverbs 18:24 (KJV)*

Garnet is dual-licensed under Apache-2.0 and MIT. See `LICENSE-APACHE` and `LICENSE-MIT`.
