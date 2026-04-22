//! Unified error type for stdlib primitives.

use std::fmt;

/// Every stdlib primitive returns `Result<T, StdError>`. The interpreter
/// bridge maps this into the managed-mode exception hierarchy or the
/// safe-mode `Result<T, E>` depending on the caller's mode (per
/// Mini-Spec v1.0 §7.4 boundary bridging).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StdError {
    /// File not found / not a directory / permission denied / etc.
    Io(String),
    /// Out of the accepted input domain (e.g., invalid UTF-8 in a
    /// `String` primitive).
    InvalidInput(String),
    /// A capability required by this primitive was not declared by the
    /// calling function. Surfaces E0903 at the source layer.
    CapsMissing { prim: String, required: String },
    /// Network operation denied by NetDefaults (RFC1918 target,
    /// DNS-rebinding mismatch, UDP amplification cap exceeded, etc.).
    NetDenied(String),
    /// Integer overflow / arithmetic error.
    Arithmetic(String),
    /// A timeout was reached on a bounded operation.
    Timeout,
}

impl fmt::Display for StdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StdError::Io(msg) => write!(f, "io error: {msg}"),
            StdError::InvalidInput(msg) => write!(f, "invalid input: {msg}"),
            StdError::CapsMissing { prim, required } => {
                write!(f, "primitive `{prim}` requires capability `{required}`")
            }
            StdError::NetDenied(msg) => write!(f, "network denied: {msg}"),
            StdError::Arithmetic(msg) => write!(f, "arithmetic error: {msg}"),
            StdError::Timeout => write!(f, "operation timed out"),
        }
    }
}

impl std::error::Error for StdError {}

impl From<std::io::Error> for StdError {
    fn from(e: std::io::Error) -> Self {
        StdError::Io(e.to_string())
    }
}
