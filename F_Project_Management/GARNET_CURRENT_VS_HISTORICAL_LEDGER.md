# Garnet Current vs Historical Ledger

Date: 2026-05-08

This ledger prevents historical handoffs from being misread as current
implementation truth.

## Reading Rule

When documents conflict, use this order:

1. Live command output from the current checkout.
2. `CURRENT_STATE.md`.
3. `README.md`.
4. `C_Language_Specification/GARNET_v0_4_2_Conformance_Matrix.md`.
5. Current CI workflow definitions.
6. Historical handoffs and archived milestone files.

## Current Proof Surfaces

| Claim surface | Current source | Verification |
|---|---|---|
| Workspace builds/tests | `Cargo.toml`, CI, command output | `cargo test --workspace --no-fail-fast` |
| Canonical app examples | `examples/mvp_*.garnet` | parse/check/run loop and `garnet-cli/tests/examples.rs` |
| First-user templates | `garnet-cli/templates/*` | `garnet new`, `garnet test`, `garnet run` |
| Language completeness | conformance matrix | implemented/partial/deferred rows |
| Release install path | README, installer, release workflow | source fallback works; native release assets require release publication |

## Historical or Descriptive Surfaces

| Surface | How to interpret it |
|---|---|
| `archive/history/_CANONICAL_DELIVERABLES_INDEX.md` | historical bundle index from the research/package era |
| `archive/history/GARNET_BUILD_INSTANTIATION_BRIEF.md` | early build-orchestration plan, not current repo setup |
| `archive/history/GARNET_PHASE7_COMPLETION_REPORT.md` | parser-phase milestone report, not current language-completeness evidence |
| `archive/examples/mvp-design-drafts/` | pre-remediation MVP application drafts, not current runtime proof |
| `F_Project_Management/GARNET_v3_2_HANDOFF.md` | historical snapshot; some example/check claims are superseded by current examples README and tests |
| `F_Project_Management/GARNET_v4_0_HANDOFF.md` | historical milestone; MVP scale claims must be checked against current runnable examples |
| `F_Project_Management/GARNET_v4_2_COMPLETE_PROJECT_STATE.md` | useful consolidated history, but path and release claims require live verification |

## Current Corrections From Dogfood Part 2

- The 10 canonical MVP examples are now compact executable smokes, not the old
  large design drafts.
- The three larger real-world examples remain parser-scale design drafts.
- The `agent-orchestrator` template now uses managed actor addresses, bounded
  mailbox calls, and actor-local memory while still succeeding on the current
  interpreter.
- Safe-mode borrow checking now has an active CLI conformance slice for direct
  `own` use-after-move, direct `mut`+`borrow` aliasing, and unambiguous
  `own self` method receiver moves. It now uses simple declared receiver types
  to distinguish same-named impl methods, and it tracks simple field places for
  same-field/parent-child aliasing plus field use-after-move. It also tracks
  indexed places conservatively as wildcard sub-places, so indexes under the
  same receiver conflict while indexes under distinct sibling fields remain
  distinct, and nested index operand expressions stay checked. It now also
  enforces a conservative lifetime-elision subset for reference returns: no
  borrowed input and multiple borrowed inputs reject, while one borrowed input
  is accepted. Phase 4G adds a B5 drop-discipline slice that rejects overlapping
  places passed to multiple `own` parameters in one call, preventing same-call
  double-drop hazards for the same binding or parent/child place. Phase 4H adds
  direct-returning branch liveness: moves in branch bodies that return from the
  function no longer poison later code on paths that can still continue. Phase
  4I adds direct-return block and loop-body liveness: statements after a direct
  `return` are not borrow-checked, and moves inside direct-returning
  `while`/`loop` bodies no longer poison paths after the loop that only exist
  when the body does not run. Full Rust-grade CFG NLL, nested/non-local
  terminators, general loop fixed-point analysis, for-loop liveness, dynamic
  places, broader drop elaboration, and generic/trait impl dispatch remain
  roadmap work, not current truth.
- Trait coherence now has an active conservative checker slice: exact duplicate
  trait impls and orphan impls where neither trait nor type is local reject,
  simple generic blanket-vs-concrete and renamed blanket impl overlaps reject,
  and qualified external type paths no longer pass by short-name collision,
  while local-trait, local-type, and qualified local-module impls remain
  accepted. Specialization, imported-package coherence, and native
  monomorphization remain roadmap work.
- Generic instantiation now has interpreter-level evidence for generic struct
  construction, generic impl method dispatch, and generic function calls. This
  is not native monomorphization or a zero-cost backend guarantee.
- Memory Core ARC/cycle work now has Phase 6E executable reference evidence:
  `CycleGraph`, `CycleRootBuffer`, and `CycleAllocatorFixture` fixtures expose
  decrement-triggered buffered roots, allocator-owned root/edge decrement
  scheduling, threshold-driven collection, trial candidates, scan-black
  retained candidates, deterministic collect-white finalization order,
  safe-mode affine exclusion from ARC trial candidates, rooted retention,
  unrooted cycle collection, unrooted acyclic retention for ordinary
  retention/eviction, and kind-partitioned cross-kind scans. Phase 6J adds a
  kind-aware allocator surface to all four stores and makes policy-configured
  episodic/semantic stores perform lazy eviction on read/search. Phase 6K adds
  `CycleAwareKindAllocator` and verifies observable store-root retain/release
  lifecycles on write, clear, policy eviction, workflow replacement, and store
  drop. Phase 6L adds fenced `EpisodeStore` text snapshot save/load with
  delimiter-safe payload encoding, malformed-file non-mutation, and
  cycle-aware root rehydration. Phase 6M adds guarded append-style text log
  commits that size-bound and validate existing logs as the store value type
  before extension and avoid live-store mutation on corrupt, empty,
  type-invalid, or oversized persistence files. Phase 6N adds a fixed typed
  episodic cache backend at `.garnet-cache/episodic/episodes.mnemos` with
  canonical project-root pathing, symlink/non-regular rejection, pre-read size
  bounds, OS-backed lockfile serialization on Unix/Windows, private Unix permissions,
  concurrent append preservation, and cycle-aware root rehydration. This is not the production
  allocator-integrated Bacon-Rajan collector, runtime finalizer path, broad
  pluggable persistence backend, or CLI signed NDJSON advisory-cache trust
  layer.
- Compiler-as-agent cache privacy now has Phase 6F executable evidence:
  absolute paths inside the active project are persisted as stable relative
  labels, while external absolute paths are redacted to `<external>/<file>`.
  Phase 6G extends this to CLI-level replay stress: same-cache foreign
  machine-key episodes and copied `.garnet-cache` episodes are ignored,
  counted, and warned as untrusted instead of surfacing stale prior-failure
  advice. Phase 6H wires CLI strategy notes through ProvenanceStrategy, so
  copied same-machine `strategies.db` rows with missing local justifying
  episodes are quarantined before they can influence diagnostics, and bounded
  concurrent episode append stress preserves all verified records. Phase 6I
  adds keyed source-tree binding, so same-machine cache copies from another
  project root are skipped before prior-failure or strategy advice can apply,
  and a 16-writer/1920-record bounded append soak preserves parseable NDJSON
  plus all verified records. Extended release-duration soak remains follow-up
  work.
- CI has an explicit canonical MVP example job in addition to the Rust test
  suite.

## Archive Boundary

`archive/` means "kept for audit trail only." Nothing under `archive/` should
be cited as current executable proof unless it is promoted back into the active
source tree with tests.

## Research Layout Note

The project already contains a proposed `research/` reorganization in
`F_Project_Management/GARNET_v4_2_GITHUB_REPO_LAYOUT.md`. That broad path move
should be done as a separate migration with link checking. This remediation pass
adds the current-state guide and historical ledger first so readers can navigate
the existing layout safely.
