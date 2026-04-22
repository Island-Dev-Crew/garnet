//! # Garnet v4.1 Code Converter
//!
//! Converts Rust / Ruby / Python / Go source into Garnet source via
//! a language-independent Common IR.
//!
//! ## Pipeline
//!
//! 1. Per-language frontend parses source into that language's AST
//! 2. Frontend lifts AST → [`Cir`] (Common IR) with lineage tags
//! 3. [`idioms`] applies language-specific CIR-to-CIR rewrites
//! 4. [`witness`] verifies every CIR node has a source lineage
//! 5. [`emitter`] produces Garnet source + lineage.json + migrate_todo.md
//!
//! Every emitted file starts with `@sandbox` (v4.0 SandboxMode default).
//! The converter cannot emit `@sandbox(unquarantine)` — that requires
//! human audit.

pub mod cir;
pub mod emitter;
pub mod error;
pub mod frontends;
pub mod idioms;
pub mod lineage;
pub mod metrics;
pub mod witness;

pub use cir::{
    Cir, CirLit, CirTy, CatchArm, FieldDecl, FuncMode, MatchArm, Ownership, Param, VariantDecl,
};
pub use emitter::{emit, EmittedSource, EmitOpts};
pub use error::ConvertError;
pub use lineage::{Lineage, LineageMap};
pub use metrics::ConvertMetrics;

/// Convenience one-shot: parse + lift + idiom + witness + emit for
/// any supported source language. Panics on unsupported language.
pub fn convert(
    source: &str,
    source_lang: SourceLang,
    source_file: &str,
    opts: EmitOpts,
) -> Result<(EmittedSource, ConvertMetrics), ConvertError> {
    let cir = match source_lang {
        SourceLang::Rust => frontends::rust::parse_and_lift(source, source_file)?,
        SourceLang::Ruby => frontends::ruby::parse_and_lift(source, source_file)?,
        SourceLang::Python => frontends::python::parse_and_lift(source, source_file)?,
        SourceLang::Go => frontends::go::parse_and_lift(source, source_file)?,
    };
    let cir = idioms::lower_all(cir);
    witness::verify(&cir)?;
    let (emitted, metrics) = emitter::emit(cir, opts)?;
    Ok((emitted, metrics))
}

/// Supported source languages per Phase 5 §3 of the converter architecture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceLang {
    Rust,
    Ruby,
    Python,
    Go,
}

impl SourceLang {
    pub fn from_str(s: &str) -> Option<SourceLang> {
        match s.to_lowercase().as_str() {
            "rust" | "rs" => Some(SourceLang::Rust),
            "ruby" | "rb" => Some(SourceLang::Ruby),
            "python" | "py" => Some(SourceLang::Python),
            "go" => Some(SourceLang::Go),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            SourceLang::Rust => "rust",
            SourceLang::Ruby => "ruby",
            SourceLang::Python => "python",
            SourceLang::Go => "go",
        }
    }

    pub fn from_extension(ext: &str) -> Option<SourceLang> {
        match ext.to_lowercase().as_str() {
            "rs" => Some(SourceLang::Rust),
            "rb" => Some(SourceLang::Ruby),
            "py" => Some(SourceLang::Python),
            "go" => Some(SourceLang::Go),
            _ => None,
        }
    }
}
