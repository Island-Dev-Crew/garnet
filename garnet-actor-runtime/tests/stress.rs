//! Stress tests for the actor runtime. `#[ignore]` by default.

use garnet_actor_runtime::{Actor, AskError, Runtime};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use std::time::Duration;

struct Adder {
    counter: Arc<AtomicI64>,
}

impl Actor for Adder {
    type Message = i64;
    type Reply = i64;
    fn handle(&mut self, n: i64) -> i64 {
        self.counter.fetch_add(n, Ordering::SeqCst) + n
    }
}

#[test]
#[ignore]
fn two_hundred_actors_x_one_thousand_messages() {
    let rt = Runtime::new();
    let counter = Arc::new(AtomicI64::new(0));
    let mut addrs = Vec::new();
    for _ in 0..200 {
        addrs.push(rt.spawn(Adder {
            counter: Arc::clone(&counter),
        }));
    }
    for addr in &addrs {
        for _ in 0..1000 {
            addr.tell(1);
        }
    }
    // Ask the last actor for its accumulated view to flush all messages.
    for addr in &addrs {
        let _ = addr.try_ask(0).expect("ask");
    }
    assert_eq!(counter.load(Ordering::SeqCst), 200_000);
}

// Slow handler must be terminable via ask_timeout so a circular call chain
// or any hung handler can never hang the test runner.
struct SlowActor;
impl Actor for SlowActor {
    type Message = ();
    type Reply = ();
    fn handle(&mut self, _: ()) {
        std::thread::sleep(Duration::from_millis(500));
    }
}

#[test]
#[ignore]
fn ask_timeout_terminates_slow_handlers() {
    let rt = Runtime::new();
    let slow = rt.spawn(SlowActor);
    let r = slow.ask_timeout((), Duration::from_millis(20));
    assert!(matches!(r, Err(AskError::Timeout)));
}
