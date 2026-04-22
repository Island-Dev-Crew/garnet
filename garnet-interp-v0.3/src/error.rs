//! Runtime errors raised by the interpreter.

use crate::value::Value;

/// Errors that terminate evaluation. `Raised` carries a user-visible exception
/// value for `try`/`rescue` to match against.
#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    /// A pure runtime failure expressible as a single message.
    #[error("{0}")]
    Message(String),

    /// A parse error encountered while loading source.
    #[error("parse error: {0}")]
    Parse(String),

    /// A user `raise` statement or propagated exception.
    #[error("exception: {0}")]
    Raised(Value),

    /// Type mismatch during an operation.
    #[error("type error: expected {expected}, got {got}")]
    Type { expected: String, got: String },

    /// Division by zero.
    #[error("division by zero")]
    DivByZero,

    /// Index out of bounds.
    #[error("index out of bounds: {idx}")]
    IndexOOB { idx: i64 },

    /// Pattern match failed exhaustively.
    #[error("no matching pattern for {value}")]
    NoMatch { value: String },

    /// Break / continue / return signals (handled by stmt/eval, not usually
    /// surfaced to user code).
    #[error("control flow: break")]
    Break(Option<Value>),
    #[error("control flow: continue")]
    Continue,
    #[error("control flow: return")]
    Return(Value),
}

impl RuntimeError {
    pub fn msg(s: impl Into<String>) -> Self {
        RuntimeError::Message(s.into())
    }

    pub fn type_err(expected: &str, got: &Value) -> Self {
        RuntimeError::Type {
            expected: expected.to_string(),
            got: got.type_name().to_string(),
        }
    }
}
