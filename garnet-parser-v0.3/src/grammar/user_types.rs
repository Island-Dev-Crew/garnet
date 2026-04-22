//! Garnet v0.3 user-defined type parser (Mini-Spec §11.3).
//! Covers: struct, enum, trait, impl.

use crate::ast::*;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::token::TokenKind;

use super::{expr, functions, stmts, types};

/// Parse: `[pub] struct Name[<T>] { fields }`
pub fn parse_struct(p: &mut Parser, public: bool) -> Result<StructDef, ParseError> {
    let start = p.expect(&TokenKind::KwStruct, "struct definition")?.span;
    let (name, _) = p.expect_ident("struct name")?;
    let type_params = types::parse_type_params(p)?;
    p.expect(&TokenKind::LBrace, "struct body")?;
    p.skip_separators();

    let mut fields = Vec::new();
    while !matches!(p.peek_kind(), TokenKind::RBrace | TokenKind::Eof) {
        fields.push(parse_field_def(p)?);
        p.eat(&TokenKind::Comma);
        p.skip_separators();
    }

    p.expect(&TokenKind::RBrace, "struct body")?;
    let span = start.join(p.prev_span());
    Ok(StructDef { public, name, type_params, fields, span })
}

fn parse_field_def(p: &mut Parser) -> Result<FieldDef, ParseError> {
    let start_span = p.peek().span;
    let field_public = p.eat(&TokenKind::KwPub);
    let (name, _) = p.expect_ident("field name")?;
    p.expect(&TokenKind::Colon, "field type")?;
    let ty = types::parse_type(p)?;
    let default = if p.eat(&TokenKind::Eq) {
        Some(expr::parse_expr(p)?)
    } else {
        None
    };
    let span = start_span.join(p.prev_span());
    Ok(FieldDef {
        public: field_public,
        name,
        ty,
        default,
        span,
    })
}

/// Parse: `[pub] enum Name[<T>] { variants }`
pub fn parse_enum_decl(p: &mut Parser, public: bool) -> Result<EnumDef, ParseError> {
    let start = p.expect(&TokenKind::KwEnum, "enum definition")?.span;
    let (name, _) = p.expect_ident("enum name")?;
    let type_params = types::parse_type_params(p)?;
    p.expect(&TokenKind::LBrace, "enum body")?;
    p.skip_separators();

    let mut variants = Vec::new();
    while !matches!(p.peek_kind(), TokenKind::RBrace | TokenKind::Eof) {
        variants.push(parse_variant(p)?);
        p.eat(&TokenKind::Comma);
        p.skip_separators();
    }

    p.expect(&TokenKind::RBrace, "enum body")?;
    let span = start.join(p.prev_span());
    Ok(EnumDef { public, name, type_params, variants, span })
}

fn parse_variant(p: &mut Parser) -> Result<Variant, ParseError> {
    let (name, start_span) = p.expect_ident("variant name")?;
    let fields = if p.eat(&TokenKind::LParen) {
        let mut types_list = Vec::new();
        if !matches!(p.peek_kind(), TokenKind::RParen) {
            loop {
                types_list.push(types::parse_type(p)?);
                if !p.eat(&TokenKind::Comma) {
                    break;
                }
            }
        }
        p.expect(&TokenKind::RParen, "variant fields")?;
        types_list
    } else {
        Vec::new()
    };
    let span = start_span.join(p.prev_span());
    Ok(Variant { name, fields, span })
}

/// Parse: `[pub] trait Name[<T>] { items }`
pub fn parse_trait(p: &mut Parser, public: bool) -> Result<TraitDef, ParseError> {
    let start = p.expect(&TokenKind::KwTrait, "trait definition")?.span;
    let (name, _) = p.expect_ident("trait name")?;
    let type_params = types::parse_type_params(p)?;
    p.expect(&TokenKind::LBrace, "trait body")?;
    p.skip_separators();

    let mut items = Vec::new();
    while !matches!(p.peek_kind(), TokenKind::RBrace | TokenKind::Eof) {
        items.push(parse_trait_item(p)?);
        p.skip_separators();
    }

    p.expect(&TokenKind::RBrace, "trait body")?;
    let span = start.join(p.prev_span());
    Ok(TraitDef { public, name, type_params, items, span })
}

fn parse_trait_item(p: &mut Parser) -> Result<TraitItem, ParseError> {
    match p.peek_kind() {
        TokenKind::KwFn => {
            let start = p.bump().span;
            let (name, _) = p.expect_ident("trait function")?;
            p.expect(&TokenKind::LParen, "trait function")?;
            let params = functions::parse_param_list(p, true)?;
            p.expect(&TokenKind::RParen, "trait function")?;
            p.expect(&TokenKind::Arrow, "trait function return type")?;
            let return_ty = Some(types::parse_type(p)?);
            let span = start.join(p.prev_span());
            Ok(TraitItem::FnSig(FnSig { mode: FnMode::Safe, name, params, return_ty, span }))
        }
        TokenKind::KwDef => {
            let start = p.bump().span;
            let (name, _) = p.expect_ident("trait function")?;
            p.expect(&TokenKind::LParen, "trait function")?;
            let params = functions::parse_param_list(p, false)?;
            p.expect(&TokenKind::RParen, "trait function")?;
            let return_ty = if p.eat(&TokenKind::Arrow) {
                Some(types::parse_type(p)?)
            } else {
                None
            };
            let span = start.join(p.prev_span());
            Ok(TraitItem::FnSig(FnSig { mode: FnMode::Managed, name, params, return_ty, span }))
        }
        TokenKind::KwConst => {
            let decl = stmts::parse_const_decl(p, false)?;
            Ok(TraitItem::Const(decl))
        }
        _ => {
            let tok = p.peek();
            Err(ParseError::unexpected_token(
                "trait item (fn, def, const)",
                &format!("{:?}", tok.kind),
                tok.span,
            ))
        }
    }
}

/// Parse: `impl [<T>] Type [for Trait] { methods }`
pub fn parse_impl(p: &mut Parser) -> Result<ImplBlock, ParseError> {
    let start = p.expect(&TokenKind::KwImpl, "impl block")?.span;
    let type_params = types::parse_type_params(p)?;
    let target = types::parse_type(p)?;
    let trait_ty = if p.eat(&TokenKind::KwFor) {
        Some(types::parse_type(p)?)
    } else {
        None
    };
    p.expect(&TokenKind::LBrace, "impl body")?;
    p.skip_separators();

    let mut methods = Vec::new();
    while !matches!(p.peek_kind(), TokenKind::RBrace | TokenKind::Eof) {
        let annotations = functions::parse_annotations(p)?;
        let public = p.eat(&TokenKind::KwPub);
        let method = match p.peek_kind() {
            TokenKind::KwDef => functions::parse_managed_fn(p, annotations, public)?,
            TokenKind::KwFn => functions::parse_safe_fn(p, annotations, public)?,
            _ => {
                let tok = p.peek();
                return Err(ParseError::unexpected_token(
                    "method (def or fn)",
                    &format!("{:?}", tok.kind),
                    tok.span,
                ));
            }
        };
        methods.push(method);
        p.skip_separators();
    }

    p.expect(&TokenKind::RBrace, "impl body")?;
    let span = start.join(p.prev_span());
    Ok(ImplBlock { type_params, target, trait_ty, methods, span })
}
