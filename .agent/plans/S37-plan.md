# S37 Plan — diff-caps (capability-surface diff as acceptance gate)

Contract: `F_Project_Management/GARNET_v0_8_SLICE_DOGFOOD.md` → S37 (the headline
novelty; third of annotations → manifest → diff).
Branch: `codex/s37-diff-caps`.

## Goal
Diff the declared capability surface between two revisions and **gate on
authority changes**. **[GRAFT]** feeds the **same fused 1–5 band** as Greptile in
S33's `garnet verify` (`min` governs the gate) — completing the
`CapabilitySignal` slot S33 stubbed.

## Design
- **`garnet-check-v0.3/src/caps_diff.rs`** (new): pure diff over two
  `CapabilitySurface`s.
  ```
  pub struct CapsDiff {
      aggregate_added: Vec<String>, aggregate_removed: Vec<String>,
      functions_added: Vec<String>, functions_removed: Vec<String>,
      functions_caps_expanded: Vec<(String, Vec<String>)>,  // fn -> caps gained
      wildcard_introduced: bool,
  }
  impl CapsDiff { pub fn authority_expanded(&self) -> bool }  // program gained authority:
      // any aggregate_added OR wildcard_introduced (a fn re-declaring a cap already
      // in the aggregate is NOT new program authority).
  pub fn diff_caps(old: &CapabilitySurface, new: &CapabilitySurface) -> CapsDiff
  ```
  Deterministic (sorted). Unit tests incl. added/removed/expanded/wildcard +
  the "removed-only is not expansion" case.
- **`garnet-cli`**:
  - `cap_manifest::surface_for_path(path) -> Result<CapabilitySurface, String>`:
    edition-aware (S32) parse + `capability_surface` (S35) over a file/dir,
    merged. `cmd/caps.rs` refactored onto it (DRY).
  - `verify_gate::capability_band(&CapsDiff) -> Band`: `5` if no authority
    expansion, `2` if expanded (so the fused `min` caps merge confidence and
    flags review).
  - `cmd/diff_caps.rs`: `garnet diff-caps <old-path> <new-path>` — surfaces →
    `diff_caps` → print added/removed/verdict/band; **exit 1 if
    authority_expanded**, else 0.
  - **Complete the S33 graft:** `garnet verify <path> [--caps-baseline <old>]`.
    With a baseline, compute `diff_caps(base, current)` →
    `CapabilitySignal::Surface(capability_band)` feeding the fuse; without it,
    stays `Pending` (back-compat).
- **Readiness lane `capability_diff_caps`** (committed-truth) + baseline regen.

## Crates touched
- `garnet-check-v0.3`: new `caps_diff.rs` + re-export.
- `garnet-cli`: `cap_manifest.rs` (`surface_for_path`), `cmd/caps.rs` (refactor),
  `verify_gate.rs` (`capability_band` + baseline wiring), `cmd/diff_caps.rs` (new),
  dispatcher + `print_help`, new tests.

## Load-bearing dogfood
- `diff-caps old new` where `new` adds a cap → exit 1, "authority expanded", band 2.
- `new` only REMOVES a cap → exit 0, band 5 (reduction is not expansion).
- identical surfaces → exit 0, band 5, empty diff.
- `garnet verify <tree> --caps-baseline <old>` where the tree gained a cap →
  fused band capped at 2 (the capability signal is now live, not stubbed).

## End-state / gates
- Lane `capability_diff_caps` added + baseline regenerated (headline bumps).
- fmt / clippy -D / test --workspace / doc -D / deny / --check-no-regression /
  conformance / python — all green. CHANGELOG + contract S37 state. Dogfood
  bundle → PR → CLI-merge → `s37` advance rides with the S38 PR.

## Honest scope / out of scope
- diff-caps reads the **declared** capability surface; it does NOT prove the
  absence of undeclared authority (that is the sandbox-policy job, S46).
- "Two revisions" = two source paths the caller supplies (e.g., two git
  checkouts / `git worktree`); S37 does not itself drive git.
