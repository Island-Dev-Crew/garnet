# Garnet v0.5.x — Next-Session Handoff

**As of:** 2026-04-26 (end of v0.4.2 release-readiness sequence).
**Last commit on `main`:** the seventh of seven release-readiness refactors (`garnet fmt` + `garnet doc` MVPs).
**Audience:** the next assistant or contributor opening this repo cold.

---

## Read this first

The v0.4.2 release-readiness sequence is complete. The `main` branch
is shippable: `cargo fmt --check`, `cargo clippy -D warnings`,
`cargo test --workspace`, and `RUSTDOCFLAGS=-D warnings cargo doc
--workspace` are all green; CI runs the same gates on push (see
`.github/workflows/ci.yml` + `security.yml` + `codeql.yml` +
`linux-packages.yml`).

Two large work items were intentionally **deferred** to v0.5.x rather
than rushed into v0.4.2. Both are scoped below with enough context
that you can pick either one up cold.

---

## Item A — `garnet-lsp` (Language Server Protocol)

### Why deferred

The v0.4.2 sweep landed `garnet fmt` (whitespace-only) and
`garnet doc` (markdown extraction). Both are MVP — full versions of
either gate on a *trivia-preserving CST* in the parser. A real LSP
would gate on the same CST plus incremental reparse, position
indexing, and diagnostic streaming. Ramming that into the v0.4.2
window would have meant either a half-built LSP or a half-built fmt;
neither is responsible.

### What "MVP LSP" means for Garnet

A v0.5.0 LSP with these capabilities is the credible target:

1. **`textDocument/didOpen` / `didChange` / `didSave`** — accept files,
   keep an in-memory parse tree.
2. **`textDocument/publishDiagnostics`** — surface parser + checker
   errors in real time. Reuse `garnet-parser-v0.3` (parser errors) +
   `garnet-check-v0.3` (caps / borrow / audit) + `garnet-stdlib`
   (capability registry) — all the diagnostic content already exists.
3. **`textDocument/hover`** — for an identifier under cursor, surface
   its kind (memory unit / actor / fn / def / struct / etc.) and any
   `///` doc comment. Reuses the `extract_doc_comments_before` helper
   from `garnet-cli/src/cmd/doc.rs`.
4. **`textDocument/definition`** — go-to-definition for symbols
   resolvable from the AST without a full type system. Stretch goal
   for v0.5.0; the basic case (function/struct/enum) is doable;
   trait method dispatch needs §11.5 trait coherence work.
5. **`textDocument/formatting`** — call into `garnet_cli::cmd::fmt`'s
   `normalize` function. Trivial because that function is already a
   pure `fn(&str) -> String`.

### Where to put it

A new workspace crate: **`garnet-lsp/`** (lib + bin), added to
`Cargo.toml` `[workspace] members`. Should NOT live inside
`garnet-cli` — keeps the binary surface minimal and lets editors
install just the LSP without pulling the full CLI.

### Recommended dependencies

- `tower-lsp` — modern async LSP framework, well-maintained.
- `tokio` — async runtime (already a transitive dep elsewhere; verify
  it lands in `cargo deny check` cleanly).
- Existing workspace crates: `garnet-parser`, `garnet-check`,
  `garnet-stdlib`, optionally `garnet-interp` for `eval`-on-hover
  (stretch).

### Gating issue: CST first

The honest sequence is:

1. **Parser CST layer.** Today `garnet-parser-v0.3` produces an AST
   that drops trivia. To do anything position-sensitive (LSP hover,
   formatter rewrites, doc-comment-on-AST-node), the parser needs to
   either:
   - emit a CST in parallel (rowan-style — recommended), or
   - keep a trivia table indexed by token offset.
2. **`garnet-lsp` MVP** (the five capabilities above).
3. **Upgrade `garnet fmt`** from whitespace-only to
   AST-driven once the CST exists. `garnet doc` upgrade follows.

Skipping step 1 buys you a brittle LSP that will be hard to grow.

### Reference points in the existing code

- Parser entry point: `garnet_parser::parse_source` in
  `garnet-parser-v0.3/src/lib.rs`.
- AST shape: `garnet-parser-v0.3/src/ast.rs` (every node has a `Span`).
- Checker entry point: `garnet_check::check_module` in
  `garnet-check-v0.3/src/lib.rs`.
- Existing dispatcher pattern to mirror: `garnet-cli/src/cmd/*.rs`.

### Estimate

1–2 weeks for MVP, assuming the CST work is done first (which is
itself probably a week). Plan accordingly; do not promise a
two-day LSP.

---

## Item B — Memory Core Tier 1 (Mnemos production allocator integration)

### Why deferred

v0.4.2 locked in the **naming** (Memory Core / Mnemos) and the
**roadmap** (`C_Language_Specification/MEMORY_CORE_ROADMAP.md`).
Tier 0 (today's reference stores) ships. Tier 1 is the first
productization step: a kind-aware allocator trait that the four
stores delegate to, eviction policy enforcement, and generics over
memory kinds. Doing it well takes a focused session; doing it
sloppily would entrench the wrong allocator surface.

### Tier 1 scope (per the Roadmap)

#### T1.1 — Kind-aware allocator trait

Define the following in `garnet-memory-v0.3/src/lib.rs` or a new
`alloc.rs`:

```rust
/// Allocator strategy, parameterized by the memory kind. Stores
/// delegate every backing allocation to an instance of this trait.
pub trait KindAllocator: Send + Sync {
    fn kind(&self) -> MemoryKind;
    /// Allocate enough space for `n` items of size `size_of::<T>()`,
    /// alignment `align_of::<T>()`. Returns a typed slab the store
    /// owns. Implementations should respect kind-specific retention.
    fn allocate<T: 'static>(&self, n: usize) -> Box<[std::mem::MaybeUninit<T>]>;
    /// Reset the entire allocator (working: drop arena; episodic:
    /// rotate log; semantic: clear index; procedural: drop history).
    fn reset(&self);
    /// Optional: report allocator-level stats.
    fn stats(&self) -> Option<AllocStats> { None }
}
```

The four reference stores get a `KindAllocator` field:

```rust
pub struct WorkingStore<T> {
    arena: RefCell<Vec<T>>,
    alloc: Box<dyn KindAllocator>,  // NEW
}
```

with a `Default` impl (for backwards compat) that uses a built-in
`HeapKindAllocator` matching today's behaviour. Existing tests
should pass unchanged.

#### T1.2 — Eviction policy enforcement

Today `MemoryPolicy::score` and `should_retain` exist but are never
called. Wire them into actual eviction loops in `EpisodeStore` and
`VectorIndex`. Recommended approach: lazy eviction at read time —
when `query_top_k` or `recall_recent` runs, evict any item where
`should_retain(score) == false`. Cheaper than background sweep, no
extra threads, no synchronization.

Add a property test: capping a store at N entries with random
inserts converges to ≤ N entries within finite reads.

#### T1.3 — Generics over memory kinds (Mini-Spec §4.4)

This is the gnarliest of the three. Today the Mini-Spec explicitly
defers it because §11.6 monomorphization is parsed-only. The
realistic v0.5.0 path:

1. Add `<Kind: MemoryKindTrait>` parameter to `MemoryHandle` and the
   four stores.
2. Move §4.4 from 🟠 to 🟡 in the conformance matrix —
   "monomorphizable interface present; full kind-generic library
   patterns gate on §11.6."
3. Accept that the language-side `memory<Kind>` syntax will need
   parser work — file as a separate ticket against
   `garnet-parser-v0.3`.

### Files to touch

- `garnet-memory-v0.3/src/lib.rs` — `KindAllocator` trait, default
  impl.
- `garnet-memory-v0.3/src/{working,episodic,semantic,procedural}.rs`
  — accept the allocator, route allocations through it.
- `garnet-memory-v0.3/src/policy.rs` — already has `score` /
  `should_retain`; no changes needed there.
- `garnet-memory-v0.3/tests/properties.rs` — add eviction-convergence
  property test.
- `C_Language_Specification/GARNET_v0_4_2_Conformance_Matrix.md` —
  rename to `GARNET_v0_5_0_Conformance_Matrix.md` (or update in
  place); flip §4.4 / §4.5 rows as items land.
- `C_Language_Specification/MEMORY_CORE_ROADMAP.md` — flip Tier 1
  rows as items land. **Do this in the same commit** that lands the
  work, per the policy at the bottom of the Roadmap.

### Pre-requisites (do these first)

- None for T1.1 / T1.2 — they are self-contained inside Mnemos.
- T1.3 wants Mini-Spec §11.6 monomorphization to actually
  monomorphize. Today it is parsed-only (see Conformance Matrix
  §11.6 row). If you can punt the language-level syntax, the
  library-side trait can land first; the syntax follows.

### Estimate

T1.1 + T1.2 in a focused session. T1.3 in a separate session
(probably alongside parser work).

---

## Item C — Smaller follow-ups (free-floating)

These are not load-bearing for v0.5.0 but are worth picking up
opportunistically:

- **Drop the deprecated `ActorAddress::ask`.** Currently `#[deprecated
  since = "0.4.0"]`. Remove in v0.5.0. All internal callsites are
  already on `try_ask`.
- **Wire signed `SHA256SUMS`.** TODO comment is in
  `.github/workflows/linux-packages.yml`. Needs `GPG_SIGNING_KEY` +
  `GPG_PASSPHRASE` repo secrets provisioned.
- **SLSA build-provenance attestation.** One-line addition once
  signing is wired (see end of refactor #1's deferred-list).
- **Update existing example `.garnet` files** to use the current
  grammar. `examples/mvp_01_os_simulator.garnet` uses `:` as a map
  literal separator instead of `=>` and fails to parse — stale
  syntax from before grammar v0.3.
- **Move `[workspace.package]` inheritance into member crates.** The
  workspace declares `version`, `edition`, `license`, `authors`,
  `repository` but no member crate uses `version.workspace = true`
  yet. Optional cleanup.

---

## Things NOT to do

- **Do not** rewrite the v0.4.2 fmt or doc commands to "do more"
  before the CST exists. They are honest at MVP scope; growing them
  with hacks creates a worse problem than leaving them alone.
- **Do not** rename Memory Core / Mnemos. The naming is locked in
  across the lib doc-header, conformance matrix, roadmap, README,
  CLI version output, and crate Cargo.toml description. Six places.
  Renaming costs more than any potential improvement.
- **Do not** add new dependencies without running `cargo deny check`
  first. The license allow-list in `deny.toml` was derived from a
  one-time audit; new deps may bring new licenses that need either
  a config update or a different crate choice.
- **Do not** force-push to `main`. The repo is `Island-Dev-Crew/garnet`
  and the only acceptable destructive operation is reverting via a
  new commit.

---

## Sanity checklist before any commit on `main`

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --no-fail-fast
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
```

All four must be clean. CI runs the same gates and will catch
regressions, but local clean is faster than a CI rebuild cycle.
