//! Mini-Spec v0.2 §4.1 — actor declarations.
//!
//! ```text
//! actor-decl    := "actor" ident "{" protocol-decl* handler-decl* "}"
//! protocol-decl := "protocol" ident "(" param-list ")" ("->" type)?
//! handler-decl  := "on" ident "(" param-list ")" block
//! ```
//!
//! `block` is undefined in v0.2; we use `{ stmt* }` per the surrounding
//! actor brace style — see `expr::parse_block`.
//!
//! The spec does not constrain protocol/handler ordering — we accept them
//! in any order, in the spirit of §4.2's "every protocol MUST have a
//! handler." Pairing enforcement is a validator-pass concern, not parsing.

use crate::ast::{ActorDef, HandlerDecl, ProtocolDecl, TypedParam};
use crate::error::ParseError;
use crate::grammar::{expr, memory};
use crate::parser::{describe_kind, Parser};
use crate::token::{Span, TokenKind};

pub fn parse_actor(p: &mut Parser) -> Result<ActorDef, ParseError> {
    let actor_tok = p.expect(&TokenKind::KwActor, "`actor`")?;
    let (name, _) = p.expect_ident("actor name")?;
    p.expect(&TokenKind::LBrace, "`{` to open actor body")?;
    p.skip_separators();

    let mut protocols = Vec::new();
    let mut handlers = Vec::new();
    while !matches!(p.peek_kind(), TokenKind::RBrace | TokenKind::Eof) {
        match p.peek_kind() {
            TokenKind::KwProtocol => protocols.push(parse_protocol_decl(p)?),
            TokenKind::KwOn => handlers.push(parse_handler_decl(p)?),
            other => {
                return Err(ParseError::unexpected_token(
                    "`protocol`, `on`, or `}`",
                    describe_kind(other),
                    p.peek().span,
                ));
            }
        }
        p.skip_separators();
    }

    let close = p.expect(&TokenKind::RBrace, "`}` to close actor body")?;
    Ok(ActorDef {
        name,
        protocols,
        handlers,
        span: actor_tok.span.join(close.span),
    })
}

fn parse_protocol_decl(p: &mut Parser) -> Result<ProtocolDecl, ParseError> {
    let proto_tok = p.expect(&TokenKind::KwProtocol, "`protocol`")?;
    let (name, _) = p.expect_ident("protocol name")?;
    p.expect(&TokenKind::LParen, "`(`")?;
    // Per §4.1, protocol params are typed.
    let params = parse_param_list(p, /*require_types=*/ true)?;
    let close_paren = p.expect(&TokenKind::RParen, "`)`")?;
    let mut span = proto_tok.span.join(close_paren.span);
    let return_ty = if matches!(p.peek_kind(), TokenKind::Arrow) {
        p.bump();
        let ty = memory::parse_type(p)?;
        span = span.join(ty.span);
        Some(ty)
    } else {
        None
    };
    Ok(ProtocolDecl {
        name,
        params,
        return_ty,
        span,
    })
}

fn parse_handler_decl(p: &mut Parser) -> Result<HandlerDecl, ParseError> {
    let on_tok = p.expect(&TokenKind::KwOn, "`on`")?;
    let (name, _) = p.expect_ident("handler name (matching a protocol)")?;
    p.expect(&TokenKind::LParen, "`(`")?;
    // Handler params MAY omit types — the spec doesn't require them, and
    // it's more ergonomic for handler bodies to bind by name only.
    let params = parse_param_list(p, /*require_types=*/ false)?;
    p.expect(&TokenKind::RParen, "`)`")?;
    let body = expr::parse_block(p)?;
    let span = on_tok.span.join(body.span);
    Ok(HandlerDecl {
        name,
        params,
        body,
        span,
    })
}

fn parse_param_list(
    p: &mut Parser,
    require_types: bool,
) -> Result<Vec<TypedParam>, ParseError> {
    let mut out = Vec::new();
    if matches!(p.peek_kind(), TokenKind::RParen) {
        return Ok(out);
    }
    loop {
        let (name, name_span) = p.expect_ident("parameter name")?;
        let mut span: Span = name_span;
        let ty = if matches!(p.peek_kind(), TokenKind::Colon) {
            p.bump();
            let ty = memory::parse_type(p)?;
            span = name_span.join(ty.span);
            Some(ty)
        } else if require_types {
            return Err(ParseError::unexpected_token(
                "`:` (protocol parameters require types)",
                describe_kind(p.peek_kind()),
                p.peek().span,
            ));
        } else {
            None
        };
        out.push(TypedParam { name, ty, span });
        if !p.eat(&TokenKind::Comma) {
            break;
        }
    }
    Ok(out)
}
