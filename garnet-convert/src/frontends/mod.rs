//! Per-source-language frontends. Each module parses its language's
//! source into the Common IR (CIR) with full lineage tagging.
//!
//! The v4.1 initial release ships **stylized parsers** that recognize
//! the most common source patterns without a full grammar. This is a
//! principled choice per the prior-art findings (§5A case 4): a
//! compiler != a converter. The converter's job is clean translation
//! of the 80% of patterns documented in Phase 2F + 3G — not full
//! language coverage.
//!
//! Production wraps tree-sitter-<lang> for each; that integration is
//! a v4.1.x stage with a well-defined plug-in boundary (each frontend
//! exports `parse_and_lift(source, filename) -> Result<Cir, ConvertError>`).

pub mod go;
pub mod python;
pub mod ruby;
pub mod rust;

pub use go::parse_and_lift as go_parse;
pub use python::parse_and_lift as python_parse;
pub use ruby::parse_and_lift as ruby_parse;
pub use rust::parse_and_lift as rust_parse;
