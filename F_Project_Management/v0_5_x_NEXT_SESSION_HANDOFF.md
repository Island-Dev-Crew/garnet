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

### Current Phase 6P status

Phase 6A added a bounded cycle-reference path before the allocator work:
`garnet-memory-v0.3/src/cycle.rs`, `garnet-memory-v0.3/tests/cycle.rs`, and
the active `deferred_arc_cycle_detection` conformance handle prove retained
roots, unrooted cycle collection, unrooted acyclic retention, and
kind-scheduled cross-kind collection. Phase 6B tightens that path with
trial-candidate and scan-black retained-candidate evidence plus a bounded
mark-gray / scan / collect-white pass. Phase 6C adds deterministic
finalization-order reporting and safe-mode affine allocation exclusion. This is
not yet production ARC. Phase 6D adds a bounded `CycleRootBuffer` so
decrement-triggered buffered roots can drive collection before the production
allocator is available. Phase 6E adds `CycleAllocatorFixture`, so an
allocator-owned surface now routes root releases and ARC edge removals through
the buffered trial-deletion path. Phase 6J starts the Tier 1 promotion by
adding an object-safe kind-aware allocator surface (`KindAllocator`,
`HeapKindAllocator`, `AllocStats`) across the four Memory Core stores and by
wiring policy-configured lazy eviction into `EpisodeStore` and `VectorIndex`.
Phase 6K adds `CycleAwareKindAllocator`, `AllocRootStats`, and object-safe root
hooks so the four stores retain observable roots on write and release them on
clear, policy eviction, workflow replacement, and drop. The next production
step is still allocator-integrated Bacon-Rajan trial deletion and runtime
finalizer invocation. Phase 6L adds a fenced `EpisodeStore::save_text` /
`load_text` persistence slice: snapshots are versioned text records with
hex-encoded payloads, writes use a sibling temp file before rename, malformed
files fail before mutating the live store, and recovered episodes retain fresh
cycle-aware roots. This is recovery evidence for the reference episodic store,
not yet a default backend by itself. Phase 6M adds
`EpisodeStore::append_text`: individual records commit to the same versioned
text format, existing logs are size-bounded and parsed as the store value type
before extension, corrupt, empty, type-invalid, or oversized logs are not
carried forward, projected oversize commits are rejected before file creation,
accepted record data is synced through a temp-file rewrite and rename, and the
live store mutates only after the on-disk commit succeeds. Phase 6N adds the
default typed episodic cache backend boundary:
`EpisodeStore::append_cache_text` / `load_cache_text` use fixed
`.garnet-cache/episodic/episodes.mnemos` storage under a canonical project
root, reject symlinked or non-regular targets, reject oversized loads before
allocation, serialize rewrite-based access with an OS-backed lockfile on
Unix/Windows, anchor Unix backend file operations to the validated episodic
directory handle, keep Unix cache dirs/files private from creation time, and
preserve corrupt/type-invalid non-mutation plus
cycle-aware root rehydration. This remains distinct from the CLI signed NDJSON
advisory cache and must not be treated as trusted compiler input without a
future MAC layer. Phase 6O adds the first directory-sync durability guardrail
for that text-commit family: after an accepted temp-file rewrite and rename,
generic Unix text commits sync the parent directory and the typed cache backend
syncs the already-validated episodic directory handle. Non-Unix platforms
retain file-data sync until a platform-specific directory-sync contract is
added. Phase 6P adds a dependency-free source-tree binding line to the typed
cache format; appends validate existing bindings before rewrite, and loads
reject copied `episodes.mnemos` files from another canonical project root
before replacing live episodic memory. This is replay-hardening evidence, not a
cryptographic MAC.

### Current Phase 6F-6I cache-security status

Phase 6F adds a narrower security/readiness gate around the compiler-as-agent
cache. `parse`, `check`, and `run` now persist privacy-preserving episode file
labels: absolute paths under the project root become stable relative labels,
and absolute paths outside the project root become `<external>/<file>`.
Phase 6G adds CLI-level replay stress: a foreign machine-key episode in the
same cache and a copied `.garnet-cache` replay are ignored, counted, and
surfaced as untrusted before stale prior-failure advice can appear. Existing
CacheHMAC and ProvenanceStrategy tests remain green. Phase 6H wires CLI
strategy notes through provenance verification: copied same-machine
`strategies.db` rows with missing local justifying episodes are quarantined
instead of printed as applicable strategies, and bounded concurrent episode
append stress preserves all verified records. Phase 6I binds episodes to a
keyed, non-reversible source-tree identifier, skips copied same-machine cache
records from a different project root, quarantines copied same-machine strategy
rows whose replayed justifications no longer verify in the current source tree,
and adds a 16-writer/1920-record bounded append soak. The next cache security
slice should add extended release-duration/cross-platform soak if needed before
release.

### Why deferred

v0.4.2 locked in the **naming** (Memory Core / Mnemos) and the
**roadmap** (`C_Language_Specification/MEMORY_CORE_ROADMAP.md`).
Tier 0 (the original reference stores) ships. Phase 6J begins Tier 1:
a kind-aware allocator trait that the four stores delegate to, plus
policy-configured eviction enforcement for episodic and semantic stores. Phase
6K continues Tier 1 by proving observable store-root lifecycles through the
cycle-aware allocator adapter. Phase 6L starts the persistence track with a
bounded episodic text snapshot API so save/load recovery can be tested without
claiming full production persistence. Phase 6M adds guarded append-style text
log commits as the next recovery guardrail. Generics over memory kinds and
allocator-integrated ARC remain pending.

### Tier 1 scope (per the Roadmap)

#### T1.1 — Kind-aware allocator trait

Phase 6J lands this as an object-safe allocator surface in
`garnet-memory-v0.3/src/alloc.rs`. It intentionally records typed allocation
intent through `AllocRequest` / `AllocStats` instead of exposing a generic
`allocate<T>` method on the trait, because generic trait methods are not object
safe and would prevent stores from carrying `Arc<dyn KindAllocator>`.

The four reference stores now carry a `KindAllocator` field with a
`HeapKindAllocator` default, preserving `new()` / `Default` compatibility while
making allocation stats observable.

#### T1.2 — Eviction policy enforcement

Phase 6J wires `MemoryPolicy::score` and `should_retain` into
`EpisodeStore::with_policy` and `VectorIndex::with_policy`. Eviction is lazy at
read/search time, so no background thread or synchronization surface is added.
Default constructors preserve the prior unbounded reference behaviour.

Covered property tests: capping an episodic store at N entries converges to N
within a finite read, semantic search keeps top policy matches under high-water
pressure, and low-relevance semantic facts are dropped when policy thresholds
require it.

#### T1.3 — Cycle-aware store-root lifecycle

Phase 6K adds `CycleAwareKindAllocator` and `AllocRootStats`. Stores retain
cycle-aware roots when values enter working, episodic, semantic, and procedural
memory and release those roots on `WorkingStore::clear`, episodic/semantic
policy eviction, procedural workflow replacement, and store drop. This is the
observable root-lifecycle proof, not the production ARC finalizer path.

Covered property tests: all four stores record created/active roots, working
clear releases roots, episodic and semantic policy eviction release roots,
procedural replacement releases the previous root, and dropping each store
releases any remaining roots.

#### T2.3 first slice — Episodic text snapshot persistence

Phase 6L lands a deliberately small persistence boundary on `EpisodeStore`.
`save_text` writes `garnet-episodic-v1` snapshots with
`timestamp<TAB>hex(value.to_string())` records; `load_text` parses the whole
file before replacing in-memory events, so malformed files are loud and
non-mutating. Loading rehydrates allocator roots, which keeps Phase 6K lifecycle
evidence intact across recovery. The larger Tier 2 target is still a pluggable
append-only backend.

Covered tests: delimiter-control payload round-trip, cycle-aware root release
on drop plus rehydration on load, and malformed-file rejection without store
mutation.

#### T2.3 guardrail — Append-style episodic text log commits

Phase 6M adds `EpisodeStore::append_text` as a narrow incremental write path on
the Phase 6L text format. It size-bounds and parses any existing log as the
store value type before extension, refuses corrupt, empty, type-invalid, or
oversized logs without carrying the file forward or mutating the live store,
rejects projected oversize commits before file creation, syncs accepted record
data through a temp-file rewrite and rename, and then routes the value through
`append_at` so allocator stats and root retention stay observable.

Covered tests: append-style log recovery with delimiter-control payloads,
corrupt/empty/type-invalid log rejection without file extension or live-store
mutation, and oversized-log/projected-oversize rejection before live-store
mutation.

#### T2.3 backend boundary — Default typed episodic cache file

Phase 6N binds the Phase 6M text format to fixed per-project
`.garnet-cache/episodic/episodes.mnemos` storage through
`episodic_cache_log_path_for`, `EpisodeStore::append_cache_text`, and
`EpisodeStore::load_cache_text`. The backend canonicalizes the project root,
creates private cache directories, rejects symlinked/non-regular backend
targets, checks load size before reading into memory, serializes rewrite-based
loads/appends with an OS-backed `episodes.mnemos.lock` on Unix/Windows, and keeps
backend files private on Unix. Missing backend logs load as empty; malformed, corrupt, oversized, and
wrong-type logs fail before live-store mutation.

Covered tests: fixed backend path, backend append/load round trip with
delimiter-control payloads, cycle-aware root rehydration, corrupt backend
rejection, wrong-type backend rejection, concurrent appends preserving all
records, Unix `0700`/`0600` permissions, symlinked cache-directory rejection,
and oversized `load_text` rejection before live-store mutation.

#### T1.4 — Generics over memory kinds (Mini-Spec §4.4)

This is the gnarliest of the three. Phase 5B gives §11.6 interpreter-level
generic instantiation evidence, but native monomorphization is still deferred. The
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

- `garnet-memory-v0.3/src/alloc.rs` — `KindAllocator`, `HeapKindAllocator`,
  `CycleAwareKindAllocator`, `AllocStats`, and `AllocRootStats`.
- `garnet-memory-v0.3/src/lib.rs` — public Memory Core exports and module
  docs.
- `garnet-memory-v0.3/src/{working,episodic,semantic,procedural}.rs`
  — accept the allocator, route allocations and observable roots through it.
- `garnet-memory-v0.3/tests/persistence.rs` — episodic text snapshot
  round-trip, malformed-file rejection, root rehydration, and append-style
  text log guardrail tests.
- `garnet-memory-v0.3/src/policy.rs` — already has `score` /
  `should_retain`; no changes needed there.
- `garnet-memory-v0.3/tests/properties.rs` — allocator stats, eviction
  convergence, cycle-aware root lifecycle, and drop-release property tests.
- `C_Language_Specification/GARNET_v0_4_2_Conformance_Matrix.md` —
  rename to `GARNET_v0_5_0_Conformance_Matrix.md` (or update in
  place); flip §4.4 / §4.5 rows as items land.
- `C_Language_Specification/MEMORY_CORE_ROADMAP.md` — flip Tier 1
  rows as items land. **Do this in the same commit** that lands the
  work, per the policy at the bottom of the Roadmap.

### Remaining pre-requisites

- T1.1, T1.2, T1.3, the first T2.3 episodic snapshot slice, the Phase 6M
  append-style text log guardrail, and the Phase 6N default typed cache
  backend boundary are now self-contained Mnemos evidence slices.
- T1.4 wants Mini-Spec §11.6 monomorphization to actually
  monomorphize. Today it is parsed-only (see Conformance Matrix
  §11.6 row). If you can punt the language-level syntax, the
  library-side trait can land first; the syntax follows.

### Remaining estimate

T1.4 in a separate session, probably alongside parser work. Production ARC
finalizers and broad pluggable persistence stay separate from the Tier 1
adapter, Phase 6L reference-snapshot evidence, Phase 6M append guardrail, and
Phase 6N typed cache backend boundary.

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
