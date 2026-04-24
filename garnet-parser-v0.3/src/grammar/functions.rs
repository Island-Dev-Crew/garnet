//! Garnet v0.3 function and closure parser (Mini-Spec §5).

use crate::ast::*;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::token::TokenKind;

use super::{expr, stmts, types};

/// Parse annotations before an item: @max_depth(N), @fan_out(K), @require_metadata, @safe, @dynamic
pub fn parse_annotations(p: &mut Parser) -> Result<Vec<Annotation>, ParseError> {
    let mut annotations = Vec::new();
    while matches!(p.peek_kind(), TokenKind::At) {
        let start = p.bump().span; // consume @
        let (name, _) = p.expect_ident("annotation")?;
        let ann = match name.as_str() {
            "max_depth" => {
                p.expect(&TokenKind::LParen, "annotation")?;
                let tok = p.peek().clone();
                let depth = match &tok.kind {
                    TokenKind::Int(v) => { let v = *v; p.bump(); v }
                    _ => return Err(ParseError::unexpected_token("integer", &format!("{:?}", tok.kind), tok.span)),
                };
                p.expect(&TokenKind::RParen, "annotation")?;
                Annotation::MaxDepth(depth, start.join(p.prev_span()))
            }
            "fan_out" => {
                p.expect(&TokenKind::LParen, "annotation")?;
                let tok = p.peek().clone();
                let width = match &tok.kind {
                    TokenKind::Int(v) => { let v = *v; p.bump(); v }
                    _ => return Err(ParseError::unexpected_token("integer", &format!("{:?}", tok.kind), tok.span)),
                };
                p.expect(&TokenKind::RParen, "annotation")?;
                Annotation::FanOut(width, start.join(p.prev_span()))
            }
            "require_metadata" => Annotation::RequireMetadata(start.join(p.prev_span())),
            "safe" => Annotation::Safe(start.join(p.prev_span())),
            "dynamic" => Annotation::Dynamic(start.join(p.prev_span())),
            "caps" => {
                // @caps(fs, net) or @caps() — empty means "no caps".
                p.expect(&TokenKind::LParen, "annotation")?;
                let mut caps = Vec::new();
                if !matches!(p.peek_kind(), TokenKind::RParen) {
                    loop {
                        // Accept either an ident or `*` (wildcard).
                        let tok = p.peek().clone();
                        let cap = match &tok.kind {
                            TokenKind::Ident(s) => {
                                let s = s.clone();
                                p.bump();
                                Capability::from_ident(&s)
                            }
                            TokenKind::Star => {
                                p.bump();
                                Capability::Wildcard
                            }
                            _ => {
                                return Err(ParseError::unexpected_token(
                                    "capability identifier (fs, net, net_internal, time, proc, ffi, *)",
                                    &format!("{:?}", tok.kind),
                                    tok.span,
                                ));
                            }
                        };
                        caps.push(cap);
                        if !p.eat(&TokenKind::Comma) {
                            break;
                        }
                    }
                }
                p.expect(&TokenKind::RParen, "annotation")?;
                Annotation::Caps(caps, start.join(p.prev_span()))
            }
            "mailbox" => {
                p.expect(&TokenKind::LParen, "annotation")?;
                let tok = p.peek().clone();
                let n = match &tok.kind {
                    TokenKind::Int(v) => {
                        let v = *v;
                        p.bump();
                        v
                    }
                    _ => {
                        return Err(ParseError::unexpected_token(
                            "integer",
                            &format!("{:?}", tok.kind),
                            tok.span,
                        ))
                    }
                };
                p.expect(&TokenKind::RParen, "annotation")?;
                Annotation::Mailbox(n, start.join(p.prev_span()))
            }
            "nonsendable" => Annotation::NonSendable(start.join(p.prev_span())),
            _ => return Err(ParseError::unexpected_token(
                "known annotation (max_depth, fan_out, require_metadata, safe, dynamic, caps, mailbox, nonsendable)",
                &name,
                start,
            )),
        };
        annotations.push(ann);
        p.skip_separators();
    }
    Ok(annotations)
}

/// Parse a managed-mode function: `def name[<T>](params) [-> type] { body }`
pub fn parse_managed_fn(
    p: &mut Parser,
    annotations: Vec<Annotation>,
    public: bool,
) -> Result<FnDef, ParseError> {
    let start = p.expect(&TokenKind::KwDef, "function definition")?.span;
    let (name, _) = p.expect_ident("function name")?;
    let type_params = types::parse_type_params(p)?;
    p.expect(&TokenKind::LParen, "function parameters")?;
    let params = parse_param_list(p, false)?;
    p.expect(&TokenKind::RParen, "function parameters")?;
    let return_ty = if p.eat(&TokenKind::Arrow) {
        Some(types::parse_type(p)?)
    } else {
        None
    };
    let body = stmts::parse_block(p)?;
    let span = start.join(body.span);
    Ok(FnDef {
        annotations,
        public,
        mode: FnMode::Managed,
        name,
        type_params,
        params,
        return_ty,
        body,
        span,
    })
}

/// Parse a safe-mode function: `fn name[<T>](typed_params) -> type { body }`
pub fn parse_safe_fn(
    p: &mut Parser,
    annotations: Vec<Annotation>,
    public: bool,
) -> Result<FnDef, ParseError> {
    let start = p.expect(&TokenKind::KwFn, "function definition")?.span;
    let (name, _) = p.expect_ident("function name")?;
    let type_params = types::parse_type_params(p)?;
    p.expect(&TokenKind::LParen, "function parameters")?;
    let params = parse_param_list(p, true)?;
    p.expect(&TokenKind::RParen, "function parameters")?;
    p.expect(&TokenKind::Arrow, "safe function return type")?;
    let return_ty = Some(types::parse_type(p)?);
    let body = stmts::parse_block(p)?;
    let span = start.join(body.span);
    Ok(FnDef {
        annotations,
        public,
        mode: FnMode::Safe,
        name,
        type_params,
        params,
        return_ty,
        body,
        span,
    })
}

/// Parse parameter list. If `require_types` is true, every param must have a type annotation.
pub fn parse_param_list(p: &mut Parser, require_types: bool) -> Result<Vec<Param>, ParseError> {
    let mut params = Vec::new();
    if matches!(p.peek_kind(), TokenKind::RParen) {
        return Ok(params);
    }
    loop {
        let param = parse_param(p, require_types)?;
        params.push(param);
        if !p.eat(&TokenKind::Comma) {
            break;
        }
    }
    Ok(params)
}

fn parse_param(p: &mut Parser, _require_types: bool) -> Result<Param, ParseError> {
    let start_span = p.peek().span;
    // Check for ownership annotation
    let ownership = match p.peek_kind() {
        TokenKind::KwOwn => {
            p.bump();
            Some(Ownership::Own)
        }
        TokenKind::KwBorrow => {
            p.bump();
            Some(Ownership::Borrow)
        }
        TokenKind::KwRef => {
            p.bump();
            Some(Ownership::Ref)
        }
        TokenKind::KwMut => {
            p.bump();
            Some(Ownership::Mut)
        }
        _ => None,
    };
    // Allow `self` as a parameter name (only meaningful in trait/impl methods).
    let (name, name_span) = if matches!(p.peek_kind(), TokenKind::KwSelf_) {
        let tok = p.bump();
        ("self".to_string(), tok.span)
    } else {
        p.expect_ident("parameter name")?
    };
    let ty = if p.eat(&TokenKind::Colon) {
        Some(types::parse_type(p)?)
    } else {
        None
    };
    let span = start_span.join(ty.as_ref().map(|t| t.span()).unwrap_or(name_span));
    Ok(Param {
        ownership,
        name,
        ty,
        span,
    })
}

/// Parse a closure: `|params| [-> type] (block | expr)`
pub fn parse_closure(p: &mut Parser) -> Result<Expr, ParseError> {
    let start = p.expect(&TokenKind::Pipe, "closure")?.span;
    let params = if matches!(p.peek_kind(), TokenKind::Pipe) {
        Vec::new()
    } else {
        parse_param_list(p, false)?
    };
    p.expect(&TokenKind::Pipe, "closure")?;
    let return_ty = if p.eat(&TokenKind::Arrow) {
        Some(Box::new(types::parse_type(p)?))
    } else {
        None
    };
    let body = if matches!(p.peek_kind(), TokenKind::LBrace) {
        let block = stmts::parse_block(p)?;
        let _span = start.join(block.span);
        Box::new(ClosureBody::Block(block))
    } else {
        let e = expr::parse_expr(p)?;
        Box::new(ClosureBody::Expr(e))
    };
    let end_span = match body.as_ref() {
        ClosureBody::Block(b) => b.span,
        ClosureBody::Expr(e) => e.span(),
    };
    let span = start.join(end_span);
    Ok(Expr::Closure {
        params,
        return_ty,
        body,
        span,
    })
}
