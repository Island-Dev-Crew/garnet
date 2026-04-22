//! Garnet v0.3 memory unit declaration parser (Mini-Spec §4).

use crate::ast::{MemoryDecl, MemoryKind};
use crate::error::ParseError;
use crate::parser::Parser;
use crate::token::TokenKind;

use super::types;

/// Parse: `memory <kind> <name> : <store-type>`
pub fn parse_memory_decl(p: &mut Parser) -> Result<MemoryDecl, ParseError> {
    let start = p.bump().span; // consume 'memory'
    let kind = parse_memory_kind(p)?;
    let (name, _) = p.expect_ident("memory declaration")?;
    p.expect(&TokenKind::Colon, "memory declaration")?;
    let store = types::parse_type(p)?;
    let span = start.join(store.span());
    Ok(MemoryDecl { kind, name, store, span })
}

fn parse_memory_kind(p: &mut Parser) -> Result<MemoryKind, ParseError> {
    let tok = p.peek().clone();
    match &tok.kind {
        TokenKind::KwWorking => { p.bump(); Ok(MemoryKind::Working) }
        TokenKind::KwEpisodic => { p.bump(); Ok(MemoryKind::Episodic) }
        TokenKind::KwSemantic => { p.bump(); Ok(MemoryKind::Semantic) }
        TokenKind::KwProcedural => { p.bump(); Ok(MemoryKind::Procedural) }
        _ => Err(ParseError::unexpected_token(
            "memory kind (working, episodic, semantic, procedural)",
            &format!("{:?}", tok.kind),
            tok.span,
        )),
    }
}
