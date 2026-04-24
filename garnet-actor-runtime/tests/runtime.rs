//! End-to-end tests for the actor runtime: tell/ask, multi-actor, shutdown.

use garnet_actor_runtime::{Actor, AskError, Runtime};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use std::time::Duration;

// ── Counter actor: integer state, increment + read ──

struct Counter {
    n: i64,
}

enum CMsg {
    Incr,
    Add(i64),
    Get,
}

enum CReply {
    Ok,
    Value(i64),
}

impl Actor for Counter {
    type Message = CMsg;
    type Reply = CReply;
    fn handle(&mut self, msg: CMsg) -> CReply {
        match msg {
            CMsg::Incr => {
                self.n += 1;
                CReply::Ok
            }
            CMsg::Add(v) => {
                self.n += v;
                CReply::Ok
            }
            CMsg::Get => CReply::Value(self.n),
        }
    }
}

#[test]
fn counter_actor_handles_incr_and_get() {
    let rt = Runtime::new();
    let addr = rt.spawn(Counter { n: 0 });
    addr.tell(CMsg::Incr);
    addr.tell(CMsg::Incr);
    addr.tell(CMsg::Incr);
    let r = addr.ask(CMsg::Get);
    assert!(matches!(r, CReply::Value(3)));
}

#[test]
fn counter_actor_add_message() {
    let rt = Runtime::new();
    let addr = rt.spawn(Counter { n: 0 });
    addr.tell(CMsg::Add(10));
    addr.tell(CMsg::Add(20));
    addr.tell(CMsg::Add(-5));
    let r = addr.ask(CMsg::Get);
    assert!(matches!(r, CReply::Value(25)));
}

#[test]
fn ask_blocks_until_reply() {
    let rt = Runtime::new();
    let addr = rt.spawn(Counter { n: 100 });
    let r = addr.ask(CMsg::Get);
    assert!(matches!(r, CReply::Value(100)));
}

#[test]
fn ask_timeout_returns_value_when_under_deadline() {
    let rt = Runtime::new();
    let addr = rt.spawn(Counter { n: 7 });
    let r = addr.ask_timeout(CMsg::Get, Duration::from_secs(2)).unwrap();
    assert!(matches!(r, CReply::Value(7)));
}

// ── Multi-actor scenario ──

struct Forwarder {
    target: garnet_actor_runtime::ActorAddress<CMsg, CReply>,
}

enum FMsg {
    Bump,
}

enum FReply {
    Done,
}

impl Actor for Forwarder {
    type Message = FMsg;
    type Reply = FReply;
    fn handle(&mut self, msg: FMsg) -> FReply {
        match msg {
            FMsg::Bump => {
                self.target.tell(CMsg::Incr);
                FReply::Done
            }
        }
    }
}

#[test]
fn forwarder_sends_to_counter() {
    let rt = Runtime::new();
    let counter = rt.spawn(Counter { n: 0 });
    let forwarder = rt.spawn(Forwarder {
        target: counter.clone(),
    });
    for _ in 0..5 {
        forwarder.ask(FMsg::Bump);
    }
    let r = counter.ask(CMsg::Get);
    assert!(matches!(r, CReply::Value(5)));
}

// ── Shared atomic state across actors ──

struct Tally {
    sum: Arc<AtomicI64>,
}

enum TMsg {
    Add(i64),
}

enum TReply {
    Total(i64),
}

impl Actor for Tally {
    type Message = TMsg;
    type Reply = TReply;
    fn handle(&mut self, msg: TMsg) -> TReply {
        match msg {
            TMsg::Add(v) => {
                let prev = self.sum.fetch_add(v, Ordering::SeqCst);
                TReply::Total(prev + v)
            }
        }
    }
}

#[test]
fn three_tally_actors_share_atomic_state() {
    let rt = Runtime::new();
    let sum = Arc::new(AtomicI64::new(0));
    let a = rt.spawn(Tally {
        sum: Arc::clone(&sum),
    });
    let b = rt.spawn(Tally {
        sum: Arc::clone(&sum),
    });
    let c = rt.spawn(Tally {
        sum: Arc::clone(&sum),
    });
    let TReply::Total(_) = a.ask(TMsg::Add(5));
    let TReply::Total(_) = b.ask(TMsg::Add(10));
    let TReply::Total(t) = c.ask(TMsg::Add(15));
    // Last reply observes the cumulative atomic value.
    assert_eq!(t, 30);
    assert_eq!(sum.load(Ordering::SeqCst), 30);
}

// ── Stats ──

#[test]
fn runtime_stats_count_spawned_actors() {
    let rt = Runtime::new();
    let _a = rt.spawn(Counter { n: 0 });
    let _b = rt.spawn(Counter { n: 0 });
    let _c = rt.spawn(Counter { n: 0 });
    let stats = rt.stats();
    assert_eq!(stats.spawned, 3);
    // running is always >= 0 (usize); confirm we still observe at most 3
    // outstanding actors. Stopped count may already be > 0 if a thread
    // raced; that's fine.
    assert!(stats.running <= 3);
}

#[test]
fn runtime_stats_show_stopped_after_drop() {
    let rt = Runtime::new();
    {
        let _addr = rt.spawn(Counter { n: 0 });
        // address dropped here closes the mailbox.
    }
    // Allow the actor thread to observe the closed channel and stop.
    std::thread::sleep(Duration::from_millis(50));
    let stats = rt.stats();
    assert_eq!(stats.spawned, 1);
    assert_eq!(stats.stopped, 1);
}

// ── Cloning addresses ──

#[test]
fn cloned_address_targets_same_actor() {
    let rt = Runtime::new();
    let a = rt.spawn(Counter { n: 0 });
    let b = a.clone();
    a.tell(CMsg::Incr);
    b.tell(CMsg::Incr);
    let r = a.ask(CMsg::Get);
    assert!(matches!(r, CReply::Value(2)));
}

// ── Many messages ──

#[test]
fn one_thousand_messages_processed_in_order_per_actor() {
    let rt = Runtime::new();
    let addr = rt.spawn(Counter { n: 0 });
    for _ in 0..1000 {
        addr.tell(CMsg::Incr);
    }
    let r = addr.ask(CMsg::Get);
    assert!(matches!(r, CReply::Value(1000)));
}

// ── Lifecycle hooks ──

struct Lifecycle {
    started: Arc<AtomicI64>,
    stopped: Arc<AtomicI64>,
}

enum LMsg {
    Ping,
}

enum LReply {
    Pong,
}

impl Actor for Lifecycle {
    type Message = LMsg;
    type Reply = LReply;
    fn handle(&mut self, _: LMsg) -> LReply {
        LReply::Pong
    }
    fn on_start(&mut self) {
        self.started.fetch_add(1, Ordering::SeqCst);
    }
    fn on_stop(&mut self) {
        self.stopped.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn on_start_runs_when_actor_spawns() {
    let rt = Runtime::new();
    let started = Arc::new(AtomicI64::new(0));
    let stopped = Arc::new(AtomicI64::new(0));
    let addr = rt.spawn(Lifecycle {
        started: Arc::clone(&started),
        stopped: Arc::clone(&stopped),
    });
    let _ = addr.ask(LMsg::Ping); // ensure actor processed at least one message
    assert_eq!(started.load(Ordering::SeqCst), 1);
}

#[test]
fn on_stop_runs_when_mailbox_closes() {
    let rt = Runtime::new();
    let stopped = Arc::new(AtomicI64::new(0));
    {
        let _addr = rt.spawn(Lifecycle {
            started: Arc::new(AtomicI64::new(0)),
            stopped: Arc::clone(&stopped),
        });
    }
    std::thread::sleep(Duration::from_millis(50));
    assert_eq!(stopped.load(Ordering::SeqCst), 1);
}

// ── Errors ──

#[test]
fn ask_timeout_returns_timeout_when_late() {
    // Construct a slow actor whose handle blocks for longer than the timeout.
    struct Slow;
    impl Actor for Slow {
        type Message = ();
        type Reply = ();
        fn handle(&mut self, _: ()) {
            std::thread::sleep(Duration::from_millis(500));
        }
    }
    let rt = Runtime::new();
    let addr = rt.spawn(Slow);
    let r = addr.ask_timeout((), Duration::from_millis(5));
    assert!(matches!(r, Err(AskError::Timeout)));
}
