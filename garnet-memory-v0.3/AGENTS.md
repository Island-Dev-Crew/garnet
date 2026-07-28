# AGENTS.md — Memory Core Contract

## Scope

Owns the reference implementation for Garnet's working, episodic, semantic, and procedural memory abstractions.

## Stable Contracts

- Preserve the four memory-kind boundary: working, episodic, semantic, procedural.
- Keep the cycle-collection reference path honest: `cycle.rs` is a bounded
  trial-deletion fixture/model for Mini-Spec §4.5 behavior, including
  root-buffer scheduling, finalization-order, and safe-mode exclusion signals,
  not the production allocator-integrated ARC collector.
- Keep `CycleAwareKindAllocator` honest: it is observable root-lifecycle
  evidence that store writes, clear, policy eviction, replacement, and drop can
  drive the bounded cycle fixture. It is not a production ARC finalizer or
  persistence backend.
- Keep isolated-root teardown linear: the cycle graph maintains exact incoming
  managed-ARC edge counts so releasing a store-owned root with no incoming ARC
  edge rejects it as a trial-deletion candidate in O(1). Rooted-reachability
  scans remain required for nodes that actually have incoming ARC peers.
- Treat `AGENTS.md` and workflow contracts as procedural-memory analogs when designing future tooling.
- Never hide sink, persistence, or machine-key failures; memory failures must be observable.
- Keep tests isolated from machine-local key races and cache state.
- Keep the typed episodic cache backend narrow and honest:
  `EpisodeStore::{append_cache_text,load_cache_text}` owns only
  `.garnet-cache/episodic/episodes.mnemos` under a caller-provided project
  root. It must keep symlink/non-regular path rejection, pre-read size bounds,
  OS-backed rewrite serialization on Unix/Windows, and private Unix
  permissions intact. Unsupported platforms should fail loudly instead of
  falling back to ad hoc sentinel locks. It is not the CLI signed NDJSON
  advisory cache and must not be treated as trusted compiler input without an
  explicit MAC/source-tree binding layer.

## Required Checks

```sh
cargo test -p garnet-memory --test cycle
cargo test -p garnet-memory
cargo test -p garnet-cli cache
```

Run workspace tests if cache or machine-key behavior changes.
