//! Reconciliation test for #221's parser CST and the canonical rowan CST.
//!
//! The parser CST's strongest ergonomic surface is token payload + source span
//! access. The rowan CST remains canonical, but it should preserve that
//! consumer-facing token view exactly.

use garnet_cst::{identifier_spans, parse_cst, token_infos, TokenInfo};
use garnet_parser::cst::{CstElement, CstNode};
use garnet_parser::parse_source_cst;
use garnet_parser::token::{Span, TokenKind};
use std::fs;
use std::path::{Path, PathBuf};

fn garnet_files(dir: &Path) -> Vec<PathBuf> {
    let mut v = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for e in entries.flatten() {
            let p = e.path();
            if p.extension().and_then(|x| x.to_str()) == Some("garnet") {
                v.push(p);
            }
        }
    }
    v.sort();
    v
}

fn corpus() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();
    let mut files = garnet_files(&root.join("examples"));
    files.extend(garnet_files(&root.join("garnet-parser-v0.3/examples")));
    files
}

fn flatten_parser_tokens(node: &CstNode, out: &mut Vec<(TokenKind, Span)>) {
    for child in &node.children {
        match child {
            CstElement::Node(node) => flatten_parser_tokens(node, out),
            CstElement::Token(token) => {
                if !matches!(token.kind, TokenKind::Eof) {
                    out.push((token.kind.clone(), token.span));
                }
            }
        }
    }
}

#[test]
fn rowan_token_infos_match_parser_cst_tokens_on_corpus() {
    let files = corpus();
    assert!(!files.is_empty(), "example corpus should be non-empty");

    for file in files {
        let src = fs::read_to_string(&file).expect("read example");
        let Ok((_, parser_cst)) = parse_source_cst(&src) else {
            continue;
        };

        let mut parser_tokens = Vec::new();
        flatten_parser_tokens(&parser_cst, &mut parser_tokens);

        let rowan_tokens: Vec<(TokenKind, Span)> = token_infos(parse_cst(&src).syntax())
            .into_iter()
            .map(|TokenInfo { kind, span, .. }| (kind, span))
            .collect();

        assert_eq!(
            rowan_tokens,
            parser_tokens,
            "rowan token view diverged from parser CST for {}",
            file.display()
        );
    }
}

#[test]
fn identifier_spans_preserve_legacy_cst_rename_surface() {
    let src = "def greet(name) { greet(name) }\n";
    let parse = parse_cst(src);
    let spans = identifier_spans(parse.syntax(), "greet");
    let texts: Vec<&str> = spans
        .iter()
        .map(|span| &src[span.start..span.end()])
        .collect();

    assert_eq!(texts, vec!["greet", "greet"]);
}
