# GARNET v3.3 — Slop Re-verification Report (Phase 1A)

**Auditor:** Claude Opus 4.7 (independent adversarial pass, Stage 1 Phase 1A)
**Date:** 2026-04-16
**Scope:** Full v3.2 implementation — all 14 phases, all claims, all tests
**Prior audit:** Explorer 1 forensic audit (reported 99% genuine)
**Result:** **v3.2 is ~95% genuine** — 4 real issues (1 WEAK, 2 MISLEADING, 1 LIGHT) that Explorer 1 missed

## Methodology

Three independent paths to verification, matching the user's "triple-verify" requirement:

1. **Code inspection**: opened every file Explorer 1 cited; re-read with adversarial lens ("does the test actually prove the claim?")
2. **Runtime verification**: built the release binary; ran actual CLI commands across different working directories and different time windows; inspected produced `.garnet-cache/` state via SQLite directly
3. **Grep audit**: `unwrap()` / `expect()` / `panic!` / `unreachable!()` / `cfg!(test)` / `todo!` / `unimplemented!` in all non-test src/

## Verdict Summary

| Claim | v3.2 Test Says | Reality | Severity |
|-------|----------------|---------|----------|
| Paper VI C5: Automatic error-model bridging | Tests 4 directions of manual try/rescue | Tests **manual** bridging — author comment admits "v0.4 wires automatic bridging" | **MISLEADING** |
| Paper VI C6: Hot-reload with state migration | Tests state "carries across" | Migrator **drops** `old`, reconstructs from values captured externally — runtime has no `extract_state` path | **MISLEADING** |
| Paper VI C7: Deterministic build | Tests back-to-back builds in same cwd | `prelude_hash` is a hash of the literal string `"garnet-prelude-v0.3.2"`, not actual prelude content | **WEAK** |
| Paper VI C7: Byte-identical across environments | Tests two builds in same dir | **My runtime test confirms byte-identical across different cwds + 2s interval** — claim holds empirically even though v3.2 test is shallow | **LIGHT** (empirically OK, but v3.2 test doesn't prove the claim rigorously) |
| Path resolution robustness | Not tested | `env.get(segs.last().unwrap())` in non-test code — panics if parser emits empty path | **LIGHT** |

## Detailed Findings

### FINDING 1 — WEAK: `prelude_hash` is a static string

**File:** `E_Engineering_Artifacts/garnet-cli/src/manifest.rs:47`

**Claim:** Paper VI Contribution 7 says deterministic builds capture "every input that affects the compiled output." The manifest includes a `prelude_hash` field documented as "prelude hash" — implying it reflects prelude content.

**Reality:**
```rust
prelude_hash: hash_str("garnet-prelude-v0.3.2"),
```

This hashes a **literal version string**, not actual prelude content. Grep for `prelude` across the workspace returns only (a) this static string in manifest.rs and (b) sidecar manifest.json files containing the resulting hash. **No actual prelude file exists to be hashed.**

**Implication:** If the prelude implementation evolves without a version bump, every existing manifest remains bit-for-bit stable — even though the compiled output would now behave differently. A careful MIT reviewer will find this.

**Fix (v3.3):**
- **Option A (preferred):** Bake the prelude source as a `pub const PRELUDE: &str = include_str!("prelude.garnet")` in the interpreter crate; hash THAT in `manifest.rs`
- **Option B:** Compute the prelude hash from the builtin registry (hash the sorted list of builtin names + type signatures)
- **Option C:** Feature-flag the current behavior as `prelude_hash_source = "version-string"` and add an `actual-source` mode

**Effort:** 2 hours. Clean fix.

---

### FINDING 2 — MISLEADING: Hot-reload state migration tests cheat

**File:** `E_Engineering_Artifacts/garnet-actor-runtime/tests/reload.rs:91-105` (test 1) and `128-145` (test 2)

**Claim:** Paper VI Contribution 6 says Garnet provides "hot-reload with state migration across mode boundaries." Test file docstring (line 11) says "Schema version monotonically increases; the migrator transfers state from v1 to v2."

**Reality:** The migrator closure takes `Box<dyn ActorBehaviour<M, R>>` as argument. `ActorBehaviour` exposes no state accessor. The test's migrator at line 100-104 does:

```rust
.reload(2, false, move |old: Box<dyn ActorBehaviour<String, String>>| {
    drop(old);                                         // ← old is discarded
    Box::new(CounterV2 {
        n: pre_count,                                  // ← captured from OUTER scope
        label: "migrated".to_string(),
    })
})
```

The state "migration" works because `pre_count` was captured from the outer function's scope **before** the reload command was issued. The test comment at lines 97-100 is explicit: *"we can't downcast to CounterV1 without Any. We do, however, know its current 'n' value because we just observed it via .ask('ping') above."*

**Implication:** Paper VI Contribution 6's state-migration contract isn't validated. In production, a user running `hot_reload_from_wire_protocol(new_version_bytes)` would have no channel to read the old actor's state. The runtime pattern admitted in the comment ("in production, the migrator would read state via a serialise/deserialise step") isn't implemented. The v3.2 test proves only:
- ✅ Reload-ordering invariant (pre-reload replies are v1, post-reload are v2)
- ✅ Downgrade refusal semantics
- ✅ In-flight handler safety
- ❌ **State transfer is NOT validated by a test that couldn't pass with a broken implementation**

**Fix (v3.3 design, v3.4 implementation):**
- **Option A (preferred):** Extend the `Actor` trait with `type State: Serialize + DeserializeOwned` and add `fn extract_state(&self) -> Self::State`. Migrator signature becomes `FnOnce(OldState) -> NewActor`. Runtime calls `extract_state` before `on_stop`; passes bytes to migrator.
- **Option B:** Require `Actor: Any` and let migrator downcast. Simpler, but forfeits wire-protocol reloads.
- **Option C:** Add `fn snapshot(&self) -> Vec<u8>` / `fn restore(bytes: &[u8]) -> Self` to Actor. Most flexible; aligns with Erlang's design.

**Effort:** 6-8 hours including test rewrites.

---

### FINDING 3 — LIGHT: v3.2 determinism test doesn't stress different environments

**File:** `E_Engineering_Artifacts/garnet-cli/tests/reproducible.rs:41-59`

**Claim:** Paper VI Contribution 7 says "byte-identical output across environments."

**Reality:** `build_twice_produces_identical_manifest` runs two consecutive builds in the **same cwd** at the **same target triple** at **back-to-back wall clocks** (microseconds apart). Because the manifest contains no time-varying or cwd-varying fields, this test trivially passes. It doesn't stress the claim.

**Runtime verification (my pass):** I ran two `garnet build --deterministic` commands in DIFFERENT working directories (`/tmp/garnet_audit/dir1/` and `/tmp/garnet_audit/dir2/`) with a 2-second sleep between them. Result:

```
source_hash = 9ffa9dbeaab614739c177aa79e7fde6e3d96dae644fa5703e9e67dea14704a75
ast_hash    = b4b06a0d6bd4d620c3ac368731c1c4f4e712a3a5f2f24fc08f1f6c52786ebc44
```

Identical across both runs. Manifests byte-identical. **Claim holds empirically.** But the v3.2 test suite doesn't prove it — my runtime test does.

**Fix (v3.3):** Add a new test `build_identical_across_different_cwds_and_intervals` in `reproducible.rs` that:
- Creates two temp dirs with different paths
- Builds in each with a 1s sleep between
- Diffs manifests byte-for-byte

**Effort:** 30 minutes.

---

### FINDING 4 — MISLEADING: "Automatic" error bridging is manual

**File:** `E_Engineering_Artifacts/garnet-interp-v0.3/tests/boundary_errors.rs:78-80`

**Claim:** Paper VI Contribution 5: *"Automatic bidirectional error-model bridging — the compiler automatically bridges between managed mode's exception model and safe mode's Result model."*

**Reality:** Test comment at lines 78-80 is explicit:
> *"Until v0.4 wires automatic bridging, the safe fn does this explicitly — which is the bridging contract: safe code MUST surface raises as Err, and try/rescue is the language-level escape hatch."*

All v3.2 bridging tests show **user-authored** try/rescue wrappers that manually convert between the two error models. The tests do prove:
- ✅ `?` operator unwraps Result::Ok / rescues Err
- ✅ `try {...} rescue e {...}` catches raised exceptions
- ✅ Err payloads survive round-trip (tested: "unlucky number" reaches rescue handler)
- ✅ Type mismatch at boundary surfaces as an error (doesn't silently corrupt)

But they don't test:
- ❌ Automatic insertion of try/rescue at mode boundaries by the interpreter/type-checker
- ❌ Zero-overhead elision when both sides use the same model

**Implication:** The paper claim is ahead of the v3.2 implementation by one version. This is the kind of thing a reviewer who reads both the paper and the test source will catch in 10 minutes.

**Fix options (v3.3):**
- **Option A:** Update Paper VI to say "bidirectional error-model bridging via try/rescue + ? semantics" (drop "automatic") — truthful and defensible.
- **Option B:** Actually implement automatic bridging — when the type-checker sees a managed→safe call, insert an implicit try/rescue wrapper. Significant work but matches the paper.
- **Option C:** Mark C5 as "v4.0 target, v3.2 proves manual path works" in Paper VI — defers the claim honestly.

**Recommendation:** Option A or C for v3.3; Option B for v4.0.

**Effort:** 2 hours (A or C), ~20 hours (B).

---

### FINDING 5 — LIGHT: `segs.last().unwrap()` in path resolution

**File:** `E_Engineering_Artifacts/garnet-interp-v0.3/src/eval.rs:176`

**Reality:**
```rust
env.get(segs.last().unwrap())  // ← unwrap on a potentially empty path
    .ok_or_else(|| RuntimeError::Message(format!("unresolved path: {}", segs.join("::"))))
```

If the parser were to emit `Expr::Path(vec![], _)` (nameless path), this unwraps to a panic in non-test code. The parser likely never emits this, but there's no defensive guard. The Phase 1 panic-elimination work was thorough elsewhere — this one slipped through.

**Fix:** Replace with:
```rust
let last = segs.last().ok_or_else(|| RuntimeError::Message("empty path expression".into()))?;
env.get(last).ok_or_else(|| ...)
```

**Effort:** 5 minutes. Include in v3.3 Phase 1A cleanup.

---

## Claims That Are GENUINE

Confirmed by my independent audit (code inspection + runtime testing):

| Claim | Evidence |
|-------|----------|
| Stress tests run at 100K / 1M / 50K / 1000 scale | `garnet-memory-v0.3/tests/stress.rs` — real numeric constants; no disguised `for i in 0..10` |
| 200 actors × 1000 messages | `garnet-actor-runtime/tests/stress.rs:22-41` — real loops; confirmed `counter == 200_000` |
| Episode logging across process restarts | Runtime test: 2 separate `garnet` invocations in same cwd → 4 lines in `.garnet-cache/episodes.log` |
| SQLite knowledge.db is real | Runtime test: `sqlite3.connect('.garnet-cache/knowledge.db')` returned 4 rows, all outcome='ok'; knowledge.db is 16384 bytes |
| Strategy replay on repeat invocation | Runtime test: second run output shows `note: strategy 'skip_check_if_unchanged_since_last_ok' applies (Hamming distance 0/256, last triggered ts=1776390390)` — **the Compiler-as-Agent is genuinely learning and replaying** |
| BLAKE3 manifest hashing with canonical BTreeMap JSON | `manifest.rs:60-91` — hand-rolled writer, no serde, no non-deterministic libs |
| Byte-identical manifests across different cwds | My runtime test above — confirmed identical |
| Kind-aware dispatch to 4 stores | `kind_dispatch.rs` — tests call kind-specific methods and verify kind-isolation errors on wrong-kind methods |
| Hot-reload runtime mechanics (drain+migrator+replay+downgrade refusal) | `runtime.rs:187-226` — full implementation with buffered replay; downgrade-refusal path verified (test 3) |
| Cross-boundary error tests cover 4 directions | `boundary_errors.rs` — all 4 tests present; Err payload survives round-trip (test 5) |
| 3 example programs parse + check cleanly | Runtime test: multi_agent_builder (8 fns / 6 boundaries), agentic_log_analyzer (22 fns / 25 boundaries), safe_io_layer (16 fns / 30 boundaries) — **0 diagnostics** on all three |
| No cfg!(test), todo!, unimplemented! in production paths | Grep: only `#[cfg(test)]` for test modules (standard pattern) |
| No panic! in production paths | Grep + verification: all `unreachable!()` are match exhaustiveness patterns at eval.rs:267, 286, 545, 550, 555 and stmt.rs:132 — each is a `_` arm of a match where earlier arms cover all real cases |

## Gap Between Explorer 1 and This Audit

Explorer 1 reported "99% genuine, zero slop." It missed all 4 of my real findings. Why?

- **Explorer 1 was surface-reading**: verified that code files exist, tests exist, APIs are called. It didn't adversarially ask "does the test PROVE the claim, or does it pass against a stub implementation?"
- **Example:** reload.rs tests 1 & 2 pass. Explorer 1 saw `assert_eq!(first_post, pre_count + 1)` and concluded "state migration works." But reading the migrator closure reveals that `pre_count` came from an outside capture — the migrator could have been empty and the test would still pass.
- **Example:** manifest.rs has `prelude_hash: hash_str("garnet-prelude-v0.3.2")` — Explorer 1 saw `hash_str` being called and concluded "real BLAKE3 hashing." But the INPUT to the hash is a static string, not actual prelude content.
- **Example:** boundary_errors.rs has a clear "Until v0.4 wires automatic bridging" comment. Explorer 1 didn't flag this despite it being a direct admission of a gap between paper claim and implementation.

**Lesson for future audits:** read tests adversarially — *"how would this test pass even if the feature were broken?"*

## Phase 1A Verdict

v3.2 is **not AI slop**. The implementations are real, the tests mostly genuine, and the runtime behavior holds empirically. But Paper VI Contributions 5, 6, 7 have documentation ahead of implementation — three of my four findings are about the gap between paper claims and what the tests actually validate.

**Stage 2 can proceed**, but with these v3.3 cleanup items folded in:

1. Fix `prelude_hash` to hash actual prelude content (2 hrs)
2. Redesign Actor trait to support real state extraction (6-8 hrs) — unblocks Paper VI Contribution 6 claim
3. Add cross-cwd determinism test (30 min)
4. Update Paper VI Contribution 5 wording OR implement automatic bridging (2 hrs for doc fix)
5. Fix eval.rs:176 unwrap (5 min)

**Total v3.3 cleanup effort: ~10 hours** (vs. the ~30-35 hrs Stage 1 budgeted).

## Next

Proceed to Phase 1B (research gap closure: Swift/Rust/Ruby blend verification).

---

*Generated by: Claude Opus 4.7 — independent adversarial audit
2026-04-16*
