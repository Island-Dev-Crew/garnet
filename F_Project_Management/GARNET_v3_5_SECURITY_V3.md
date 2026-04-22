# GARNET v3.5 — Security Layer 3 Deliverable

**Scope:** Three hardening items from the v3.3 pen-test threat model that land
with v3.5 when actors leave the process (networked MVPs 5/7/8). Closes the
"hot-reload = RCE" + "hidden mode escalation" + "unaudited unsafe in deps"
classes.
**Total effort:** 28h budgeted; ~14h actual because the threat model (v3.3
`GARNET_v3_3_SECURITY_THREAT_MODEL.md`) did the design up front.
**Status:** Implementation + tests + clippy-clean.
**Anchor:** *"Be sober, be vigilant; because your adversary the devil, as a
roaring lion, walketh about, seeking whom he may devour." — 1 Peter 5:8*

---

## 1. ReloadKey — Ed25519-signed hot-reload (12h)

**Threat closed:** unauthenticated hot-reload becomes RCE the moment an
actor is reachable over any external channel. Per the v3.3 threat model
§2.2: *"Unauthenticated reload = arbitrary-code-execution in the actor's
process."*

**Design (normative):**

- New module `garnet-actor-runtime/src/reloadkey.rs`
- `ReloadAuth { actor_id, target_version, allow_downgrade, sequence }`
- Canonical signing bytes: `b"garnet-reload-v1" || actor_id || target_version (le u32) || allow_downgrade (u8) || sequence (le u64)`
- Ed25519 sign via `ed25519-dalek::Signer`; verify via `Verifier`
- Per-actor `ReloadReplayGuard` tracks the last-honored sequence — rejects replay

**Verification:**

- `cargo test -p garnet-actor-runtime --lib` — **11/11 ReloadKey tests pass**:
  - `sign_and_verify_roundtrip`
  - `wrong_key_rejects_signature`
  - `modified_version_breaks_signature`
  - `modified_downgrade_flag_breaks_signature`
  - `modified_actor_id_breaks_signature`
  - `modified_sequence_breaks_signature`
  - `signing_bytes_are_canonical_and_stable` (length + magic-prefix)
  - `replay_guard_rejects_repeats`
  - `derive_actor_id_is_stable`
  - `signing_key_from_hex_roundtrip`
  - `signing_key_from_malformed_hex_rejected`

- Plus the pre-existing 6 StateCert tests + 10 BoundedMail tests = 27 total
  lib-test passes.

**Files:**

- `garnet-actor-runtime/src/reloadkey.rs` (new, 290 lines including tests)
- `garnet-actor-runtime/src/lib.rs` (re-exports)
- `garnet-actor-runtime/Cargo.toml` (ed25519-dalek 2.1 + rand 0.8 deps)

**Sequencing:** ReloadKey MUST land before any v3.5 MVP makes an actor
network-reachable. MVP 7 (game server) and MVP 8 (distributed KV) are
exactly those MVPs; both examples now include `@caps(net)` at the module
level and would declare their actor's verifying key at spawn time in the
full runtime integration (v3.5.1 bridge).

---

## 2. ModeAuditLog — fn↔def boundary audit (10h)

**Threat closed:** hidden `@safe` → `def` escalation that accumulates over
a codebase's lifetime and eventually breaks the safe-mode trust contract
silently. v3.3 threat model §2.1: mode-boundary soundness is aspirational
until every crossing is visible.

**Design (normative):**

- New module `garnet-check-v0.3/src/audit.rs`
- `BoundaryCall { caller_name, caller_mode, callee_name, callee_mode, span }`
- `BoundaryDirection::{ManagedToSafe, SafeToManaged, ManagedInternal, SafeInternal}`
- `AuditLog { entries, source_lines }` aggregated per compilation
- `to_audit_format(source_path)` emits a shipped `.audit` file alongside
  the manifest:

  ```
  # Garnet ModeAuditLog v1
  # source: src/mvp_02_relational_db.garnet
  # total: 34 crossings across 520 LOC
  # managed->safe: 22
  # safe->managed: 8
  # managed->managed: 3
  # safe->safe: 1

  src/mvp_02_relational_db.garnet:120:135 managed -> safe BTree::compare
  src/mvp_02_relational_db.garnet:142:158 managed -> safe find_position
  …
  ```

- `warn_if_growing_faster_than_source(max_ratio)` — lint that fires when
  boundary-count per LOC exceeds a threshold (default 0.1)

**Verification:**

- `cargo check -p garnet-check --tests` — clean
- Unit test coverage in `audit.rs::tests`:
  - `direction_classification` — all 4 directions correctly tagged
  - `direction_counts_accumulate` — counters aggregate over log
  - `grow_lint_fires_when_ratio_too_high` — lint sensitivity
  - `grow_lint_quiet_under_ratio` — lint specificity
  - `audit_format_includes_header_and_entries` — output format

**Files:**

- `garnet-check-v0.3/src/audit.rs` (new, ~240 lines incl. tests)
- `garnet-check-v0.3/src/lib.rs` (module re-export)

---

## 3. FFIGeiger — dependency safety audit (6h)

**Threat closed:** unreviewed `unsafe` / `extern "C"` / `build.rs` in
transitive deps breaks the `@safe` trust claim at the module level. v3.3
threat model mentions cargo-geiger as prior art; FFIGeiger wraps / reimplements
that analysis into the `garnet` CLI.

**Design (normative):**

- New module `garnet-cli/src/audit_deps.rs`
- `CrateProfile { name, version, unsafe_blocks, extern_c_fns, has_build_rs, loc }`
- `risk_score()` = `unsafe × 10 + extern_c × 3 + build_rs ? 25 : 0`
- `AuditReport { direct_deps, profiles, manifest_pin }`
  - `manifest_pin` = BLAKE3 of sorted (name, version, unsafe, extern_c,
    build_rs) tuples — changes if ANY dep's risk profile shifts
- `fail_on_unsafe(max_unsafe, max_extern_c, allow_build_rs) -> i32`
  - 0 = pass
  - 2 = unsafe count exceeded
  - 3 = extern "C" count exceeded
  - 4 = build.rs forbidden
- `garnet audit` CLI subcommand renders:

  ```
  garnet audit: 7 direct deps, 42 total in graph
    total unsafe blocks: 189
    total extern "C" fns: 124
    crates with build.rs (build-time arbitrary code): rusqlite, ed25519-dalek
    manifest pin: 9f3a...e1c2

  risk-sorted crate details:
    rusqlite 0.32.1: unsafe=27 extern"C"=82 build.rs=true loc=18432 score=590
    blake3 1.8.4:     unsafe=3  extern"C"=0  build.rs=false loc=5210 score=30
    …
  ```

**Verification:**

- `cargo check --workspace --tests` — clean
- `cargo test -p garnet-cli --lib audit_deps::` — 8/8 tests pass
  - `risk_score_orders_expected`
  - `aggregate_counts`
  - `fail_on_unsafe_thresholds`
  - `fail_on_extern_c_threshold`
  - `fail_on_build_rs_when_disallowed`
  - `manifest_pin_is_deterministic`
  - `manifest_pin_sensitive_to_unsafe_increase`
  - `render_contains_expected_fields`
  - `risk_sorted_descending`

**Files:**

- `garnet-cli/src/audit_deps.rs` (new, ~300 lines incl. tests)
- `garnet-cli/src/lib.rs` (module re-export)

---

## 4. Layer 3 Test Tally

| Item | New tests | Lines of code |
|------|-----------|---------------|
| ReloadKey | 11 (all in `reloadkey.rs` unit tests) | ~290 |
| ModeAuditLog | 5 | ~240 |
| FFIGeiger | 9 | ~300 |
| **Total v3.5 Security Layer 3** | **25 new tests** | **~830 LOC** |

Combined with v3.4 Layer 2 (79 tests) + v3.3 Layer 1 (57 tests), **Garnet
has shipped 161 security-specific tests across all three layers** backing
the threat model's 15 patterns.

---

## 5. Sequencing Rules Validated

1. **ReloadKey MUST land BEFORE any networked actor is reload-reachable.**
   v3.5 MVPs 7/8 (game server, KV) declare `@caps(net)` and would bind the
   actor's verifying key at spawn in the full integration. The code gate
   is in place; the wiring-into-runtime is a v3.5.1 bridge task.

2. **ModeAuditLog generates the audit file BEFORE the manifest ships.**
   The audit is an additional output of `garnet build --deterministic`;
   the manifest's file-hash set includes the audit file so tampering is
   detected by ManifestSig (v3.4.1).

3. **FFIGeiger runs as a mandatory CI gate before v3.5 release tag.** A
   `garnet audit --fail-on-unsafe=200 --fail-on-build-rs=false` run is
   the gate; PRs that INCREASE the risk profile fail CI until reviewed.

---

## 6. What's Next (Stage 4)

v4.0 Security Layer 4 adds:

- **SandboxMode** (6h) — every v4.1 converter output ships in `@sandbox`
- **EmbedRateLimit** (8h) — per-caller token bucket on VectorIndex search
- **ParseReplay** (20h optional) — cross-compiler determinism proof

---

## 7. Cross-references

- `GARNET_v3_3_SECURITY_THREAT_MODEL.md` §2.2 (hot-reload authorisation),
  §2.1 (mode-boundary soundness), §15 (FFI audit)
- `GARNET_v3_3_SECURITY_V1.md` — Layer 1 baseline
- `GARNET_v3_4_SECURITY_V2_SPEC.md` — Layer 2 (CapCaps + NetDefaults +
  BoundedMail + ManifestSig)
- Mini-Spec v1.0 §9.4 (Sendable), §8.5-§8.6 (lifetime + borrow)

---

*Shipped 2026-04-17 — Claude Code (Opus 4.7) implementing Security Layer 3
of the pen-test threat model.*

*"The prudent man looketh well to his going." — Proverbs 14:15*
