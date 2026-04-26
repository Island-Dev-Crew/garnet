//! Type-safe address for sending messages to a running actor.

use crate::runtime::{ActorBehaviour, Control, ReloadCommand, ReloadOutcome};
use std::fmt;
use std::sync::mpsc::{self, Sender, SyncSender};
use std::time::Duration;

/// A failure that can occur during a synchronous `ask` to an actor.
#[derive(Debug)]
pub enum AskError {
    /// The actor's mailbox is closed because the actor has shut down.
    MailboxClosed,
    /// The actor accepted the message but did not reply within the timeout.
    Timeout,
    /// The reply channel was dropped before producing a value.
    ReplyDropped,
    /// The actor's mailbox is full (v3.4 BoundedMail) — `ask`/`ask_timeout`
    /// did not block waiting for capacity.
    MailboxFull,
}

impl fmt::Display for AskError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AskError::MailboxClosed => write!(f, "actor mailbox is closed"),
            AskError::Timeout => write!(f, "actor did not reply within timeout"),
            AskError::ReplyDropped => write!(f, "reply channel was dropped"),
            AskError::MailboxFull => write!(f, "actor mailbox is full"),
        }
    }
}

impl std::error::Error for AskError {}

/// A failure during a non-blocking `try_tell` (v3.4 BoundedMail).
///
/// Closes the unbounded-mailbox DOS class: a sender that wants to handle
/// overload explicitly can use `try_tell` to learn whether a message
/// fit, instead of blocking on a full mailbox.
#[derive(Debug)]
pub enum SendError {
    /// The mailbox is at capacity. The caller's choice: drop, retry, or log.
    Full,
    /// The actor's mailbox is closed because the actor has shut down.
    Closed,
}

impl fmt::Display for SendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SendError::Full => write!(f, "actor mailbox is full"),
            SendError::Closed => write!(f, "actor mailbox is closed"),
        }
    }
}

impl std::error::Error for SendError {}

/// An address that can send messages to a running actor and (optionally)
/// receive replies. Cheap to clone (each clone shares the same mailbox).
///
/// v3.4 BoundedMail: the underlying channel is bounded at the actor's
/// `mailbox_capacity()` (default 1024). `tell` blocks if the mailbox is
/// full; `try_tell` returns `SendError::Full` without blocking.
pub struct ActorAddress<M, R> {
    pub(crate) tx: SyncSender<Control<M, R>>,
}

impl<M, R> Clone for ActorAddress<M, R> {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
        }
    }
}

pub(crate) struct Envelope<M, R> {
    pub message: M,
    pub reply_to: Option<Sender<R>>,
}

impl<M: Send + 'static, R: Send + 'static> ActorAddress<M, R> {
    /// Fire-and-forget: send the message; do not wait for a reply.
    /// Returns `false` if the mailbox is closed.
    ///
    /// v3.4 BoundedMail: blocks if the mailbox is full. Use `try_tell`
    /// for non-blocking semantics that distinguish Full from Closed.
    pub fn tell(&self, message: M) -> bool {
        self.tx
            .send(Control::User(Envelope {
                message,
                reply_to: None,
            }))
            .is_ok()
    }

    /// Non-blocking fire-and-forget (v3.4 BoundedMail).
    ///
    /// Returns `Ok(())` if the message was queued, `Err(SendError::Full)`
    /// if the mailbox is at capacity (the message is NOT queued — the
    /// caller chooses what to do: drop, retry with backoff, or log),
    /// or `Err(SendError::Closed)` if the actor has shut down.
    pub fn try_tell(&self, message: M) -> Result<(), SendError> {
        match self.tx.try_send(Control::User(Envelope {
            message,
            reply_to: None,
        })) {
            Ok(()) => Ok(()),
            Err(mpsc::TrySendError::Full(_)) => Err(SendError::Full),
            Err(mpsc::TrySendError::Disconnected(_)) => Err(SendError::Closed),
        }
    }

    /// Send the message and block until the actor replies, returning a
    /// `Result` so closed mailboxes / dropped reply channels surface as
    /// recoverable errors instead of panics. This is the release-grade
    /// API. Use `ask_timeout` if you need a bounded wait, or accept the
    /// `AskError` and call `.expect(...)` if you genuinely want to abort
    /// on failure.
    pub fn try_ask(&self, message: M) -> Result<R, AskError> {
        let (tx_reply, rx_reply) = mpsc::channel();
        self.tx
            .send(Control::User(Envelope {
                message,
                reply_to: Some(tx_reply),
            }))
            .map_err(|_| AskError::MailboxClosed)?;
        rx_reply.recv().map_err(|_| AskError::ReplyDropped)
    }

    /// Send the message and block until the actor replies. Panics if the
    /// mailbox is closed or the reply channel is dropped before producing
    /// a value.
    ///
    /// Deprecated since 0.4.0: panicking on cross-actor failure is rarely
    /// what production code wants — use [`try_ask`](Self::try_ask) for a
    /// `Result`-returning equivalent, or [`ask_timeout`](Self::ask_timeout)
    /// for a bounded wait. To preserve the old panic-on-failure
    /// semantics, write `addr.try_ask(msg).expect("…")` at the call
    /// site so the failure mode is visible there.
    #[deprecated(
        since = "0.4.0",
        note = "ask() panics on closed mailbox / dropped reply; use try_ask() for Result, ask_timeout() for bounded wait, or try_ask().expect(...) to keep the panic explicit"
    )]
    pub fn ask(&self, message: M) -> R {
        self.try_ask(message)
            .expect("actor ask() failed (mailbox closed or reply dropped)")
    }

    /// Like `ask`, but with a timeout. Returns an `AskError` for any
    /// failure mode (closed, timeout, dropped).
    pub fn ask_timeout(&self, message: M, timeout: Duration) -> Result<R, AskError> {
        let (tx_reply, rx_reply) = mpsc::channel();
        self.tx
            .send(Control::User(Envelope {
                message,
                reply_to: Some(tx_reply),
            }))
            .map_err(|_| AskError::MailboxClosed)?;
        match rx_reply.recv_timeout(timeout) {
            Ok(v) => Ok(v),
            Err(mpsc::RecvTimeoutError::Timeout) => Err(AskError::Timeout),
            Err(mpsc::RecvTimeoutError::Disconnected) => Err(AskError::ReplyDropped),
        }
    }

    /// Hot-reload the actor's behaviour. Implements Paper VI Contribution 6.
    ///
    /// `target_version` is the schema version the new behaviour will report.
    /// If it is *less than* the current behaviour's version and
    /// `allow_downgrade` is false, the runtime refuses without invoking the
    /// migrator. The migrator takes ownership of the old boxed behaviour and
    /// returns the new one (typically constructed from migrated state).
    ///
    /// Blocks until the actor processes the reload command. Returns either
    /// `ReloadOutcome::Ok { replayed }` with the count of buffered messages
    /// processed against the new behaviour, or `ReloadOutcome::DowngradeRefused`.
    pub fn reload<F>(
        &self,
        target_version: u32,
        allow_downgrade: bool,
        migrator: F,
    ) -> Result<ReloadOutcome, AskError>
    where
        F: FnOnce(Box<dyn ActorBehaviour<M, R>>) -> Box<dyn ActorBehaviour<M, R>> + Send + 'static,
    {
        let (tx_reply, rx_reply) = mpsc::channel();
        let cmd = ReloadCommand {
            target_version,
            allow_downgrade,
            migrator: Box::new(migrator),
            reply: tx_reply,
        };
        self.tx
            .send(Control::Reload(cmd))
            .map_err(|_| AskError::MailboxClosed)?;
        rx_reply.recv().map_err(|_| AskError::ReplyDropped)
    }
}
