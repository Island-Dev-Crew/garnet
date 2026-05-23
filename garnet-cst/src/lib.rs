//! # garnet-cst — trivia-preserving Concrete Syntax Tree for Garnet
//!
//! A rowan-backed CST built **cold** for the v0.7 build-both-then-compare A/B
//! (slice S15). See `AGENTS.md` for the full trait surface, the planned
//! `cst_to_ast` projection, and the stability tier.
//!
//! ## Independence contract
//!
//! This crate is built without reference to the in-parser CST merged in #221
//! (`garnet-parser-v0.3/src/cst.rs`), which is preserved untouched as the
//! S15-Compare baseline. The only surfaces shared with the parser are the
//! trivia-preserving lexer (`garnet_parser::lex_source`) and the AST type
//! (`garnet_parser::ast::Module`, the target of the future `cst_to_ast`).
//!
//! ## Stability
//!
//! `experimental`. The surface may evolve until the S15-Compare checkpoint
//! (Jon) picks the canonical CST. The compiler `@stability(experimental)`
//! annotation is wired once S17 ships it.
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

mod syntax_kind;

pub use syntax_kind::{GarnetLanguage, SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken};

use rowan::{GreenNodeBuilder, Language};

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

/// Parse a Garnet source string into a CST.
///
/// **PR-1 stub.** This wraps the entire source in a single `Root` node holding
/// one trivia leaf (the whole input). It is trivia-preserving by construction
/// (round-trips byte-for-byte) but performs no structural parsing. The real
/// rowan recursive-descent builder lands in PR-2 (`S15: trivia-preserving CST
/// via rowan`). The trait surface above is real and stable; only this impl is
/// intentionally trivial.
#[must_use]
pub fn parse_cst(input: &str) -> Parse<SyntaxNode> {
    let mut builder = GreenNodeBuilder::new();
    builder.start_node(GarnetLanguage::kind_to_raw(SyntaxKind::Root));
    if !input.is_empty() {
        // Entire source as one trivia leaf — a lossless round-trip.
        builder.token(GarnetLanguage::kind_to_raw(SyntaxKind::Whitespace), input);
    }
    builder.finish_node();
    Parse {
        root: SyntaxNode::new_root(builder.finish()),
        errors: Vec::new(),
    }
}
