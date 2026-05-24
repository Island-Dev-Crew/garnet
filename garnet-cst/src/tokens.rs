//! Token/span helpers for rowan CST consumers.
//!
//! The legacy in-parser CST (#221) exposed `CstToken { kind, span }`, which is
//! convenient for LSP rename and semantic-token logic. Rowan stores token text
//! and text ranges instead. This module preserves that useful ergonomic surface
//! without making the parser CST canonical.

use crate::syntax_kind::{SyntaxNode, SyntaxToken};
use garnet_parser::lexer::Lexer;
use garnet_parser::token::{Span, TokenKind};

/// A rowan token recovered as Garnet lexer metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenInfo {
    /// The lexer token kind, including payloads such as identifier names.
    pub kind: TokenKind,
    /// The token's byte span in the original source.
    pub span: Span,
    /// The token's exact source text.
    pub text: String,
}

/// Convert a rowan token text range into the parser's byte-span type.
#[must_use]
pub fn token_span(token: &SyntaxToken) -> Span {
    let range = token.text_range();
    let start: u32 = range.start().into();
    let end: u32 = range.end().into();
    Span::new(start as usize, end.saturating_sub(start) as usize)
}

/// Recover the parser lexer's `TokenKind` for a rowan token.
///
/// Rowan keeps token text and a lightweight `SyntaxKind`; LSP consumers often
/// need lexer payloads, especially identifier names and literal values. Re-lex
/// the isolated token text with the same lexer and take the first emitted token.
#[must_use]
pub fn token_kind(token: &SyntaxToken) -> Option<TokenKind> {
    let text = token.text();
    let mut lexer = Lexer::new(text);
    lexer.lex().ok().and_then(|tokens| {
        tokens
            .into_iter()
            .find(|tok| tok.span.start == 0 && tok.span.len == text.len())
            .map(|tok| tok.kind)
    })
}

/// Return every token under `node` in source order, including trivia.
///
/// The rowan tree intentionally omits the parser's zero-width EOF sentinel.
#[must_use]
pub fn all_tokens(node: &SyntaxNode) -> Vec<SyntaxToken> {
    node.descendants_with_tokens()
        .filter_map(|element| element.into_token())
        .collect()
}

/// Return every token under `node` as lexer metadata.
#[must_use]
pub fn token_infos(node: &SyntaxNode) -> Vec<TokenInfo> {
    all_tokens(node)
        .into_iter()
        .filter_map(|token| {
            token_kind(&token).map(|kind| TokenInfo {
                kind,
                span: token_span(&token),
                text: token.text().to_string(),
            })
        })
        .collect()
}

/// Return byte spans for identifier tokens whose lexer payload equals `name`.
#[must_use]
pub fn identifier_spans(node: &SyntaxNode, name: &str) -> Vec<Span> {
    token_infos(node)
        .into_iter()
        .filter_map(|token| match token.kind {
            TokenKind::Ident(ident) if ident == name => Some(token.span),
            _ => None,
        })
        .collect()
}
