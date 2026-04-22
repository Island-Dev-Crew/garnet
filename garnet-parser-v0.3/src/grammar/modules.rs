//! Garnet v0.3 module and import parser (Mini-Spec §3).

use crate::ast::*;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::token::TokenKind;

/// Parse: `[pub] module Name { items }`
pub fn parse_module_decl(p: &mut Parser, safe: bool, public: bool) -> Result<ModuleDecl, ParseError> {
    let start = p.expect(&TokenKind::KwModule, "module declaration")?.span;
    let (name, _) = p.expect_ident("module name")?;
    p.expect(&TokenKind::LBrace, "module body")?;
    p.skip_separators();

    let mut items = Vec::new();
    while !matches!(p.peek_kind(), TokenKind::RBrace | TokenKind::Eof) {
        let item = super::parse_item_inner(p)?;
        items.push(item);
        p.skip_separators();
    }

    p.expect(&TokenKind::RBrace, "module body")?;
    let span = start.join(p.prev_span());
    Ok(ModuleDecl { safe, public, name, items, span })
}

/// Parse: `use Path[::{ names } | ::*]`
pub fn parse_use_decl(p: &mut Parser) -> Result<UseDecl, ParseError> {
    let start = p.expect(&TokenKind::KwUse, "use declaration")?.span;
    let (first, _) = p.expect_ident("module path")?;
    let mut path = vec![first];

    while p.eat(&TokenKind::ColonCol) {
        // Check for glob or named imports
        if p.eat(&TokenKind::Star) {
            let span = start.join(p.prev_span());
            return Ok(UseDecl { path, imports: UseImports::Glob, span });
        }
        if p.eat(&TokenKind::LBrace) {
            let mut names = Vec::new();
            loop {
                let (name, _) = p.expect_ident("import name")?;
                names.push(name);
                if !p.eat(&TokenKind::Comma) {
                    break;
                }
            }
            p.expect(&TokenKind::RBrace, "import list")?;
            let span = start.join(p.prev_span());
            return Ok(UseDecl { path, imports: UseImports::Named(names), span });
        }
        let (seg, _) = p.expect_ident("module path segment")?;
        path.push(seg);
    }

    let span = start.join(p.prev_span());
    Ok(UseDecl { path, imports: UseImports::Module, span })
}

