//! BoundedMail tests (v3.4 Security Layer 2).
//!
//! Closes the unbounded-mailbox DOS class introduced by v0.3's
//! `mpsc::channel` (Mini-Spec §9 / Security Threat Model #8).
//!
//! Default mailbox capacity is 1024; per-actor override via
//! `Actor::mailbox_capacity()`; per-spawn override via
//! `Runtime::spawn_with_capacity`. `tell` blocks on full; `try_tell`
//! returns `SendError::Full` non-blocking.

use garnet_actor_runtime::{Actor, Runtime, SendError, DEFAULT_MAILBOX_CAPACITY};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

// ─── A baseline tiny actor used across tests ───
//
// The actor's handler intentionally sleeps so the mailbox can
// build up without being drained. `started` and `processed`
// counters let tests reason about ordering.

struct SlowEcho {
    processed: Arc<AtomicI64>,
    sleep_ms: u64,
    capacity: usize,
}

enum Msg {
    Tick,
    Get,
}

enum Reply {
    Acked,
    Count(i64),
}

impl Actor for SlowEcho {
    type Message = Msg;
    type Reply = Reply;

    fn handle(&mut self, m: Msg) -> Reply {
        match m {
            Msg::Tick => {
                if self.sleep_ms > 0 {
                    thread::sleep(Duration::from_millis(self.sleep_ms));
                }
                self.processed.fetch_add(1, Ordering::SeqCst);
                Reply::Acked
            }
            Msg::Get => Reply::Count(self.processed.load(Ordering::SeqCst)),
        }
    }

    fn mailbox_capacity(&self) -> usize {
        self.capacity
    }
}

// ─── Default capacity test ───

#[test]
fn default_mailbox_capacity_constant_is_1024() {
    assert_eq!(DEFAULT_MAILBOX_CAPACITY, 1024);
}

// ─── try_tell happy path ───

#[test]
fn try_tell_succeeds_when_mailbox_has_room() {
    let rt = Runtime::new();
    let processed = Arc::new(AtomicI64::new(0));
    let addr = rt.spawn(SlowEcho {
        processed: Arc::clone(&processed),
        sleep_ms: 0,
        capacity: 1024,
    });

    for _ in 0..10 {
        addr.try_tell(Msg::Tick).expect("should fit easily");
    }

    // Drain by issuing an ask — when this returns, all prior fire-and-
    // forgets have been processed.
    let r = addr.try_ask(Msg::Get).expect("ask");
    match r {
        Reply::Count(n) => assert_eq!(n, 10),
        _ => panic!("unexpected reply"),
    }
}

// ─── try_tell rejects when full ───

#[test]
fn try_tell_returns_full_when_mailbox_at_capacity() {
    let rt = Runtime::new();
    let processed = Arc::new(AtomicI64::new(0));
    // capacity=4, slow handler so the mailbox fills before any drains
    let addr = rt.spawn(SlowEcho {
        processed: Arc::clone(&processed),
        sleep_ms: 100,
        capacity: 4,
    });

    // Burst-send: first few may be accepted, eventually we hit capacity.
    // Because the receiver may be mid-handle, we accept either 4 or 5
    // accepted before the first Full (one in-flight on the handler thread,
    // capacity-many buffered).
    let mut accepted = 0usize;
    let mut hit_full = false;
    for _ in 0..16 {
        match addr.try_tell(Msg::Tick) {
            Ok(()) => accepted += 1,
            Err(SendError::Full) => {
                hit_full = true;
                break;
            }
            Err(e) => panic!("unexpected error: {e}"),
        }
    }
    assert!(hit_full, "expected SendError::Full once mailbox saturated");
    assert!(
        (4..=5).contains(&accepted),
        "expected ~capacity messages accepted before Full, got {accepted}"
    );
}

// ─── try_tell returns Closed when actor's receiver is dropped ───
//
// NOTE on API limitation: the v3.4 actor runtime has no
// programmatic "stop this actor" command — actors terminate only when
// every Sender (i.e., every ActorAddress clone) is dropped, at which
// point `rx.recv()` returns Err and the loop exits. This test
// therefore induces "Closed" by spawning a very fast actor that
// processes one message and then we let it run to completion via a
// drop-then-clone-from-prior-send pattern.
//
// We exercise the SendError::Closed branch by:
// 1. Spawning an actor and getting its address.
// 2. Sending a message that we know will cause the actor to crash
//    via panic (a managed-mode panic in handle terminates the thread,
//    which drops the receiver, which closes the channel from the
//    sender's perspective).
//
// Implementation note: in v3.5 we'll add `Runtime::stop(addr)` for a
// clean shutdown semantic. For now this panic-induced shutdown is
// the only way to observe the Closed path without holding-Sender
// deadlock, and serves the BoundedMail correctness obligation.

struct PanickyActor;
enum PMsg {
    Crash,
}
#[allow(dead_code)]
enum PReply {
    Never,
}
impl Actor for PanickyActor {
    type Message = PMsg;
    type Reply = PReply;
    fn handle(&mut self, _: PMsg) -> PReply {
        panic!("intentional panic for Closed test")
    }
    fn mailbox_capacity(&self) -> usize {
        4
    }
}

#[test]
fn try_tell_returns_closed_after_actor_panics() {
    let rt = Runtime::new();
    let addr = rt.spawn(PanickyActor);

    // Send the panic-trigger via tell (which blocks for capacity, accepts immediately).
    let _ = addr.tell(PMsg::Crash);

    // The actor thread panics, drops its receiver. We poll try_tell until
    // we observe Closed (or timeout).
    let mut closed = false;
    for _ in 0..100 {
        thread::sleep(Duration::from_millis(20));
        match addr.try_tell(PMsg::Crash) {
            Err(SendError::Closed) => {
                closed = true;
                break;
            }
            Err(SendError::Full) => continue, // mailbox still has buffered msgs
            Ok(()) => continue,               // not yet panicked — try again
        }
    }
    assert!(closed, "expected SendError::Closed within 2s after panic");
}

// ─── Capacity recovery: once a slot frees, try_tell succeeds again ───

#[test]
fn try_tell_succeeds_again_after_drain() {
    let rt = Runtime::new();
    let processed = Arc::new(AtomicI64::new(0));
    let addr = rt.spawn(SlowEcho {
        processed: Arc::clone(&processed),
        sleep_ms: 30,
        capacity: 4,
    });

    // Saturate
    for _ in 0..16 {
        if addr.try_tell(Msg::Tick).is_err() {
            break;
        }
    }

    // Wait for the actor to drain a few
    thread::sleep(Duration::from_millis(150));

    // Now try_tell should succeed again
    let mut succeeded_again = false;
    for _ in 0..8 {
        if addr.try_tell(Msg::Tick).is_ok() {
            succeeded_again = true;
            break;
        }
        thread::sleep(Duration::from_millis(30));
    }
    assert!(
        succeeded_again,
        "expected try_tell to succeed once actor drained"
    );
}

// ─── spawn_with_capacity overrides Actor::mailbox_capacity ───

#[test]
fn spawn_with_capacity_overrides_default() {
    let rt = Runtime::new();
    let processed = Arc::new(AtomicI64::new(0));
    // Actor's own mailbox_capacity says 1024, but spawn_with_capacity(2)
    // wins. We saturate quickly with sleep_ms=200.
    let addr = rt.spawn_with_capacity(
        SlowEcho {
            processed: Arc::clone(&processed),
            sleep_ms: 200,
            capacity: 1024, // would-be default; overridden below
        },
        2,
    );

    let mut accepted = 0;
    let mut hit_full = false;
    for _ in 0..10 {
        match addr.try_tell(Msg::Tick) {
            Ok(()) => accepted += 1,
            Err(SendError::Full) => {
                hit_full = true;
                break;
            }
            Err(_) => panic!(),
        }
    }
    assert!(hit_full, "spawn_with_capacity(2) should saturate fast");
    assert!(
        (2..=3).contains(&accepted),
        "expected ~2 accepted before Full, got {accepted}"
    );
}

// ─── tell still works (back-compat) and blocks rather than dropping ───

#[test]
fn tell_returns_true_on_accepted_send() {
    let rt = Runtime::new();
    let processed = Arc::new(AtomicI64::new(0));
    let addr = rt.spawn(SlowEcho {
        processed,
        sleep_ms: 0,
        capacity: 8,
    });

    for _ in 0..6 {
        assert!(addr.tell(Msg::Tick), "tell should accept under capacity");
    }
}

// ─── Per-actor override via Actor::mailbox_capacity ───

struct TinyActor {
    cap: usize,
    n: Arc<AtomicI64>,
}

enum TMsg {
    Bump,
    Read,
}

enum TReply {
    Ok,
    N(i64),
}

impl Actor for TinyActor {
    type Message = TMsg;
    type Reply = TReply;

    fn handle(&mut self, m: TMsg) -> TReply {
        match m {
            TMsg::Bump => {
                thread::sleep(Duration::from_millis(50));
                self.n.fetch_add(1, Ordering::SeqCst);
                TReply::Ok
            }
            TMsg::Read => TReply::N(self.n.load(Ordering::SeqCst)),
        }
    }

    fn mailbox_capacity(&self) -> usize {
        self.cap
    }
}

#[test]
fn actor_mailbox_capacity_method_is_honored_by_default_spawn() {
    let rt = Runtime::new();
    let n = Arc::new(AtomicI64::new(0));
    // cap=1 is intentionally tiny — first few try_tells will hit Full
    // immediately because the handler sleeps 50ms.
    let addr = rt.spawn(TinyActor {
        cap: 1,
        n: Arc::clone(&n),
    });

    let mut accepted = 0;
    let mut hit_full = false;
    for _ in 0..8 {
        match addr.try_tell(TMsg::Bump) {
            Ok(()) => accepted += 1,
            Err(SendError::Full) => {
                hit_full = true;
                break;
            }
            Err(_) => panic!(),
        }
    }
    assert!(
        hit_full,
        "cap=1 should saturate with try_tell almost immediately"
    );
    let TReply::N(observed) = addr.try_ask(TMsg::Read).expect("ask") else {
        panic!("read should return the actor count");
    };
    assert!(
        observed >= accepted,
        "read should observe at least the accepted bumps"
    );
    assert!(
        (1..=2).contains(&accepted),
        "expected 1 or 2 accepted before Full, got {accepted}"
    );
}

// ─── No starvation under N concurrent senders ───

#[test]
fn many_senders_sharing_one_actor_eventually_all_succeed() {
    let rt = Runtime::new();
    let processed = Arc::new(AtomicI64::new(0));
    let addr = rt.spawn(SlowEcho {
        processed: Arc::clone(&processed),
        sleep_ms: 0, // no artificial slowdown — pure throughput
        capacity: 64,
    });

    let mut handles = Vec::new();
    let target_per_sender = 50;
    let n_senders = 8;
    for _ in 0..n_senders {
        let a = addr.clone();
        handles.push(thread::spawn(move || {
            for _ in 0..target_per_sender {
                while a.try_tell(Msg::Tick).is_err() {
                    thread::sleep(Duration::from_millis(1));
                }
            }
        }));
    }
    for h in handles {
        h.join().unwrap();
    }

    // Final ask drains; all sends should be accounted for.
    match addr.try_ask(Msg::Get).expect("ask") {
        Reply::Count(n) => assert_eq!(n, (n_senders * target_per_sender) as i64),
        _ => panic!(),
    }
}

// ─── Existing 1000-message test should still pass under the default cap ───

#[test]
fn one_thousand_tells_succeed_under_default_cap() {
    // Reproduces runtime.rs::one_thousand_messages_processed_in_order_per_actor
    // but using `tell` (which now blocks on full) — we exercise the fact
    // that with cap=1024, 1000 sends fit even with no concurrent draining.
    let rt = Runtime::new();
    let processed = Arc::new(AtomicI64::new(0));
    let addr = rt.spawn(SlowEcho {
        processed: Arc::clone(&processed),
        sleep_ms: 0,
        capacity: DEFAULT_MAILBOX_CAPACITY,
    });

    for _ in 0..1000 {
        assert!(addr.tell(Msg::Tick));
    }
    match addr.try_ask(Msg::Get).expect("ask") {
        Reply::Count(n) => assert_eq!(n, 1000),
        _ => panic!(),
    }
}
