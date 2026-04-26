//! Hot-reload mode boundaries — Paper VI Contribution 6.
//!
//! These tests prove the four reload contracts:
//!
//! 1. **Ordering invariant.** Messages enqueued before reload arrives are
//!    handled by the OLD behaviour; messages enqueued after — including
//!    the buffered ones the runtime drains and replays — are handled by
//!    the NEW behaviour. Reply ordering must show a clean v1→v2 cutover.
//! 2. **Forward migration.** Schema version monotonically increases; the
//!    migrator transfers state from v1 to v2; the new behaviour answers
//!    based on the migrated state.
//! 3. **Backward refusal.** Reloading to a lower schema version without
//!    `allow_downgrade=true` is refused; the actor continues running the
//!    old behaviour unchanged.
//! 4. **In-flight handler safety.** The reload swap point sits between
//!    `handle()` calls, never mid-handler — proven by sending a long-running
//!    handler and observing the reload waits its turn.

use garnet_actor_runtime::{Actor, ActorBehaviour, ReloadOutcome, Runtime, TaggedState};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use std::time::Duration;

// ── Versioned counter actor: state is `n: i64`, schema_version = N ──
//
// v3.3 StateCert: `extract_state()` returns a TaggedState carrying a
// BLAKE3 type fingerprint. Migrators call `.downcast::<T>()` on it;
// mismatched T returns `FingerprintMismatch` instead of panicking.
// Closes the type-confusion cliff that plain `Box<dyn Any>` would have
// created once hot-reload reaches any external channel.

struct CounterV1 {
    n: i64,
}

impl Actor for CounterV1 {
    type Message = String;
    type Reply = String;
    fn handle(&mut self, msg: String) -> String {
        match msg.as_str() {
            "ping" => format!("v1:{}", self.n),
            "incr" => {
                self.n += 1;
                format!("v1:{}", self.n)
            }
            other => format!("v1:?{other}"),
        }
    }
    fn schema_version(&self) -> u32 {
        1
    }
    fn extract_state(&self) -> Option<TaggedState> {
        Some(TaggedState::new(self.n))
    }
}

struct CounterV2 {
    n: i64,
    /// New field added in v2: a label appended to every reply.
    label: String,
}

impl Actor for CounterV2 {
    type Message = String;
    type Reply = String;
    fn handle(&mut self, msg: String) -> String {
        match msg.as_str() {
            "ping" => format!("v2[{}]:{}", self.label, self.n),
            "incr" => {
                self.n += 1;
                format!("v2[{}]:{}", self.label, self.n)
            }
            other => format!("v2[{}]:?{other}", self.label),
        }
    }
    fn schema_version(&self) -> u32 {
        2
    }
    fn extract_state(&self) -> Option<TaggedState> {
        // v2 carries (n, label) so a v3 migrator could recover both fields.
        Some(TaggedState::new((self.n, self.label.clone())))
    }
}

// ── Test 1: Ordering invariant ──────────────────────────────────────

#[test]
fn ordering_invariant_v1_replies_before_v2_replies() {
    let rt = Runtime::new();
    let addr = rt.spawn(CounterV1 { n: 0 });

    // Enqueue 5 incrs synchronously so they all hit v1 before reload.
    let mut early = Vec::new();
    for _ in 0..5 {
        early.push(addr.try_ask("incr".to_string()).expect("ask"));
    }
    for r in &early {
        assert!(r.starts_with("v1:"), "pre-reload reply must be v1: {r}");
    }

    // Reload to v2. The migrator extracts state directly from `old` via the
    // Actor trait's `extract_state()` method — no external pre-read. This
    // path is what a production hot-reload would do: new code receives a
    // boxed state, downcasts, and constructs the new actor. Tests that
    // don't exercise this path can't distinguish working migration from
    // broken migration (v3.2 gap).
    let outcome = addr
        .reload(
            2,
            false,
            move |old: Box<dyn ActorBehaviour<String, String>>| {
                let tagged = old
                    .dyn_extract_state()
                    .expect("CounterV1 must expose extract_state for reload");
                // StateCert: fingerprint-verified downcast. If CounterV1's
                // state type ever drifts from i64, this returns
                // FingerprintMismatch rather than silently invoking the
                // wrong constructor on the new actor.
                let old_n = *tagged
                    .downcast::<i64>()
                    .expect("CounterV1 state fingerprint must match i64");
                drop(old);
                Box::new(CounterV2 {
                    n: old_n,
                    label: "migrated".to_string(),
                })
            },
        )
        .expect("reload should succeed");
    assert!(matches!(outcome, ReloadOutcome::Ok { .. }));

    // Post-reload: every reply must be v2-tagged.
    let mut late = Vec::new();
    for _ in 0..5 {
        late.push(addr.try_ask("incr".to_string()).expect("ask"));
    }
    for r in &late {
        assert!(r.starts_with("v2["), "post-reload reply must be v2: {r}");
    }

    // Migration carried state that came FROM the old behaviour. If
    // extract_state returned a different value than the live counter,
    // this assertion would fail — distinguishing working migration from
    // broken migration.
    let pre_count = early.len() as i64; // we did 5 incrs
    let first_post: i64 = late[0].trim_start_matches("v2[migrated]:").parse().unwrap();
    assert_eq!(
        first_post,
        pre_count + 1,
        "v2's first incr must return pre_count+1 — proving extract_state transferred the i64 correctly"
    );
}

// ── Test 2: Forward migration ───────────────────────────────────────

#[test]
fn forward_migration_v1_to_v2_transfers_state() {
    let rt = Runtime::new();
    let addr = rt.spawn(CounterV1 { n: 99 });
    // Confirm v1 is live.
    assert_eq!(addr.try_ask("ping".to_string()).expect("ask"), "v1:99");

    // Migrator extracts state from `old` via fingerprint-verified
    // downcast — refuses silently-wrong type on mismatch.
    addr.reload(2, false, |old| {
        let tagged = old
            .dyn_extract_state()
            .expect("CounterV1 exposes extract_state");
        let n = *tagged.downcast::<i64>().expect("CounterV1 state is i64");
        Box::new(CounterV2 {
            n,
            label: "carried".to_string(),
        })
    })
    .unwrap();

    let r = addr.try_ask("ping".to_string()).expect("ask");
    assert_eq!(r, "v2[carried]:99");
}

// ── Test 2b: Migration from v2 to hypothetical v3 carries compound state ──

#[test]
fn compound_state_migration_v2_carries_tuple() {
    // Verifies that actors returning complex state (a tuple) from
    // extract_state round-trip correctly through downcast. If the
    // migrator receives a different type than declared, downcast returns
    // Err — we assert the success case.
    let rt = Runtime::new();
    let addr = rt.spawn(CounterV2 {
        n: 7,
        label: "before".to_string(),
    });
    assert_eq!(
        addr.try_ask("ping".to_string()).expect("ask"),
        "v2[before]:7"
    );

    // Migrate within v2 to a fresh v2 behaviour that inherits state.
    // Fingerprint-verified downcast to (i64, String) tuple.
    addr.reload(2, false, |old| {
        let tagged = old
            .dyn_extract_state()
            .expect("CounterV2 exposes extract_state");
        let (n, label) = *tagged
            .downcast::<(i64, String)>()
            .expect("state fingerprint is (i64, String)");
        // Prove both fields came through: append "-carried" to label.
        Box::new(CounterV2 {
            n,
            label: format!("{label}-carried"),
        })
    })
    .unwrap();

    assert_eq!(
        addr.try_ask("ping".to_string()).expect("ask"),
        "v2[before-carried]:7"
    );
}

// ── Test 3: Backward refusal ────────────────────────────────────────

#[test]
fn backward_migration_refused_without_allow_downgrade() {
    let rt = Runtime::new();
    let addr = rt.spawn(CounterV2 {
        n: 5,
        label: "live".to_string(),
    });
    // Confirm v2 is live.
    assert_eq!(addr.try_ask("ping".to_string()).expect("ask"), "v2[live]:5");

    // Attempt to downgrade to v1 without permission.
    let outcome = addr
        .reload(1, false, |_old| Box::new(CounterV1 { n: 5 }))
        .unwrap();
    match outcome {
        ReloadOutcome::DowngradeRefused { from, to } => {
            assert_eq!(from, 2);
            assert_eq!(to, 1);
        }
        other => panic!("expected DowngradeRefused, got {other:?}"),
    }

    // The actor must still be on v2 — the refused reload should not have
    // mutated state.
    assert_eq!(addr.try_ask("ping".to_string()).expect("ask"), "v2[live]:5");
}

#[test]
fn backward_migration_allowed_when_allow_downgrade_is_set() {
    let rt = Runtime::new();
    let addr = rt.spawn(CounterV2 {
        n: 7,
        label: "live".to_string(),
    });

    let outcome = addr
        .reload(1, true, |_old| Box::new(CounterV1 { n: 7 }))
        .unwrap();
    assert!(matches!(outcome, ReloadOutcome::Ok { .. }));

    assert_eq!(addr.try_ask("ping".to_string()).expect("ask"), "v1:7");
}

// ── Test 4: In-flight handler safety ────────────────────────────────

struct SlowActor {
    handled_count: Arc<AtomicI64>,
}

impl Actor for SlowActor {
    type Message = ();
    type Reply = ();
    fn handle(&mut self, _: ()) {
        // Intentionally slow: sleep so that a reload sent immediately after
        // arrives while we're still handling.
        std::thread::sleep(Duration::from_millis(80));
        self.handled_count.fetch_add(1, Ordering::SeqCst);
    }
    fn schema_version(&self) -> u32 {
        1
    }
}

struct FastActor {
    handled_count: Arc<AtomicI64>,
}

impl Actor for FastActor {
    type Message = ();
    type Reply = ();
    fn handle(&mut self, _: ()) {
        self.handled_count.fetch_add(100, Ordering::SeqCst);
    }
    fn schema_version(&self) -> u32 {
        2
    }
}

#[test]
fn reload_waits_for_in_flight_handler_to_complete() {
    let rt = Runtime::new();
    let counter = Arc::new(AtomicI64::new(0));
    let addr = rt.spawn(SlowActor {
        handled_count: Arc::clone(&counter),
    });

    // Fire a slow message, then enqueue a reload immediately. The reload
    // command sits behind the slow handler in the mailbox; it must NOT
    // run until the handler returns. After reload, fire another message
    // which should be handled by the FastActor.
    addr.tell(()); // slow: takes ~80ms
    let counter_for_migrator = Arc::clone(&counter);
    let outcome = addr
        .reload(2, false, move |_old| {
            Box::new(FastActor {
                handled_count: Arc::clone(&counter_for_migrator),
            })
        })
        .unwrap();
    assert!(matches!(outcome, ReloadOutcome::Ok { .. }));

    // Counter should be 1 (slow handler ran to completion before reload swapped).
    let observed_after_reload = counter.load(Ordering::SeqCst);
    assert_eq!(
        observed_after_reload, 1,
        "the slow v1 handler must have completed before reload swapped behaviour"
    );

    // Fire one more message under the new behaviour.
    let _ = addr.ask_timeout((), Duration::from_secs(2));
    assert_eq!(counter.load(Ordering::SeqCst), 101);
}

// ── Test 5: StateCert — fingerprint-verified downcast refuses wrong type ──

#[test]
fn statecert_rejects_wrong_downcast_type_without_panic() {
    // v3.3 Security Layer 1 proof: a migrator that attempts to downcast
    // the extracted state to the WRONG type must receive a structured
    // FingerprintMismatch error — never a silent panic, never a
    // layout-compatible coercion.
    //
    // We reload from CounterV1 (state = i64) and the migrator deliberately
    // asks for u64 — same size and alignment as i64 on every platform
    // Garnet supports, but a different type identity. v3.2's
    // `Box<dyn Any>` would have returned Err from downcast silently
    // and panicked via .unwrap(); with StateCert, the fingerprint
    // comparison rejects the downcast loudly with a readable hex diff.

    let rt = Runtime::new();
    let addr = rt.spawn(CounterV1 { n: 42 });
    assert_eq!(addr.try_ask("ping".to_string()).expect("ask"), "v1:42");

    // Capture the mismatch outcome from inside the migrator via a
    // channel so the test can assert on it.
    use std::sync::mpsc;
    let (tx, rx) = mpsc::channel::<String>();

    let outcome = addr
        .reload(2, false, move |old| {
            let tagged = old
                .dyn_extract_state()
                .expect("CounterV1 exposes extract_state");
            // Deliberately wrong downcast type. i64 and u64 share size
            // and alignment — exactly the layout-compatible-but-distinct
            // scenario StateCert is built to reject.
            let result = tagged.downcast::<u64>();
            match result {
                Ok(_) => {
                    let _ = tx.send("downcast unexpectedly succeeded".to_string());
                    // Keep the contract: return *something* so reload
                    // doesn't hang. Use a fresh CounterV2 with dummy state.
                    Box::new(CounterV2 {
                        n: 0,
                        label: "unexpected".to_string(),
                    })
                }
                Err(mismatch) => {
                    let _ = tx.send(format!("{mismatch}"));
                    Box::new(CounterV2 {
                        n: 42,
                        label: "mismatch-recovered".to_string(),
                    })
                }
            }
        })
        .unwrap();
    assert!(matches!(outcome, ReloadOutcome::Ok { .. }));

    // The migrator closure ran and reported the mismatch error.
    let reported = rx.recv().expect("migrator must report mismatch");
    assert!(
        reported.starts_with("type fingerprint mismatch"),
        "expected FingerprintMismatch diagnostic, got: {reported}"
    );
    // Hex expected fingerprint (of u64) and actual fingerprint (of i64)
    // must both be present in the error.
    assert!(
        reported.contains("expected"),
        "error must label the expected fingerprint: {reported}"
    );
    assert!(
        reported.contains("got"),
        "error must label the actual fingerprint: {reported}"
    );

    // The actor recovered and is on v2 with the migrated-after-error
    // value; if the downcast had panicked instead, this assertion would
    // fail because the runtime thread would have died.
    assert_eq!(
        addr.try_ask("ping".to_string()).expect("ask"),
        "v2[mismatch-recovered]:42"
    );
}

#[test]
fn statecert_fingerprints_are_stable_within_a_run() {
    // Every call to `extract_state` on the same actor produces the same
    // fingerprint — determinism property used by migrators that want to
    // compare fingerprints across the reload boundary without running
    // the full downcast.
    use garnet_actor_runtime::TypeFingerprint;
    let actor = CounterV1 { n: 1 };
    let first = <CounterV1 as Actor>::extract_state(&actor)
        .expect("CounterV1 produces TaggedState")
        .fingerprint();
    let second = <CounterV1 as Actor>::extract_state(&actor)
        .expect("CounterV1 produces TaggedState")
        .fingerprint();
    assert_eq!(first, second);
    // And matches the expected fingerprint for i64.
    assert_eq!(first, TypeFingerprint::of::<i64>());
}
