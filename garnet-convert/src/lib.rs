//! # Garnet Migration Assistant (v0.4.x)
//!
//! Lifts a **stylized subset** of Rust / Ruby / Python / Go source into
//! Garnet source via a language-independent Common IR. This crate is
//! deliberately **not** a full transpiler: it is a scaffolding tool that
//! gets you most of the way through a port and explicitly hands the rest
//! to a human via two first-class CIR outputs.
//!
//! ## Honest scope
//!
//! - **Stylized parsers.** Each frontend parses a recognizable, common
//!   slice of its source language (e.g. `def name(args):` for Python,
//!   `func name(params) ReturnType {…}` for Go). It is not a complete
//!   `rustc` / `mri` / `cpython` / `gc` parser and never will be in this
//!   crate. Inputs outside the recognized slice produce `Untranslatable`.
//! - **`MigrateTodo` is a feature, not a failure.** When the frontend
//!   recognizes a construct that needs human attention (Python decorators,
//!   Ruby `method_missing`, Go variadic generics), it emits a
//!   `MigrateTodo` CIR node with a pointer to the relevant Mini-Spec v1.0
//!   section (e.g. §11.7 `@dynamic` for `method_missing`). The emitter
//!   collects these into a `migrate_todo.md` checklist beside the output.
//! - **`Untranslatable` halts mechanical lowering.** Constructs with no
//!   safe mapping (`eval` / `exec` / arbitrary metaclass shenanigans) are
//!   wrapped in an `Untranslatable` node. Strict mode
//!   (`fail_on_untranslatable`) turns these into hard errors; default
//!   mode emits them as commented stubs the human must replace.
//! - **Sandbox-on-by-default.** Every emitted file starts with `@sandbox`
//!   (v4.0 SandboxMode default). The converter cannot emit
//!   `@sandbox(unquarantine)` — that escape hatch requires human audit.
//!
//! ## Pipeline
//!
//! 1. Per-language frontend parses source into that language's AST
//! 2. Frontend lifts AST → [`Cir`] (Common IR) with lineage tags
//! 3. [`idioms`] applies language-specific CIR-to-CIR rewrites
//! 4. [`witness`] verifies every CIR node has a source lineage
//! 5. [`emitter`] produces Garnet source + `lineage.json` + `migrate_todo.md`
//!
//! ## When to use this crate
//!
//! Use it when you want a Garnet-shaped first draft of an existing
//! codebase, ready for a human reviewer to finish. Do **not** use it
//! expecting compilable, behaviorally-equivalent output: that is a
//! research-grade transpiler problem this crate intentionally does not
//! solve.

pub mod cir;
pub mod emitter;
pub mod error;
pub mod frontends;
pub mod idioms;
pub mod lineage;
pub mod metrics;
pub mod witness;

pub use cir::{
    CatchArm, Cir, CirLit, CirTy, FieldDecl, FuncMode, MatchArm, Ownership, Param, VariantDecl,
};
pub use emitter::{emit, EmitOpts, EmittedSource};
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

impl std::str::FromStr for SourceLang {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "rust" | "rs" => Ok(SourceLang::Rust),
            "ruby" | "rb" => Ok(SourceLang::Ruby),
            "python" | "py" => Ok(SourceLang::Python),
            "go" => Ok(SourceLang::Go),
            _ => Err(()),
        }
    }
}
