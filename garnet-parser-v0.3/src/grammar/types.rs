//! Garnet v0.3 type expression parsing.
//! Covers: `Named<T>`, `(A, B) -> C`, `(A, B)`, `&mut T`

use crate::ast::TypeExpr;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::token::TokenKind;

/// Parse a type expression.
pub fn parse_type(p: &mut Parser) -> Result<TypeExpr, ParseError> {
    match p.peek_kind() {
        TokenKind::Amp => parse_ref_type(p),
        TokenKind::LParen => parse_fn_or_tuple_type(p),
        _ => parse_named_type(p),
    }
}

/// Parse optional type params: `<T, U>`. Returns empty vec if no `<`.
pub fn parse_type_params(p: &mut Parser) -> Result<Vec<String>, ParseError> {
    if !p.eat(&TokenKind::Lt) {
        return Ok(Vec::new());
    }
    let mut params = Vec::new();
    loop {
        let (name, _) = p.expect_ident("type parameter")?;
        params.push(name);
        if !p.eat(&TokenKind::Comma) {
            break;
        }
    }
    p.expect(&TokenKind::Gt, "type parameters")?;
    Ok(params)
}

/// Parse optional generic args: `<Int, String>`. Returns empty vec if no `<`.
pub fn parse_type_args(p: &mut Parser) -> Result<Vec<TypeExpr>, ParseError> {
    if !p.eat(&TokenKind::Lt) {
        return Ok(Vec::new());
    }
    let mut args = Vec::new();
    loop {
        args.push(parse_type(p)?);
        if !p.eat(&TokenKind::Comma) {
            break;
        }
    }
    p.expect(&TokenKind::Gt, "type arguments")?;
    Ok(args)
}

/// Named type with optional path and generics: `Int`, `Array<T>`, `Foo::Bar<T>`
fn parse_named_type(p: &mut Parser) -> Result<TypeExpr, ParseError> {
    let (first, start_span) = p.expect_ident("type name")?;
    let mut path = vec![first];

    // Extend path with :: segments
    while p.eat(&TokenKind::ColonCol) {
        let (seg, _) = p.expect_ident("type path segment")?;
        path.push(seg);
    }

    let args = parse_type_args(p)?;
    let span = start_span.join(p.prev_span());

    Ok(TypeExpr::Named { path, args, span })
}

/// `&T` or `&mut T`
fn parse_ref_type(p: &mut Parser) -> Result<TypeExpr, ParseError> {
    let start = p.bump().span; // consume &
    let mutable = p.eat(&TokenKind::KwMut);
    let inner = parse_type(p)?;
    let span = start.join(inner.span());
    Ok(TypeExpr::Ref {
        mutable,
        inner: Box::new(inner),
        span,
    })
}

/// `(A, B) -> C` (fn type) or `(A, B)` (tuple type) or `(A)` (parens, unwrap)
fn parse_fn_or_tuple_type(p: &mut Parser) -> Result<TypeExpr, ParseError> {
    let start = p.bump().span; // consume (
    let mut types = Vec::new();
    if !matches!(p.peek_kind(), TokenKind::RParen) {
        loop {
            types.push(parse_type(p)?);
            if !p.eat(&TokenKind::Comma) {
                break;
            }
        }
    }
    p.expect(&TokenKind::RParen, "type")?;

    // Check for -> return type (fn type)
    if p.eat(&TokenKind::Arrow) {
        let ret = parse_type(p)?;
        let span = start.join(ret.span());
        Ok(TypeExpr::Fn {
            params: types,
            ret: Box::new(ret),
            span,
        })
    } else if types.len() >= 2 {
        let span = start.join(p.prev_span());
        Ok(TypeExpr::Tuple {
            elements: types,
            span,
        })
    } else if types.len() == 1 {
        // SAFETY: this branch is gated on types.len() == 1, so .next() is Some.
        Ok(types
            .into_iter()
            .next()
            .expect("types.len() == 1 verified above"))
    } else {
        // Empty parens () — unit type, represent as empty tuple
        let span = start.join(p.prev_span());
        Ok(TypeExpr::Tuple {
            elements: Vec::new(),
            span,
        })
    }
}
