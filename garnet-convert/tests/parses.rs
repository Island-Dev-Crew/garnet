//! C1-18 (Q48): every converter output parses as Garnet.
//!
//! The `@sandbox` marker is a comment, blocks the converter cannot lower become
//! whole-statement `@migrate_todo` comments, and simple loops are lowered to
//! brace form, so `garnet parse` accepts the file as emitted.

use garnet_convert::{convert, EmitOpts, SourceLang};

fn convert_src(src: &str, lang: SourceLang, lang_name: &str, file: &str) -> (String, String) {
    let opts = EmitOpts {
        source_lang: lang_name.into(),
        source_file: file.into(),
        target_file: format!("{file}.garnet"),
        source_loc: src.lines().count(),
        strict: false,
        fail_on_todo: false,
        fail_on_untranslatable: false,
    };
    let (out, _) = convert(src, lang, file, opts).unwrap();
    (out.garnet, out.migrate_todo_md)
}

fn assert_parses(garnet: &str) {
    if let Err(e) = garnet_parser::parse_source(garnet) {
        panic!("converter output does not parse: {e:?}\n----\n{garnet}");
    }
}

/// Lines that are code, not comments or blank.
fn code_lines(garnet: &str) -> Vec<&str> {
    garnet
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect()
}

const HOMEPAGE_RUBY: &str = "def parse_config(text)\n  text.split(\"\\n\").map do |line|\n    k, v = line.split(\"=\", 2)\n    [k.strip, v.strip]\n  end.to_h\nend\n";

#[test]
fn homepage_parse_config_rb_parses_with_the_block_kept_as_a_todo() {
    let (garnet, checklist) = convert_src(HOMEPAGE_RUBY, SourceLang::Ruby, "ruby", "parse_config.rb");
    assert_parses(&garnet);
    let code = code_lines(&garnet);
    assert!(code.iter().any(|l| l.starts_with("def parse_config(text)")), "{garnet}");
    // The `.map do |line| ... end.to_h` block is kept whole, as comments.
    assert!(!code.iter().any(|l| l.contains("do |line|") || l.starts_with("k, v")), "{garnet}");
    assert!(garnet.contains("# @migrate_todo:"), "{garnet}");
    assert!(garnet.contains(".map do |line|"), "the block text survives in the comment: {garnet}");
    assert!(checklist.contains("require human review"), "{checklist}");
}

#[test]
fn python_for_loop_is_lowered_to_brace_form() {
    let src = "def total(items):\n    s = 0\n    for x in items:\n        s = s + x\n    return s\n";
    let (garnet, _) = convert_src(src, SourceLang::Python, "python", "loop.py");
    assert_parses(&garnet);
    let code = code_lines(&garnet);
    let for_at = code.iter().position(|l| *l == "for x in items {").expect(&garnet);
    assert_eq!(code[for_at + 1], "s = s + x", "{garnet}");
    assert_eq!(code[for_at + 2], "}", "the loop body stays inside the loop: {garnet}");
}

#[test]
fn python_with_block_becomes_a_whole_statement_todo() {
    let src = "def save(path, data):\n    with open(path, \"w\") as f:\n        f.write(data)\n    return True\n";
    let (garnet, _) = convert_src(src, SourceLang::Python, "python", "save.py");
    assert_parses(&garnet);
    let code = code_lines(&garnet);
    assert!(!code.iter().any(|l| l.contains("with open") || l.contains("f.write")), "{garnet}");
    assert!(garnet.contains("with open(path, \"w\") as f:"), "{garnet}");
    assert!(garnet.contains("f.write(data)"), "{garnet}");
}

#[test]
fn python_if_else_is_kept_whole_not_flattened() {
    let src = "def sign(n):\n    if n > 0:\n        return 1\n    else:\n        return -1\n";
    let (garnet, _) = convert_src(src, SourceLang::Python, "python", "sign.py");
    assert_parses(&garnet);
    let code = code_lines(&garnet);
    assert!(!code.iter().any(|l| *l == "return 1" || *l == "return -1"), "{garnet}");
}

#[test]
fn ruby_each_block_is_lowered_to_brace_form() {
    let src = "def total(xs)\n  s = 0\n  xs.each do |x|\n    s = s + x\n  end\n  s\nend\n";
    let (garnet, _) = convert_src(src, SourceLang::Ruby, "ruby", "total.rb");
    assert_parses(&garnet);
    let code = code_lines(&garnet);
    let for_at = code.iter().position(|l| *l == "for x in xs {").expect(&garnet);
    assert_eq!(code[for_at + 1], "s = s + x", "{garnet}");
    assert_eq!(code[for_at + 2], "}", "{garnet}");
    assert!(code.iter().any(|l| *l == "s"), "the trailing value stays in the def: {garnet}");
}

#[test]
fn rust_and_go_functions_parse() {
    let (rust, _) = convert_src("fn add(a: i64, b: i64) -> i64 {\n    a + b\n}\n", SourceLang::Rust, "rust", "add.rs");
    assert_parses(&rust);
    let (go, _) = convert_src("package main\n\nfunc add(a int, b int) int {\n\treturn a + b\n}\n", SourceLang::Go, "go", "add.go");
    assert_parses(&go);
}

#[test]
fn sandbox_marker_is_a_comment_and_no_unquarantine_advice_remains() {
    let (garnet, checklist) = convert_src(HOMEPAGE_RUBY, SourceLang::Ruby, "ruby", "parse_config.rb");
    assert!(!garnet.lines().any(|l| l.trim() == "@sandbox"), "{garnet}");
    assert!(garnet.contains("@sandbox"), "the marker is still named in a comment: {garnet}");
    for text in [&garnet, &checklist] {
        assert!(!text.contains("unquarantine"), "{text}");
    }
}

#[test]
fn checklist_without_todos_does_not_claim_a_clean_conversion() {
    let (garnet, checklist) = convert_src("fn one() -> i64 {\n    1\n}\n", SourceLang::Rust, "rust", "one.rs");
    assert_parses(&garnet);
    assert!(!checklist.contains("conversion was clean"), "{checklist}");
    assert!(checklist.contains("No migration to-dos were recorded"), "{checklist}");
}
