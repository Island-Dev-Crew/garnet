# Lane 1 governance-activation bootstrap review companion

Date: 2026-07-17

State: independent exact trust-content review complete; authenticated
exact-PR-head approval, Linux/Windows CI, and U-17 live readback pending

Merge authority: Jon only

## Exact review boundary

- Base: `cede73c03c5d535306ed179b5882e99e4d17b050`
- Reviewed trust-content head: `b9f8bc91dd0660f0988e711fea31a535c0aae8f5`
- Reviewed tree: `6ed0001189997c35f90a7c1612e1780518aa5dc4`
- Trust-change digest: `sha256:8e72c1c917dc91b501e3b75ddf82d5b1d2492b516728d372bce3ac87f1e7dc40`
- Complete endpoint change set: 80 paths
- Trust-kernel subset: 50 paths

The reviewed head is provenance for the trust bytes listed below. This
companion does not extend or backdate review coverage beyond that head. The
later structured record and state-only commits may not alter any trust-kernel
path. The final authenticated GitHub approval must bind the exact final PR head,
including that record and this companion.

## All 50 touched trust-kernel paths

| Path | What changed and why |
|---|---|
| `.github/CODEOWNERS` | Implements U-16 as checked-in procedural/future ownership for every `scripts/garnet_github_*` governance transport path; the current ruleset deliberately does not claim required CODEOWNER approval, and this file is itself classified as trust-kernel so the boundary cannot be silently relaxed. |
| `.github/rulesets/README.md` | Documents immutable action-pin maintenance, semantic fingerprints, and the mandatory bootstrap-before-activation order. |
| `.github/rulesets/external-action-pins.json` | Adds the reviewed full-SHA manifest and update provenance for every external action. |
| `.github/rulesets/governance-activation-ceremony.json` | Adds a canonical Jon-only two-PR 31-to-32 ceremony; activation remains false and blocked on U-17. |
| `.github/rulesets/required-context-producers.json` | Upgrades producer rows to semantic SHA-256 bindings, pins the active aggregate, and prepares the optional base-controlled producer without requiring it. |
| `.github/workflows/agentic-dogfood-matrix.yml` | Replaces mutable action refs with reviewed commits and makes the Rust channel explicit. |
| `.github/workflows/base-controlled-trust.yml` | Adds the old-base `pull_request_target` evaluator that treats the fork candidate only as inert Git objects and uses an event-scoped review token. |
| `.github/workflows/ci.yml` | Pins actions; binds review to the exact fork head; isolates credentials; wires rolling-review v2; and runs the same four policy suites on Ubuntu, Windows, and macOS. |
| `.github/workflows/codeql.yml` | Pins CodeQL and checkout actions to reviewed commits. |
| `.github/workflows/determinism.yml` | Pins actions and supplies explicit stable Rust toolchain semantics. |
| `.github/workflows/dogfood-readiness.yml` | Pins checkout to its reviewed commit. |
| `.github/workflows/fuzz-nightly.yml` | Pins actions and supplies explicit nightly Rust toolchain semantics. |
| `.github/workflows/linux-packages.yml` | Pins actions, supplies explicit stable Rust, and preserves release publishing on an immutable action commit. |
| `.github/workflows/macos-studio.yml` | Pins actions and supplies explicit stable Rust toolchain semantics. |
| `.github/workflows/security.yml` | Pins actions and supplies explicit stable Rust toolchain semantics. |
| `.github/workflows/vscode-extension.yml` | Pins actions and both reviewed Node setup versions. |
| `.github/workflows/web-pwa-readiness.yml` | Pins checkout to its reviewed commit. |
| `Cargo.lock` | Locks the exact `same-file` and TOML dependency graph used by the fail-closed retained-source and manifest paths; it is now itself classified as trust-kernel. |
| `F_Project_Management/W_TRUST/LANDED_REVIEW_MARKERS.json` | Adds the canonical empty append-only registry that future squash-durable landed markers must enter. |
| `garnet-cli/Cargo.toml` | Adds exact dependency declarations for retained file identity and complete TOML validation; it is now itself classified as trust-kernel. |
| `garnet-cli/src/bound_source.rs` | Adds retained-handle identity binding used by vendored, helper, and discovered source reads; it is now itself classified as trust-kernel. |
| `garnet-cli/src/cmd/add.rs` | Replaces partial dependency parsing with full TOML 1.0 validation consumed by the fail-closed run path; it is now itself classified as trust-kernel. |
| `garnet-cli/src/cmd/run.rs` | Fails closed on malformed vendored manifests and consumes source bytes from the identity-bound handle. |
| `garnet-cli/src/cmd/test.rs` | Fails closed on unreadable discovery/helper inputs, missing or ambiguous roots, and zero-test sources whose setup does not load cleanly. |
| `garnet-cli/src/lib.rs` | Exports the retained-source module used by the fail-closed CLI paths; it is now itself classified as trust-kernel. |
| `scripts/garnet_base_controlled_trust_status.py` | Implements the trusted-old-base, candidate-inert 31/32 transition evaluator. |
| `scripts/garnet_cross_os_policy_manifest.py` | Runs the four suites from an exact clean head and emits fail-closed, zero-skip, head-independent parity plus head/OS-bound evidence digests. |
| `scripts/garnet_github_governance_gate.py` | Adds runnable explicit-stdin GOV-009 runtime/admin collection, clean exact-local-head binding, bounded endpoint projection, freshness, outcome, identity, and strict checked/live policy equality for the pinned 31 and 32 states. |
| `scripts/garnet_github_governance_transport.py` | Tightens bounded authenticated transport behavior used by review and GOV-009 enumeration without inheriting or rendering ambient credentials. |
| `scripts/garnet_governance_activation_ceremony.py` | Enforces the exact preparation package and two-PR Jon-only order, and makes activation runnable only with the separate U-17 admin stdin token and exact no-bypass live readback. |
| `scripts/garnet_msrv_status.py` | Projects pinned workflow actions and keeps the exact Rust 1.95 gate aligned after workflow changes. |
| `scripts/garnet_required_context_contract.py` | Adds semantic producer fingerprints, pinned aggregates, strict file handling, and 31-to-32 transition rules. |
| `scripts/garnet_trust_kernel_review_status.py` | Replaces v1 presence/ancestry claims with fail-closed enumeration, raw commit/tree append-only review-record history, one minimal scrubbed environment for every Git probe, authenticated exact-head premerge review, exact GitHub author/committer principal-union binding, and U-19 squash-durable exact landing-edge content proof. |
| `scripts/garnet_workflow_action_integrity_status.py` | Adds the all-workflow immutable-action manifest reporter and explicit Rust-toolchain check. |
| `scripts/garnet_workflow_identity_policy.py` | Shares the active-31 and prepared-32 producer identity, semantic, and full-binding checks used by local and live governance evaluation. |
| `scripts/garnet_workflow_schema_policy.py` | Rejects vacuous `run: true` and false job/step conditions as semantic policy failures. |
| `scripts/test_garnet_base_controlled_trust_status.py` | Traps transition downgrade, byte drift, semantic failure, rolling-review failure, token misuse, and candidate execution. |
| `scripts/test_garnet_cross_os_policy_manifest.py` | Traps wrong OS/head, dirtiness, special index flags, skips, test-ID/count/order divergence, suite mutation, and head-dependent parity. |
| `scripts/test_garnet_github_governance_gate.py` | Traps incomplete/malformed collections, freshness/head/outcome drift, fabricated identities, duplicate checked JSON, and policy mismatch. |
| `scripts/test_garnet_github_governance_transport.py` | Extends bounded transport regressions, including terminal-page completeness. |
| `scripts/test_garnet_governance_activation_ceremony.py` | Traps ceremony/order/token/digest/bypass drift and preserves blocked-U-17 activation. |
| `scripts/test_garnet_msrv_status.py` | Updates exact workflow/MSRV expectations after immutable pins. |
| `scripts/test_garnet_required_context_contract.py` | Adds semantic, strict-file, macOS `/var`, and ancestor-symlink regressions. |
| `scripts/test_garnet_required_context_evaluator.py` | Adds observed semantic-digest and coordinated-fabrication traps. |
| `scripts/test_garnet_trust_kernel_review_status.py` | Expands to 110 fail-closed enumeration, parallel/record-only addition, intermediate mutation/restoration, Git control-plane and graft isolation, exact API identity types, author/committer independence, credential, append-only, exact-head, exact landing-edge, and squash-durability traps. |
| `scripts/test_garnet_vscode_release_assets.py` | Updates release workflow expectations for the reviewed pinned action. |
| `scripts/test_garnet_workflow_action_integrity_status.py` | Traps mutable, unknown, wrong, unused, and Rust-channel-less action pins. |
| `scripts/test_garnet_workflow_file_policy.py` | Makes NFC/NFD collision proof raw-index based and host-filesystem independent. |
| `scripts/test_garnet_workflow_schema_policy.py` | Adds exact vacuous-command and false-condition regressions. |
| `scripts/test_garnet_workflow_yaml_policy.py` | Updates the typed-YAML suite for semantic-policy projection. |

## Independent reviewers and verdicts

- `/root/review_item2_final` — earlier **APPROVE** covered the 92-test
  predecessor after edit/revert, pagination, path-alias, credential-isolation,
  and U-19 probes; later repairs are covered only by the exact-head reviewer
  recorded below.
- `/root/item1_strict_loader_review` — **APPROVE** Item 1's local/offline
  mechanism after 18/18 evaluator and 23/23 transport tests plus direct strict
  loader and ambient-credential probes; U-17 explicitly excluded.
- `/root/ceremony_recon` — **APPROVE** Items 3 and 4 after semantic suites,
  independent action-ref resolution, predecessor immutable-action proof, and U-16
  inspection.
- `/root/ceremony_recon/independent_item7_review` — **APPROVE** Item 7 after
  independent old-base/candidate-inert/token/order review and focused gates.
- `/root/item5_macos_parity_review` — **APPROVE** Item 5 on macOS after the four
  policy suites and targeted `/var`, ancestor-symlink, and NFC/NFD probes;
  Linux/Windows execution explicitly excluded.
- `/root/integrated_trust_diff_review` — repaired four final rolling-review
  findings and reran 110/110; because that agent authored the repairs, it is
  explicitly not the independent final reviewer.
- `/root/lane1_exact_head_review` — **APPROVE** all 50 trust paths at exact head
  `b9f8bc91dd0660f0988e711fea31a535c0aae8f5`; no critical, important, or minor
  implementation findings. The reviewer independently reran rolling review
  110/110, GOV transport 24/24, GOV evaluator 29/29, base control 4/4,
  ceremony 10/10, producer/parity suites, exact-head macOS 36/36 with zero
  skips, and action integrity 90/90 with zero mutable references. External
  GitHub approval, Linux/Windows, and U-17 were explicitly excluded from this
  local verdict.

These scoped agent reviews are substantive Covenant 3 evidence, but they do not
substitute for the canonical review JSON or the authenticated GitHub approval
from its recorded independent reviewer at the exact final PR head.

## Fresh evidence files

- `ops/lane1/evidence/10-rolling-review-v2-design.md`
- `ops/lane1/evidence/20-item1-gov009-design.md`
- `ops/lane1/evidence/30-item3-semantic-producer-design.md`
- `ops/lane1/evidence/40-item4-action-integrity.md`
- `ops/lane1/evidence/50-item5-macos-parity.md`
- `ops/lane1/evidence/60-item6-fail-soft.md`
- `ops/lane1/evidence/70-item7-ceremony.md`

Fresh macOS evidence is green: all four policy suites, rolling review 110/110,
action integrity 90/90 with zero mutable refs, Rust MSRV 25/25, agent contracts,
`cargo fmt`, the full `garnet-cli` suite, and `cargo test --workspace
--no-fail-fast`. The system Python lacked exact PyYAML 6.0.3, so YAML-dependent
commands used the disposable pinned lane environment; that degradation is
recorded in the mission ledger.

The clean detached macOS parity manifest at the exact reviewed head reports
36/36 tests, zero skips/failures/errors, parity digest
`cf3631ea6afd3443d040500c5c453c07c7609c5994975bbff74e6d9f608c8cd6`,
and macOS evidence digest
`d88e841b163837b13c9a2b3ca39e3b0e65f15cd81fb92329cdac81e792a50483`.

Linux and Windows evidence files do not yet exist. Their identical four-suite
matrix and the rest of the required CI chain must pass on the final PR head.
Until those artifacts and the authenticated structured record exist, the
trust-kernel gate and merge-readiness claim remain RED.

## Governance and non-claims

- This is the bootstrap PR. It leaves the checked-in and live ledger at 31 and
  does not include the S114 terminus artifact.
- Jon must merge this bootstrap before a separate activation/terminus PR can
  add the already-landed context; otherwise `pull_request_target` cannot load
  the producer from `main` and the context deadlocks.
- U-17 is open. No ambient, fork, plugin, browser, or connector credential was
  used as an admin-authoritative substitute.
- Launch remains HOLD at 100.0% S114, 93.1% truth pulse, 50.0%
  launch-critical, and 37.5% launch ledger. Band 3 and S6 advisory remain the
  honest ceiling.
- The governance freeze is prepared, not active. Only Jon's later merge of the
  activation/terminus PR can arm it.
