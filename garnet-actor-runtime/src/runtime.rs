//! Spawn-and-mailbox runtime: each actor gets one OS thread plus a mpsc
//! receiver that drives the handler loop.
//!
//! v3.2 adds **hot-reload** (Paper VI Contribution 6): the runtime can
//! replace an actor's behaviour at runtime, drain pending mailbox traffic,
//! invoke a user-supplied migrator with the old behaviour, install the new
//! behaviour, and replay the buffered messages. Schema versions guard
//! accidental downgrades.

use crate::address::{ActorAddress, Envelope};
use crate::statecert::TaggedState;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread::{self, JoinHandle};

/// Default mailbox capacity (v3.4 BoundedMail / Security Layer 2).
///
/// Closes the unbounded-mailbox DOS class: a misbehaving sender can no
/// longer OOM a receiver simply by sending faster than the receiver
/// drains. Backpressure surfaces as either a blocking `tell` (when the
/// caller used the simple path) or a `SendError::Full` from `try_tell`
/// (when the caller wants to handle overflow explicitly).
///
/// Override per-actor via `Actor::mailbox_capacity`.
pub const DEFAULT_MAILBOX_CAPACITY: usize = 1024;

/// User-implemented behaviour for an actor: take ownership of `self`, see one
/// message at a time, and produce a reply value. The default `on_start` /
/// `on_stop` hooks are no-ops.
///
/// `schema_version` is consulted at hot-reload time. The default of 0 means
/// "unversioned" — actors that opt into hot-reload should override this to
/// a monotonically increasing integer.
pub trait Actor: Send + 'static {
    type Message: Send + 'static;
    type Reply: Send + 'static;

    fn handle(&mut self, message: Self::Message) -> Self::Reply;

    fn on_start(&mut self) {}

    fn on_stop(&mut self) {}

    /// Schema version of this actor's state. Used by `Runtime::reload` to
    /// reject downgrades unless `allow_downgrade` is set.
    fn schema_version(&self) -> u32 {
        0
    }

    /// Extract this actor's migratable state for hot-reload (Paper VI
    /// Contribution 6). Default returns `None` — actors that support
    /// reload-with-state override this to return
    /// `Some(TaggedState::new(self.clone_state()))`.
    ///
    /// v3.3 Security Layer 1 (StateCert): the returned `TaggedState`
    /// carries a BLAKE3 fingerprint derived from the Rust type's
    /// identity — specifically its name, size, and alignment.
    /// Migrators call `.downcast::<T>()`, which refuses the downcast
    /// on fingerprint mismatch with a structured `FingerprintMismatch`
    /// error — never panics. Closes the type-confusion cliff that the
    /// v3.2 `Box<dyn Any>` approach would have opened once hot-reload
    /// reached any external channel.
    fn extract_state(&self) -> Option<TaggedState> {
        None
    }

    /// Mailbox capacity for this actor (v3.4 BoundedMail). Default is
    /// `DEFAULT_MAILBOX_CAPACITY` (1024). Override to a smaller value for
    /// memory-constrained agents or a larger value for high-throughput
    /// pipelines whose downstream actor can drain faster than 1024
    /// messages per receiver tick.
    ///
    /// The capacity is enforced via `mpsc::sync_channel` — `tell` blocks
    /// when the mailbox is full; `try_tell` returns `SendError::Full`
    /// without blocking.
    fn mailbox_capacity(&self) -> usize {
        DEFAULT_MAILBOX_CAPACITY
    }
}

/// Object-safe internal dyn-trait projection of `Actor` so the runtime can
/// hold the actor as `Box<dyn ActorBehaviour<M, R>>` and swap it at reload.
/// Users should not implement this trait directly — the blanket impl below
/// covers every `Actor` implementor.
pub trait ActorBehaviour<M, R>: Send + 'static {
    fn dyn_handle(&mut self, msg: M) -> R;
    fn dyn_on_start(&mut self);
    fn dyn_on_stop(&mut self);
    fn dyn_schema_version(&self) -> u32;
    fn dyn_extract_state(&self) -> Option<TaggedState>;
    fn dyn_mailbox_capacity(&self) -> usize;
}

impl<A> ActorBehaviour<A::Message, A::Reply> for A
where
    A: Actor,
{
    fn dyn_handle(&mut self, msg: A::Message) -> A::Reply {
        Actor::handle(self, msg)
    }
    fn dyn_on_start(&mut self) {
        Actor::on_start(self);
    }
    fn dyn_on_stop(&mut self) {
        Actor::on_stop(self);
    }
    fn dyn_schema_version(&self) -> u32 {
        Actor::schema_version(self)
    }
    fn dyn_extract_state(&self) -> Option<TaggedState> {
        Actor::extract_state(self)
    }
    fn dyn_mailbox_capacity(&self) -> usize {
        Actor::mailbox_capacity(self)
    }
}

/// Outcome of a hot-reload attempt, returned to the caller via the reply
/// channel embedded in the reload command.
#[derive(Debug)]
pub enum ReloadOutcome {
    /// Reload completed: the migrator ran, the new behaviour is installed,
    /// and `replayed` previously-buffered messages have been processed.
    Ok { replayed: usize },
    /// Reload was refused because the requested target version is lower
    /// than the current one and `allow_downgrade` was false.
    DowngradeRefused { from: u32, to: u32 },
}

/// Boxed migrator function: takes ownership of the old behaviour, returns
/// the new one. Used in `ReloadCommand`.
pub type MigratorFn<M, R> =
    Box<dyn FnOnce(Box<dyn ActorBehaviour<M, R>>) -> Box<dyn ActorBehaviour<M, R>> + Send>;

/// Hot-reload command sent through the actor's control channel. The
/// migrator takes ownership of the old behaviour and returns the new one
/// (typically constructed from the old's state). `target_version` is
/// declared up-front so the runtime can refuse a downgrade without running
/// the migrator (which is potentially expensive).
pub(crate) struct ReloadCommand<M, R> {
    pub target_version: u32,
    pub allow_downgrade: bool,
    pub migrator: MigratorFn<M, R>,
    pub reply: mpsc::Sender<ReloadOutcome>,
}

/// Internal control envelope: every message that reaches the actor loop is
/// either a user payload or a reload command. Hidden from the public API.
pub(crate) enum Control<M, R> {
    User(Envelope<M, R>),
    Reload(ReloadCommand<M, R>),
}

/// Aggregate counters across the runtime — useful for tests and for the
/// CLI's `stats` subcommand later on.
#[derive(Debug, Clone, Default)]
pub struct RuntimeStats {
    pub spawned: usize,
    pub running: usize,
    pub stopped: usize,
}

/// The runtime owns the JoinHandles + counters for every actor it spawned.
pub struct Runtime {
    spawned: Arc<AtomicUsize>,
    stopped: Arc<AtomicUsize>,
    handles: std::sync::Mutex<Vec<JoinHandle<()>>>,
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            spawned: Arc::new(AtomicUsize::new(0)),
            stopped: Arc::new(AtomicUsize::new(0)),
            handles: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// Spawn an actor on a fresh OS thread. The returned address can be
    /// cloned and shared.
    ///
    /// v3.4 BoundedMail: the mailbox is bounded at `Actor::mailbox_capacity()`
    /// (default 1024). `tell` blocks if the mailbox is full; `try_tell`
    /// returns `SendError::Full` without blocking.
    pub fn spawn<A: Actor>(&self, actor: A) -> ActorAddress<A::Message, A::Reply> {
        let capacity = actor.mailbox_capacity();
        self.spawn_with_capacity(actor, capacity)
    }

    /// Spawn with an explicit capacity, overriding `Actor::mailbox_capacity()`.
    /// Useful for tests that exercise specific bounded-mailbox scenarios.
    pub fn spawn_with_capacity<A: Actor>(
        &self,
        actor: A,
        capacity: usize,
    ) -> ActorAddress<A::Message, A::Reply> {
        let (tx, rx) = mpsc::sync_channel::<Control<A::Message, A::Reply>>(capacity);
        let stopped = Arc::clone(&self.stopped);
        self.spawned.fetch_add(1, Ordering::SeqCst);
        let handle = thread::spawn(move || {
            let boxed: Box<dyn ActorBehaviour<A::Message, A::Reply>> = Box::new(actor);
            run_actor_loop(boxed, rx);
            stopped.fetch_add(1, Ordering::SeqCst);
        });
        self.handles.lock().unwrap().push(handle);
        ActorAddress { tx }
    }

    pub fn stats(&self) -> RuntimeStats {
        let spawned = self.spawned.load(Ordering::SeqCst);
        let stopped = self.stopped.load(Ordering::SeqCst);
        RuntimeStats {
            spawned,
            stopped,
            running: spawned.saturating_sub(stopped),
        }
    }

    /// Wait for every spawned actor to finish (called when all addresses for
    /// each actor have been dropped, which closes the mailbox).
    pub fn join_all(self) {
        let mut handles = self.handles.into_inner().unwrap();
        for h in handles.drain(..) {
            let _ = h.join();
        }
    }
}

fn run_actor_loop<M, R>(
    mut actor: Box<dyn ActorBehaviour<M, R>>,
    rx: mpsc::Receiver<Control<M, R>>,
    // ↑ unchanged receive-side: SyncSender / Sender both produce mpsc::Receiver
) where
    M: Send + 'static,
    R: Send + 'static,
{
    actor.dyn_on_start();
    while let Ok(control) = rx.recv() {
        match control {
            Control::User(env) => {
                let reply = actor.dyn_handle(env.message);
                if let Some(tx_reply) = env.reply_to {
                    let _ = tx_reply.send(reply);
                }
            }
            Control::Reload(cmd) => {
                let from = actor.dyn_schema_version();
                let to = cmd.target_version;
                if to < from && !cmd.allow_downgrade {
                    let _ = cmd.reply.send(ReloadOutcome::DowngradeRefused { from, to });
                    continue;
                }
                // Drain anything already pending in the mailbox so the
                // migrator and the new behaviour see a quiescent queue.
                let mut buffered = Vec::new();
                while let Ok(c) = rx.try_recv() {
                    match c {
                        Control::User(env) => buffered.push(env),
                        Control::Reload(extra) => {
                            // Reload-while-reloading is rejected — only one
                            // hot-swap per recv tick to keep ordering
                            // invariants tractable. The extra reload's caller
                            // gets DowngradeRefused with from=to so it can
                            // retry on the next iteration.
                            let _ = extra.reply.send(ReloadOutcome::DowngradeRefused {
                                from: cmd.target_version,
                                to: extra.target_version,
                            });
                        }
                    }
                }
                actor.dyn_on_stop();
                actor = (cmd.migrator)(actor);
                actor.dyn_on_start();
                let replayed = buffered.len();
                for env in buffered {
                    let reply = actor.dyn_handle(env.message);
                    if let Some(tx_reply) = env.reply_to {
                        let _ = tx_reply.send(reply);
                    }
                }
                let _ = cmd.reply.send(ReloadOutcome::Ok { replayed });
            }
        }
    }
    actor.dyn_on_stop();
}
