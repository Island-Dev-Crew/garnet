//! Garnet v0.3 grammar — top-level item dispatch.
//! Delegates to sub-modules for each grammar section.

pub mod actors;
pub mod control_flow;
pub mod expr;
pub mod functions;
pub mod memory;
pub mod modules;
pub mod patterns;
pub mod stmts;
pub mod types;
pub mod user_types;

use crate::ast::*;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::token::TokenKind;

/// Parse a sequence of top-level items from the token stream.
/// Handles @safe at file level and dispatches each item to its grammar module.
pub fn parse_items(p: &mut Parser) -> Result<(bool, Vec<Item>), ParseError> {
    let mut items = Vec::new();
    let mut file_safe = false;

    p.skip_separators();

    // Check for file-level @safe annotation
    if matches!(p.peek_kind(), TokenKind::At) {
        let next = p.peek_nth(1);
        if matches!(&next.kind, TokenKind::Ident(s) if s == "safe") {
            // Check that this is at the very start (before any items)
            if items.is_empty() {
                p.bump(); // @
                p.bump(); // safe
                file_safe = true;
                p.skip_separators();
            }
        }
    }

    while !p.at_end() {
        p.skip_separators();
        if p.at_end() {
            break;
        }

        let item = parse_item(p)?;
        items.push(item);
        p.skip_separators();
    }

    // File-level @safe propagates to every nested module so that downstream
    // tools (the safe-mode checker, codegen) only need to consult the module.
    if file_safe {
        for it in items.iter_mut() {
            if let Item::Module(m) = it {
                m.safe = true;
            }
        }
    }

    Ok((file_safe, items))
}

/// Parse a single top-level item (also used by module body parsing).
pub fn parse_item_inner(p: &mut Parser) -> Result<Item, ParseError> {
    parse_item(p)
}

fn parse_item(p: &mut Parser) -> Result<Item, ParseError> {
    // Collect annotations
    let annotations = functions::parse_annotations(p)?;

    // Check for pub
    let public = p.eat(&TokenKind::KwPub);

    // Check if any annotation was @safe (for inline modules)
    let has_safe_annotation = annotations.iter().any(|a| matches!(a, Annotation::Safe(_)));

    match p.peek_kind() {
        TokenKind::KwUse => {
            let decl = modules::parse_use_decl(p)?;
            Ok(Item::Use(decl))
        }
        TokenKind::KwModule => {
            let decl = modules::parse_module_decl(p, has_safe_annotation, public)?;
            Ok(Item::Module(decl))
        }
        TokenKind::KwMemory => {
            let decl = memory::parse_memory_decl(p)?;
            Ok(Item::Memory(decl))
        }
        TokenKind::KwActor => {
            let decl = actors::parse_actor(p, public)?;
            Ok(Item::Actor(decl))
        }
        TokenKind::KwStruct => {
            let decl = user_types::parse_struct(p, public)?;
            Ok(Item::Struct(decl))
        }
        TokenKind::KwEnum => {
            let decl = user_types::parse_enum_decl(p, public)?;
            Ok(Item::Enum(decl))
        }
        TokenKind::KwTrait => {
            let decl = user_types::parse_trait(p, public)?;
            Ok(Item::Trait(decl))
        }
        TokenKind::KwImpl => {
            let decl = user_types::parse_impl(p)?;
            Ok(Item::Impl(decl))
        }
        TokenKind::KwDef => {
            let decl = functions::parse_managed_fn(p, annotations, public)?;
            Ok(Item::Fn(decl))
        }
        TokenKind::KwFn => {
            let decl = functions::parse_safe_fn(p, annotations, public)?;
            Ok(Item::Fn(decl))
        }
        TokenKind::KwConst => {
            let decl = stmts::parse_const_decl(p, public)?;
            Ok(Item::Const(decl))
        }
        TokenKind::KwLet => {
            let decl = stmts::parse_let_decl(p)?;
            Ok(Item::Let(decl))
        }
        _ => {
            let tok = p.peek();
            Err(ParseError::unexpected_token(
                "a top-level item (def, fn, struct, enum, trait, impl, actor, memory, module, use, let, const)",
                &format!("{:?}", tok.kind),
                tok.span,
            ))
        }
    }
}
