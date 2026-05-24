//! # garnet-cst — trivia-preserving Concrete Syntax Tree for Garnet
//!
//! A rowan-backed CST built **cold** for the v0.7 build-both-then-compare A/B
//! (slice S15). S15-Compare chose this crate as Garnet's canonical CST for
//! v0.7/S16. See `AGENTS.md` for the full trait surface, `cst_to_ast`
//! projection, token/span helpers, and stability tier.
//!
//! ## Independence contract
//!
//! This crate was built without reference to the in-parser CST merged in #221
//! (`garnet-parser-v0.3/src/cst.rs`), which is preserved as a temporary legacy
//! migration oracle after S15-Compare. The only surfaces shared with the parser
//! are the
//! trivia-preserving lexer (`garnet_parser::lex_source`) and the AST type
//! (`garnet_parser::ast::Module`, the target of `cst_to_ast`).
//!
//! ## Stability
//!
//! Canonical for v0.7 after S15-Compare. The Rust API remains additive and
//! experimental until S16 hardens the LSP-facing surface and S17 wires compiler
//! `@stability(experimental)` annotations.
//!
//! ## Round-trip
//!
//! For any input the CST emits as tokens, `cst_to_source` is byte-identical to
//! the input — rowan concatenates every token (whitespace and comments
//! included) in source order.
//!
//! ```
//! let src = "def greet(name) { name }\n";
//! let parse = garnet_cst::parse_cst(src);
//! assert_eq!(parse.to_source(), src);
//! ```

mod builder;
mod convert;
mod nodes;
mod syntax_kind;
mod tokens;

pub use convert::cst_to_ast;
pub use nodes::{CstNodeExt, EnumDef, FnDef, Name, Param, ParamList, Root, StructDef};
pub use syntax_kind::{GarnetLanguage, SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken};
pub use tokens::{all_tokens, identifier_spans, token_infos, token_kind, token_span, TokenInfo};

/// A syntax error recorded during CST construction.
///
/// Best-effort: the PR-1 stub records none. PR-2's recursive-descent builder
/// records recoverable errors here while still producing a (possibly partial)
/// tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxError {
    /// Human-readable description of the problem.
    pub message: String,
    /// Byte offset into the source where the error was detected.
    pub offset: usize,
}

/// The result of a CST parse: a typed root plus any recorded errors.
#[derive(Debug, Clone)]
pub struct Parse<T> {
    /// The parsed root.
    pub root: T,
    /// Errors recorded during construction (empty on a clean parse).
    pub errors: Vec<SyntaxError>,
}

impl Parse<SyntaxNode> {
    /// The CST root node.
    #[must_use]
    pub fn syntax(&self) -> &SyntaxNode {
        &self.root
    }

    /// Reconstruct the source text from the CST (trivia-preserving).
    #[must_use]
    pub fn to_source(&self) -> String {
        cst_to_source(&self.root)
    }

    /// True if the parse recorded no errors.
    #[must_use]
    pub fn ok(&self) -> bool {
        self.errors.is_empty()
    }
}

/// The CST node trait — every typed wrapper around a [`SyntaxNode`] implements
/// it. This is the load-bearing seam S16 (LSP precision) builds against.
pub trait CstNode {
    /// The underlying untyped syntax node.
    fn syntax(&self) -> &SyntaxNode;

    /// The kind of this node.
    fn kind(&self) -> SyntaxKind {
        self.syntax().kind()
    }
}

/// Reconstruct source text from any CST node (trivia-preserving).
///
/// rowan concatenates every contained token, including whitespace and comments,
/// in source order — so for a full parse this is byte-identical to the input.
#[must_use]
pub fn cst_to_source(node: &SyntaxNode) -> String {
    node.text().to_string()
}

/// Parse a Garnet source string into a CST via the rowan recursive-descent
/// builder (cold from Mini-Spec v1.0 §2–§11).
///
/// Round-trip is guaranteed for any input that lexes: every token's source
/// slice is emitted in order, so `cst_to_source(parse_cst(s).syntax()) == s`.
/// Structural nesting follows the grammar; recovery from malformed input is
/// best-effort (the tree still round-trips). For inputs that fail to *lex*,
/// the whole source is preserved under an `Error` leaf and the lex error is
/// recorded in `errors`.
#[must_use]
pub fn parse_cst(input: &str) -> Parse<SyntaxNode> {
    builder::parse(input)
}
