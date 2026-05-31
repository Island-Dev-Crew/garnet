# S36 Plan — capability manifest (derived from annotations)

Contract: `F_Project_Management/GARNET_v0_8_SLICE_DOGFOOD.md` → S36
(annotations → manifest → diff; second of the arc).
Branch: `codex/s36-capability-manifest`.

## Goal
Derive a per-program / per-package **capability manifest** from S35's
`CapabilitySurface`. The manifest is the artifact `diff-caps` (S37) compares
across revisions and `seal` (S38) embeds. Distinct from the build `Manifest`
(`manifest.rs`), which carries source/ast hashes but no capability surface.

## Design (bounded)
- **`garnet-cli/src/cap_manifest.rs`** (new):
  - `CapabilityManifest { schema, surface: CapabilitySurface }`, `SCHEMA =
    "garnet-capability-manifest-v1"`, `from_surface(...)`.
  - `to_json(&self)` — deterministic JSON `{schema, aggregate, functions:
    [{name, caps}], wildcard}`, reusing `diagnostics::json_escape` (S34).
  - `merge_surfaces(Vec<CapabilitySurface>)` — package mode: union aggregate,
    sorted+deduped functions (`BTreeSet<(name, caps)>`), OR-ed wildcard.
- **`garnet caps <path>`** (`cmd/caps.rs`): edition-aware parse (S32) +
  `capability_surface` (S35) over a file (per-program) or every `.garnet` under
  a dir (per-package, via `merge_surfaces`); emit the manifest JSON; exit 0 on
  success, 1 on parse/IO error, 2 on usage.
- Reuse: `verify_gate::collect_targets` and `diagnostics::json_escape` promoted
  to `pub(crate)` (DRY — no duplicate walk / escape).

## Crates touched (writable)
- `garnet-cli`: new `cap_manifest.rs` + `cmd/caps.rs`, dispatcher + `print_help`,
  `pub(crate)` on the two reused helpers.
- `garnet-check-v0.3`, `garnet-parser-v0.3` — read-only (consume S35 surface).

## Load-bearing dogfood
- `garnet caps <file>` → deterministic JSON with `schema`, sorted `aggregate`,
  per-function `caps`, `wildcard`. Byte-identical across runs (unit test).
- `garnet caps <dir>` → package aggregate (union of all files' caps).
- The manifest round-trips the S35 surface (same aggregate/functions).

## End-state / gates
- No new readiness lane (the contract lane table mandates lanes at S37/S38, not S36).
- fmt / clippy -D / test --workspace / doc -D / deny / --check-no-regression /
  conformance / python suites — all green.
- CHANGELOG `[Unreleased]` + contract S36 state. Dogfood bundle → PR (Navigata1)
  → CLI-merge (IslandDevCrew) → `s36` advance rides with the S37 PR.

## Honest scope / out of scope
- The manifest captures the DECLARED capability surface (S35); it does not prove
  the absence of undeclared authority (that is the sandbox job, S46) and does not
  yet enforce the project `[caps]` budget.
- Per-package per-function entries are concatenated+deduped by `(name, caps)`;
  cross-file same-name functions both appear (honest surface, not a resolver).
- S37 `diff-caps` consumes two of these manifests; S38 `seal` embeds one.
