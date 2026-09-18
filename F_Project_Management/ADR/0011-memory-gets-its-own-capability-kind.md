# ADR 0011 — Memory tiers get their own capability kind

**Status.** Accepted (2026-09-17). Implemented 2026-09-18 (D-04): `RequiredCaps::mem()`, four `memory::*` rows (`Guard::GateEntry`, `Stability::Experimental`), `CapSet::MEM` between `fs` and `net`, both runtime backstops in `bridge_memory_kind`, and the `BRIDGE_ONLY` const removed; pinned by `garnet-cli/tests/check_memory_capability.rs` and the D-04 section of `caps_enforcement.rs`. Extended 2026-09-18 (D-04b) after independent review found that a first-class `memory <kind> <name> : <type>` declaration allocated a tier outside the registry with no gate: declarations (top-level, module, actor) are now gated at load by the same `mem` entry/call-chain pair and charged to `main` by the checker.

## Context

A Garnet program can declare all four memory tiers, construct them and write to
them while `garnet caps` reports an empty capability set. The four memory
natives are bridged directly into the interpreter and carry no row in the
standard library registry, so the checker, the manifest and `diff-caps` cannot
see memory at all. Memory Core is therefore a Rust crate with reserved words
rather than a part of the language's evidence story.

## Decision

Memory gets its own capability kind, `mem`, with a registry row for each of the
four tier constructors, gated at the runtime entry the way the other twenty
gated primitives are (the gated surface becomes 24). A tier then appears in `garnet caps`, an undeclared tier
traps identically on the interpreter and the virtual machine, and adding one
moves a `diff-caps` verdict.

The rule covers every way a program comes to own a tier: constructing one
through a `memory::*` native **or** declaring one with the `memory` keyword.
Declaring is not a way around the capability.

## Alternatives rejected

- **Map memory onto the file capability.** It would let a program that declares
  file access reach memory, and a reader could not tell the two apart.
- **Leave memory uncapped.** Memory Core stays a library that happens to have
  keywords, and the strongest sentence available to Garnet stays unsayable.
