//! Garnet v0.3 actor declaration parser (Mini-Spec §9).

use crate::ast::*;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::token::TokenKind;

use super::{functions, memory, stmts, types};

/// Parse: `[pub] actor Name { items }`
pub fn parse_actor(p: &mut Parser, public: bool) -> Result<ActorDef, ParseError> {
    let start = p.expect(&TokenKind::KwActor, "actor declaration")?.span;
    let (name, _) = p.expect_ident("actor name")?;
    p.expect(&TokenKind::LBrace, "actor body")?;
    p.skip_separators();

    let mut items = Vec::new();
    while !matches!(p.peek_kind(), TokenKind::RBrace | TokenKind::Eof) {
        let item = parse_actor_item(p)?;
        items.push(item);
        p.skip_separators();
    }

    p.expect(&TokenKind::RBrace, "actor body")?;
    let span = start.join(p.prev_span());
    Ok(ActorDef { public, name, items, span })
}

fn parse_actor_item(p: &mut Parser) -> Result<ActorItem, ParseError> {
    // Handler annotations (@max_depth, @fan_out, etc.) precede the `on` keyword.
    let _annotations = functions::parse_annotations(p)?;
    p.skip_separators();
    match p.peek_kind() {
        TokenKind::KwProtocol => {
            let decl = parse_protocol_decl(p)?;
            Ok(ActorItem::Protocol(decl))
        }
        TokenKind::KwOn => {
            let decl = parse_handler_decl(p)?;
            Ok(ActorItem::Handler(decl))
        }
        TokenKind::KwMemory => {
            let decl = memory::parse_memory_decl(p)?;
            Ok(ActorItem::Memory(decl))
        }
        TokenKind::KwLet => {
            let decl = stmts::parse_let_decl(p)?;
            Ok(ActorItem::Let(decl))
        }
        _ => {
            let tok = p.peek();
            Err(ParseError::unexpected_token(
                "actor item (protocol, on, memory, let)",
                &format!("{:?}", tok.kind),
                tok.span,
            ))
        }
    }
}

fn parse_protocol_decl(p: &mut Parser) -> Result<ProtocolDecl, ParseError> {
    let start = p.expect(&TokenKind::KwProtocol, "protocol declaration")?.span;
    let (name, _) = p.expect_ident("protocol name")?;
    p.expect(&TokenKind::LParen, "protocol parameters")?;
    let params = functions::parse_param_list(p, true)?;
    p.expect(&TokenKind::RParen, "protocol parameters")?;
    let return_ty = if p.eat(&TokenKind::Arrow) {
        Some(types::parse_type(p)?)
    } else {
        None
    };
    let span = start.join(p.prev_span());
    Ok(ProtocolDecl { name, params, return_ty, span })
}

fn parse_handler_decl(p: &mut Parser) -> Result<HandlerDecl, ParseError> {
    let start = p.expect(&TokenKind::KwOn, "handler declaration")?.span;
    let (name, _) = p.expect_ident("handler name")?;
    p.expect(&TokenKind::LParen, "handler parameters")?;
    let params = functions::parse_param_list(p, false)?;
    p.expect(&TokenKind::RParen, "handler parameters")?;
    let body = stmts::parse_block(p)?;
    let span = start.join(body.span);
    Ok(HandlerDecl { name, params, body, span })
}
