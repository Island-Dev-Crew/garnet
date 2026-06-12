//! Token-view parity: the rowan CST's consumer-facing token surface must
//! match the shared lexer exactly.
//!
//! RB-4a retired #221's parser CST (the "temporary legacy migration
//! oracle") after its recorded deletion precondition — rowan-backed LSP
//! coverage green — was met. The reconciliation invariant it anchored
//! survives here in a STRONGER form: the legacy CST's token stream was the
//! lexer's stream re-threaded through AST spans, so comparing
//! `token_infos` directly against `garnet_parser::lex_source` checks the
//! same consumer-facing surface (token payload + source span) without the
//! middleman. Unlike the old differential, this one does not REQUIRE parse
//! success — the corpus is fully parseable today (verified: the old test
//! skipped zero files), so the no-parse-needed path is exercised by the
//! explicit lexable-but-unparseable cases below, not by corpus accident.

use garnet_cst::{identifier_spans, parse_cst, token_infos, TokenInfo};
use garnet_parser::lex_source;
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

#[test]
fn rowan_token_infos_match_the_lexer_on_corpus() {
    let files = corpus();
    assert!(!files.is_empty(), "example corpus should be non-empty");

    let mut compared = 0usize;
    let total = files.len();
    for file in files {
        let src = fs::read_to_string(&file).expect("read example");
        let Ok(lexed) = lex_source(&src) else {
            continue;
        };
        compared += 1;
        let lexer_tokens: Vec<(TokenKind, Span)> = lexed
            .into_iter()
            .filter(|t| !matches!(t.kind, TokenKind::Eof))
            .map(|t| (t.kind, t.span))
            .collect();

        let rowan_tokens: Vec<(TokenKind, Span)> = token_infos(parse_cst(&src).syntax())
            .into_iter()
            .map(|TokenInfo { kind, span, .. }| (kind, span))
            .collect();

        assert_eq!(
            rowan_tokens,
            lexer_tokens,
            "rowan token view diverged from the lexer for {}",
            file.display()
        );
    }
    // Non-vacuous by construction: the corpus is fully lexable today; a
    // future deliberately-unlexable fixture must update this assertion
    // explicitly rather than silently shrinking coverage.
    assert_eq!(compared, total, "every corpus file must be compared");
}

#[test]
fn token_view_matches_the_lexer_on_lexable_but_unparseable_sources() {
    // The path the old oracle differential could not take (it required
    // parse success): grammatically invalid but lexable inputs still get
    // an exact token-view parity check, because parse_cst is
    // error-recovering while the token stream is pure lexer truth.
    for src in [
        "def f( {
",
        "def f() { 1 +++ }
",
        ")))
",
        "@caps(
",
    ] {
        let lexed = lex_source(src).expect("snippet must lex");
        let lexer_tokens: Vec<(TokenKind, Span)> = lexed
            .into_iter()
            .filter(|t| !matches!(t.kind, TokenKind::Eof))
            .map(|t| (t.kind, t.span))
            .collect();
        let rowan_tokens: Vec<(TokenKind, Span)> = token_infos(parse_cst(src).syntax())
            .into_iter()
            .map(|TokenInfo { kind, span, .. }| (kind, span))
            .collect();
        assert_eq!(rowan_tokens, lexer_tokens, "diverged on {src:?}");
    }
}

#[test]
fn identifier_spans_preserve_the_rename_surface() {
    let src = "def greet(name) { greet(name) }\n";
    let parse = parse_cst(src);
    let spans = identifier_spans(parse.syntax(), "greet");
    let texts: Vec<&str> = spans
        .iter()
        .map(|span| &src[span.start..span.end()])
        .collect();

    assert_eq!(texts, vec!["greet", "greet"]);
}
