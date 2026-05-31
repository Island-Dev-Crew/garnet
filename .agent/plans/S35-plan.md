# S35 Plan — source annotations (@caps): the canonical capability surface

Contract: `F_Project_Management/GARNET_v0_8_SLICE_DOGFOOD.md` → S35
(annotations → manifest → diff; first of the arc).
Branch: `codex/s35-caps-surface`.

## Finding (transparent): the @caps SYNTAX already exists
The `@caps(...)` annotation syntax is implemented (v3.4 CapCaps):
`Annotation::Caps(Vec<Capability>, Span)` (`ast.rs`), the `Capability` enum with
`from_ident`/`as_str`, `grammar::functions::parse_annotations`, transitive
propagation (`caps_graph.rs`), and per-function extraction (`CheckReport.fn_caps`).
So S35's *literal* "add the syntax" is done. The contract's actual words —
**"the surface from which the capability manifest is later derived"** — name the
genuinely-missing piece, which S36 needs and which is currently ad-hoc.

## What S35 adds (non-redundant)
1. **`garnet-check-v0.3/src/capability_surface.rs`** (new): a first-class,
   deterministic `CapabilitySurface` artifact:
   ```
   pub struct CapabilitySurface {
       pub aggregate: Vec<String>,                    // sorted, deduped union of declared caps
       pub per_function: Vec<(String, Vec<String>)>,  // sorted by fn name; caps sorted+deduped
       pub has_wildcard: bool,                         // any @caps(*) (debug-only marker)
   }
   pub fn capability_surface(module: &Module) -> CapabilitySurface
   ```
   Walks top-level `Item::Fn` annotations, maps each `Capability` via the
   canonical `Capability::as_str()` (NOT Debug), normalizes (sort + dedupe),
   aggregates. Purely syntactic (declared @caps), deterministic.
2. **Consolidate + bug-fix `garnet trust-report`**: its `collect()` builds the
   caps surface via `format!("{c:?}").to_lowercase()` (Debug) — wrong for
   `NetInternal` ("netinternal" vs canonical "net_internal"), `Other` and
   `Wildcard`. Refactor it to use `capability_surface`, fixing the mislabel and
   removing the duplicate extraction. (The existing test only checks `fs`/`net`,
   so it stays green; the fix corrects the untested variants.)

## Crates touched (writable)
- `garnet-check-v0.3`: new `capability_surface.rs` + module registration.
- `garnet-cli`: `cmd/trust_report.rs` uses the new artifact.
- `garnet-parser-v0.3` — read-only (`Capability::as_str`).

## Load-bearing dogfood
- `capability_surface` over a multi-fn module → aggregate sorted/deduped, per-fn
  sorted, wildcard flagged; `Other`/`NetInternal` use the canonical string
  (proves the bug fix); byte-deterministic across runs.
- `garnet trust-report` still reports the same fs/net surface, now via the shared
  artifact.

## End-state / gates
- No new readiness lane (not mandated for S35 by the contract lane table).
- fmt / clippy -D warnings / test --workspace / doc -D warnings / deny /
  --check-no-regression / conformance / python suites — all green.
- CHANGELOG `[Unreleased]` + contract S35 state (transparent about the
  pre-existing syntax). Dogfood bundle → PR (Navigata1) → CLI-merge
  (IslandDevCrew) → `s35` advance rides with the S36 PR.

## Honest scope / out of scope
- The @caps SYNTAX pre-exists; S35 adds the canonical surface artifact + a
  consolidation/bug-fix, not new syntax.
- Surface is top-level functions' DECLARED caps (the diff-caps surface). Actor
  methods, transitive/effective caps, and project-`[caps]`-budget enforcement
  are out of scope (later slices); S36 builds the per-program/package manifest
  artifact from this surface.
