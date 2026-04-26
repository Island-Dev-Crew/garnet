//! # Garnet Actor Runtime (v0.4.0)
//!
//! Rung 6 concurrent execution surface. Each actor runs on a dedicated OS
//! thread, receives messages over an `mpsc::Sender`, and returns replies on
//! per-message reply channels. The design is intentionally small: it is a
//! reference scheduler that proves the actor model is operational, not a
//! production work-stealing runtime (that's a separable extension).
//!
//! The runtime is independent of the interpreter — it shuffles user-defined
//! payloads, not Garnet `Value` instances. The interpreter wraps it through
//! the `garnet-actor-runtime-bridge` adapter (future work).
//!
//! ## Usage
//!
//! ```
//! use garnet_actor_runtime::{Actor, Runtime};
//! use std::sync::atomic::{AtomicI64, Ordering};
//! use std::sync::Arc;
//!
//! struct Counter { n: Arc<AtomicI64> }
//! enum Msg { Incr, Get }
//! enum Reply { Ok, Value(i64) }
//!
//! impl Actor for Counter {
//!     type Message = Msg;
//!     type Reply = Reply;
//!     fn handle(&mut self, msg: Msg) -> Reply {
//!         match msg {
//!             Msg::Incr => { self.n.fetch_add(1, Ordering::SeqCst); Reply::Ok }
//!             Msg::Get => Reply::Value(self.n.load(Ordering::SeqCst)),
//!         }
//!     }
//! }
//!
//! let rt = Runtime::new();
//! let n = Arc::new(AtomicI64::new(0));
//! let addr = rt.spawn(Counter { n: Arc::clone(&n) });
//! addr.tell(Msg::Incr);
//! addr.tell(Msg::Incr);
//! let r = addr.try_ask(Msg::Get).expect("counter responded");
//! assert!(matches!(r, Reply::Value(2)));
//! ```

pub mod address;
pub mod reloadkey;
pub mod runtime;
pub mod statecert;

pub use address::{ActorAddress, AskError, SendError};
pub use reloadkey::{
    derive_actor_id, generate_keypair, signing_key_from_hex, ReloadAuth, ReloadReplayGuard,
    RELOAD_SIGNATURE_MAGIC,
};
pub use runtime::{
    Actor, ActorBehaviour, ReloadOutcome, Runtime, RuntimeStats, DEFAULT_MAILBOX_CAPACITY,
};
pub use statecert::{FingerprintMismatch, TaggedState, TypeFingerprint};
