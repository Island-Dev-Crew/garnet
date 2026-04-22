//! Unified error type for the converter pipeline.

use std::fmt;

#[derive(Debug, Clone)]
pub enum ConvertError {
    /// Source-language parser failed. The inner string is the frontend's
    /// best-effort diagnostic.
    ParseError { source_lang: String, message: String },
    /// Witness verification found a CIR node with no lineage — potential
    /// hallucination or frontend bug.
    MissingLineage { node_kind: String, at: usize },
    /// Emitter can't serialize a CIR construct into valid Garnet.
    EmitFailure { reason: String },
    /// Configuration error (bad CLI flag combination, etc.).
    Config(String),
    /// Strict mode fired — untranslatable construct reached emit.
    UntranslatableInStrictMode { reason: String, at: String },
    /// Strict mode fired — MigrateTodo reached emit.
    TodoInStrictMode { note: String, at: String },
}

impl fmt::Display for ConvertError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConvertError::ParseError { source_lang, message } => {
                write!(f, "parse error ({source_lang}): {message}")
            }
            ConvertError::MissingLineage { node_kind, at } => {
                write!(f, "witness failure: CIR node {node_kind} at index {at} has no lineage")
            }
            ConvertError::EmitFailure { reason } => write!(f, "emit failure: {reason}"),
            ConvertError::Config(msg) => write!(f, "config error: {msg}"),
            ConvertError::UntranslatableInStrictMode { reason, at } => {
                write!(f, "strict mode: untranslatable construct at {at}: {reason}")
            }
            ConvertError::TodoInStrictMode { note, at } => {
                write!(f, "strict mode: MigrateTodo at {at}: {note}")
            }
        }
    }
}

impl std::error::Error for ConvertError {}
