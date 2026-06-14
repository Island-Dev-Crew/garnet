# AGENTS.md — Parser Contract

## Scope

Owns Mini-Spec v1.0 lexing, parsing, AST shape, grammar examples, and parser tests.

## Stable Contracts

- Keep grammar behavior aligned with `C_Language_Specification/GARNET_v0_3_Formal_Grammar_EBNF.md` and Mini-Spec v1.0.
- Preserve diagnostic span quality when changing lexer or parser code.
- Do not silently accept syntax that the spec does not describe unless the spec is updated in the same change.
- Examples under this crate are parser fixtures, not product demos.
- **Editions gate the lexical surface ONLY** (`edition.rs`, S32). The AST,
  checker, interpreter, and capability manifest are **edition-invariant by
  construction** — an edition may change what spelling is legal but never what
  authority a program holds (proven by the `parse_source_with_edition`
  AST-equality test + the CLI manifest-invariance test). A new edition is an
  **RFC-gated** event (GOVERNANCE.md). Keep this invariant intact: any edition
  delta lives in lexing; nothing downstream of the AST may branch on edition.

## Editions (spec note)

Parked here per the W-REBUILD RB-4b.4 reconciliation so
`GARNET_v1_0_Mini_Spec.md` stays under the maintainer's hand (same convention
as the `garnet-cst/AGENTS.md` CST note):

> **Editions** are named compatibility epochs (Rust 2015/2018/2021 style),
> decoupled from the rolling compiler version. The default and only shipped
> edition is **`v1.0`** (Mini-Spec §16.3 canonical form); a second edition
> **`v2.0`** (`Edition::Next`) is **registered only to prove the mechanism** in
> v0.8 — it reserves exactly one identifier (`async`) that is free under `v1.0`,
> and is not a shipped language version. Pin an edition in `Garnet.toml` with
> `[project]` `edition = "v1.0"`. Legacy forms warn but never break: the
> `edition = "garnet-0.3"` **value** maps to the default (`v1.0`), while the
> `[package]` **table name** is accepted with a rename warning and its pinned
> edition value is still honored (`garnet-cli/src/edition_manifest.rs`). The
> load-bearing **one-canonical-IR
> invariant** is above: editions are a *parse-time* surface only; capability
> semantics are **edition-invariant**, exactly as GOVERNANCE.md declares.
>
> **Directive-9 binding (design intent, not yet built):** the planned
> graduated-syntax *surface collapse* ships as a **new edition** with
> per-module opt-in and mix-and-match interop — old-surface modules keep
> compiling, so there is no flag-day and never a Python-3 decade. It binds to
> **this existing RFC-gated edition mechanism** rather than inventing a new
> vehicle; the collapse itself is future, RFC-gated work (no surface change
> lands with this note). Because caps are edition-invariant, the collapse can
> never alter a program's authority — only its spelling.

## Required Checks

Run parser-focused tests after changes:

```sh
cargo test -p garnet-parser
```

Run full workspace tests when grammar changes affect downstream crates.
