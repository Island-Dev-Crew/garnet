//! Mini-Spec v0.2 §2.1 — memory unit declarations.
//!
//! ```text
//! memory-decl  := "memory" memory-kind ident ":" store-type
//! memory-kind  := "working" | "episodic" | "semantic" | "procedural"
//! store-type   := ident ("<" type-args ">")?
//! ```

use crate::ast::{MemoryDecl, MemoryKind, Type};
use crate::error::ParseError;
use crate::parser::{describe_kind, Parser};
use crate::token::{Span, TokenKind};

pub fn parse_memory_decl(p: &mut Parser) -> Result<MemoryDecl, ParseError> {
    let mem_tok = p.expect(&TokenKind::KwMemory, "`memory`")?;
    let kind = parse_memory_kind(p)?;
    let (name, _) = p.expect_ident("memory unit name")?;
    p.expect(&TokenKind::Colon, "`:` before store-type")?;
    let store = parse_type(p)?;
    let span = mem_tok.span.join(store.span);
    Ok(MemoryDecl {
        kind,
        name,
        store,
        span,
    })
}

fn parse_memory_kind(p: &mut Parser) -> Result<MemoryKind, ParseError> {
    match p.peek_kind() {
        TokenKind::KwWorking => {
            p.bump();
            Ok(MemoryKind::Working)
        }
        TokenKind::KwEpisodic => {
            p.bump();
            Ok(MemoryKind::Episodic)
        }
        TokenKind::KwSemantic => {
            p.bump();
            Ok(MemoryKind::Semantic)
        }
        TokenKind::KwProcedural => {
            p.bump();
            Ok(MemoryKind::Procedural)
        }
        other => Err(ParseError::unexpected_token(
            "memory kind (`working`, `episodic`, `semantic`, or `procedural`)",
            describe_kind(other),
            p.peek().span,
        )),
    }
}

/// Parse a `store-type`: ident, optionally followed by `<type-args>`.
/// Recursive — type args are themselves store-types. Used for both
/// memory-decl store types and actor protocol-decl/handler-decl types.
pub fn parse_type(p: &mut Parser) -> Result<Type, ParseError> {
    let (name, name_span) = p.expect_ident("type name")?;
    let mut span = name_span;
    let mut args = Vec::new();
    if matches!(p.peek_kind(), TokenKind::Lt) {
        p.bump();
        loop {
            let arg = parse_type(p)?;
            args.push(arg);
            if !p.eat(&TokenKind::Comma) {
                break;
            }
        }
        let close = p.expect(&TokenKind::Gt, "`>` to close type arguments")?;
        span = span.join(close.span);
    }
    Ok(Type { name, args, span })
}

/// Re-export for downstream use; lets actors.rs share span info on the
/// closing token of a generic argument list.
pub fn span_of(s: Span) -> Span {
    s
}
