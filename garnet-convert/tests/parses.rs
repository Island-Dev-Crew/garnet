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
    let (garnet, checklist) =
        convert_src(HOMEPAGE_RUBY, SourceLang::Ruby, "ruby", "parse_config.rb");
    assert_parses(&garnet);
    let code = code_lines(&garnet);
    assert!(
        code.iter().any(|l| l.starts_with("def parse_config(text)")),
        "{garnet}"
    );
    // The `.map do |line| ... end.to_h` block is kept whole, as comments.
    assert!(
        !code
            .iter()
            .any(|l| l.contains("do |line|") || l.starts_with("k, v")),
        "{garnet}"
    );
    assert!(garnet.contains("# @migrate_todo:"), "{garnet}");
    assert!(
        garnet.contains(".map do |line|"),
        "the block text survives in the comment: {garnet}"
    );
    assert!(checklist.contains("require human review"), "{checklist}");
}

#[test]
fn python_for_loop_is_lowered_to_brace_form() {
    let src =
        "def total(items):\n    s = 0\n    for x in items:\n        s = s + x\n    return s\n";
    let (garnet, _) = convert_src(src, SourceLang::Python, "python", "loop.py");
    assert_parses(&garnet);
    let code = code_lines(&garnet);
    let for_at = code
        .iter()
        .position(|l| *l == "for x in items {")
        .expect(&garnet);
    assert_eq!(code[for_at + 1], "s = s + x", "{garnet}");
    assert_eq!(
        code[for_at + 2],
        "}",
        "the loop body stays inside the loop: {garnet}"
    );
}

#[test]
fn python_with_block_becomes_a_whole_statement_todo() {
    let src = "def save(path, data):\n    with open(path, \"w\") as f:\n        f.write(data)\n    return True\n";
    let (garnet, _) = convert_src(src, SourceLang::Python, "python", "save.py");
    assert_parses(&garnet);
    let code = code_lines(&garnet);
    assert!(
        !code
            .iter()
            .any(|l| l.contains("with open") || l.contains("f.write")),
        "{garnet}"
    );
    assert!(garnet.contains("with open(path, \"w\") as f:"), "{garnet}");
    assert!(garnet.contains("f.write(data)"), "{garnet}");
}

#[test]
fn python_if_else_is_kept_whole_not_flattened() {
    let src = "def sign(n):\n    if n > 0:\n        return 1\n    else:\n        return -1\n";
    let (garnet, _) = convert_src(src, SourceLang::Python, "python", "sign.py");
    assert_parses(&garnet);
    let code = code_lines(&garnet);
    assert!(
        !code.iter().any(|l| *l == "return 1" || *l == "return -1"),
        "{garnet}"
    );
}

#[test]
fn ruby_each_block_is_lowered_to_brace_form() {
    let src = "def total(xs)\n  s = 0\n  xs.each do |x|\n    s = s + x\n  end\n  s\nend\n";
    let (garnet, _) = convert_src(src, SourceLang::Ruby, "ruby", "total.rb");
    assert_parses(&garnet);
    let code = code_lines(&garnet);
    let for_at = code
        .iter()
        .position(|l| *l == "for x in xs {")
        .expect(&garnet);
    assert_eq!(code[for_at + 1], "s = s + x", "{garnet}");
    assert_eq!(code[for_at + 2], "}", "{garnet}");
    assert!(
        code.contains(&"s"),
        "the trailing value stays in the def: {garnet}"
    );
}

#[test]
fn rust_and_go_functions_parse() {
    let (rust, _) = convert_src(
        "fn add(a: i64, b: i64) -> i64 {\n    a + b\n}\n",
        SourceLang::Rust,
        "rust",
        "add.rs",
    );
    assert_parses(&rust);
    let (go, _) = convert_src(
        "package main\n\nfunc add(a int, b int) int {\n\treturn a + b\n}\n",
        SourceLang::Go,
        "go",
        "add.go",
    );
    assert_parses(&go);
}

#[test]
fn sandbox_marker_is_a_comment_and_no_unquarantine_advice_remains() {
    let (garnet, checklist) =
        convert_src(HOMEPAGE_RUBY, SourceLang::Ruby, "ruby", "parse_config.rb");
    assert!(!garnet.lines().any(|l| l.trim() == "@sandbox"), "{garnet}");
    assert!(
        garnet.contains("@sandbox"),
        "the marker is still named in a comment: {garnet}"
    );
    for text in [&garnet, &checklist] {
        assert!(!text.contains("unquarantine"), "{text}");
    }
}

#[test]
fn checklist_without_todos_does_not_claim_a_clean_conversion() {
    let (garnet, checklist) = convert_src(
        "fn one() -> i64 {\n    1\n}\n",
        SourceLang::Rust,
        "rust",
        "one.rs",
    );
    assert_parses(&garnet);
    assert!(!checklist.contains("conversion was clean"), "{checklist}");
    assert!(
        checklist.contains("No migration to-dos were recorded"),
        "{checklist}"
    );
}

// Codex lane B on #607 (01751aca): a trailing comment on a block header, a
// statement that spans physical lines, or a Go block read line by line left
// fragments of the statement active while its header became a comment.

fn assert_inactive(garnet: &str, fragment: &str) {
    assert!(
        !code_lines(garnet).iter().any(|l| l.contains(fragment)),
        "`{fragment}` must not be active code:\n{garnet}"
    );
}

#[test]
fn ruby_each_with_a_trailing_comment_is_still_lowered_inside_its_def() {
    let src = "def save_all(items)\n  items.each do |item| # each enabled item\n    persist(item)\n  end\n  return 0\nend\n";
    let (garnet, _) = convert_src(src, SourceLang::Ruby, "ruby", "save_all.rb");
    assert_parses(&garnet);
    assert!(
        !garnet.contains("unparsed Ruby statement: return 0"),
        "the def must not close early:\n{garnet}"
    );
    let code = code_lines(&garnet);
    assert!(
        code.iter().any(|l| l.starts_with("for item in items")),
        "{garnet}"
    );
    assert!(code.contains(&"return 0"), "{garnet}");
}

#[test]
fn ruby_keyword_block_with_a_trailing_comment_is_kept_whole() {
    let src =
        "def save(ready)\n  if ready # only when ready\n    persist()\n  end\n  return 0\nend\n";
    let (garnet, _) = convert_src(src, SourceLang::Ruby, "ruby", "save.rb");
    assert_parses(&garnet);
    assert_inactive(&garnet, "persist()");
    assert!(
        !garnet.contains("unparsed Ruby statement: return 0"),
        "{garnet}"
    );
}

#[test]
fn python_block_header_with_a_trailing_comment_is_kept_whole() {
    let src =
        "def save(enabled):\n    if enabled:  # optional save\n        persist()\n    return 0\n";
    let (garnet, _) = convert_src(src, SourceLang::Python, "python", "save.py");
    assert_parses(&garnet);
    assert_inactive(&garnet, "persist()");
    assert!(garnet.contains("if enabled:  # optional save"), "{garnet}");
}

#[test]
fn python_statement_spanning_lines_is_kept_whole() {
    let src = "def run():\n    result = process(\n        persist()\n    )\n    return result\n";
    let (garnet, _) = convert_src(src, SourceLang::Python, "python", "run.py");
    assert_parses(&garnet);
    assert_inactive(&garnet, "persist()");
    assert!(garnet.contains("result = process("), "{garnet}");
}

#[test]
fn go_block_is_kept_whole_and_does_not_close_the_function() {
    for header in ["if enabled {", "if enabled { // optional save"] {
        let src = format!(
            "package main\n\nfunc save(enabled bool) int {{\n\t{header}\n\t\treturn persist()\n\t}}\n\treturn 0\n}}\n"
        );
        let (garnet, _) = convert_src(&src, SourceLang::Go, "go", "save.go");
        assert_parses(&garnet);
        assert_inactive(&garnet, "persist()");
        assert!(
            !garnet.contains("unparsed Go item: return 0"),
            "{header}:\n{garnet}"
        );
        assert!(
            code_lines(&garnet).contains(&"return 0"),
            "{header}:\n{garnet}"
        );
    }
}

#[test]
fn go_statement_spanning_lines_is_kept_whole() {
    let src = "package main\n\nfunc run() int {\n\tresult := process(\n\t\tpersist(),\n\t)\n\treturn result\n}\n";
    let (garnet, _) = convert_src(src, SourceLang::Go, "go", "run.go");
    assert_parses(&garnet);
    assert_inactive(&garnet, "persist()");
}

#[test]
fn go_closure_block_is_kept_whole_and_does_not_close_the_function() {
    let src =
        "package main\n\nfunc run() int {\n\tdefer func() {\n\t\tpersist()\n\t}()\n\treturn 0\n}\n";
    let (garnet, _) = convert_src(src, SourceLang::Go, "go", "run.go");
    assert_parses(&garnet);
    assert_inactive(&garnet, "persist()");
    assert!(!garnet.contains("unparsed Go item: return 0"), "{garnet}");
    assert!(code_lines(&garnet).contains(&"return 0"), "{garnet}");
}

// Codex delta review on #607 (3d77d8e9): variants of the same class. Python and Go
// are lexed fully; Ruby outside the lexed subset is refused, never half-converted.

fn try_convert(src: &str, lang: SourceLang, lang_name: &str, file: &str) -> Result<String, String> {
    let opts = EmitOpts {
        source_lang: lang_name.into(),
        source_file: file.into(),
        target_file: format!("{file}.garnet"),
        source_loc: src.lines().count(),
        strict: false,
        fail_on_todo: false,
        fail_on_untranslatable: false,
    };
    convert(src, lang, file, opts)
        .map(|(out, _)| out.garnet)
        .map_err(|e| e.to_string())
}

#[test]
fn python_block_whose_header_spans_lines_is_kept_whole() {
    let src =
        "def save(enabled):\n    if (\n        enabled\n    ):\n        persist()\n    return 0\n";
    let (garnet, _) = convert_src(src, SourceLang::Python, "python", "save.py");
    assert_parses(&garnet);
    assert_inactive(&garnet, "persist()");
    assert!(code_lines(&garnet).contains(&"return 0"), "{garnet}");
}

#[test]
fn python_code_inside_a_multiline_string_is_never_active() {
    let src = "def run():\n    \"\"\"Docs.\n\npersist()\n    \"\"\"\n    return 0\n";
    let (garnet, _) = convert_src(src, SourceLang::Python, "python", "run.py");
    assert_parses(&garnet);
    assert_inactive(&garnet, "persist()");
}

#[test]
fn python_fstring_with_its_own_quote_in_a_field_is_refused() {
    let src = "def run(d):\n    return f\"{d[\"k\"]}\"\n";
    let err = try_convert(src, SourceLang::Python, "python", "run.py").unwrap_err();
    assert!(err.contains("f-string"), "{err}");
}

#[test]
fn go_braces_inside_comments_and_raw_strings_do_not_count() {
    let cases = [
        "\tif enabled { /* } */\n\t\tpersist()\n\t}\n",
        "\t/*\n\tpersist()\n\t*/\n",
        "\ts := `{\n\tpersist()\n\t`\n\t_ = s\n",
    ];
    for body in cases {
        let src = format!("package main\n\nfunc run(enabled bool) int {{\n{body}\treturn 0\n}}\n");
        let (garnet, _) = convert_src(&src, SourceLang::Go, "go", "run.go");
        assert_parses(&garnet);
        assert_inactive(&garnet, "persist()");
        assert!(
            !garnet.contains("unparsed Go item: return 0"),
            "{body}:\n{garnet}"
        );
        assert!(
            code_lines(&garnet).contains(&"return 0"),
            "{body}:\n{garnet}"
        );
    }
}

#[test]
fn ruby_outside_the_lexed_subset_is_refused() {
    for (line, what) in [
        ("  %w[#].each do |item|", "percent literal"),
        ("  text = <<~EOS", "heredoc"),
        ("  c = ?#", "character literal"),
        ("  puts \"#{h[\"k\"]}\"", "interpolation"),
    ] {
        let src = format!("def run(items)\n{line}\n    persist(item)\n  end\n  return 0\nend\n");
        let err = try_convert(&src, SourceLang::Ruby, "ruby", "run.rb").unwrap_err();
        assert!(
            err.contains(what) && err.contains("line 2"),
            "{what}: {err}"
        );
    }
}

#[test]
fn ruby_regex_with_a_hash_does_not_hide_a_block_header() {
    let src = "def run(line)\n  if line =~ /#/\n    persist()\n  end\n  return 0\nend\n";
    let (garnet, _) = convert_src(src, SourceLang::Ruby, "ruby", "run.rb");
    assert_parses(&garnet);
    assert_inactive(&garnet, "persist()");
    assert!(code_lines(&garnet).contains(&"return 0"), "{garnet}");
}

// The statement is a defined unit per language, so no variant of a split
// statement, block or definition can leave a fragment active:
// - Python: a logical line (brackets, `\`, triple-quoted strings) and, for a
//   compound statement, its indented body and clauses;
// - Go: lines up to where Go inserts a semicolon, once every bracket, block
//   comment and raw string is closed;
// - Ruby (the lexed subset): lines up to where every bracket and keyword block
//   is closed, the line does not end in an operator, and the next line does not
//   start with `.`.

/// No line of active code is exactly `line`.
fn assert_no_code_line(garnet: &str, line: &str) {
    assert!(
        !code_lines(garnet).contains(&line),
        "`{line}` must not be a line of active code:\n{garnet}"
    );
}

#[test]
fn python_def_whose_signature_spans_lines_keeps_only_its_body_active() {
    let src = "def save(\n    path,\n    data,\n):\n    persist(data)\n    return 0\n";
    let (garnet, _) = convert_src(src, SourceLang::Python, "python", "save.py");
    assert_parses(&garnet);
    for fragment in ["path,", "data,", "):"] {
        assert_no_code_line(&garnet, fragment);
    }
    let code = code_lines(&garnet);
    assert!(code.iter().any(|l| l.starts_with("def save(")), "{garnet}");
    assert!(code.contains(&"return 0"), "{garnet}");
    // A header continued with `\` reads the same as one continued in brackets.
    let src = "def save(path, \\\n         data):\n    return 0\n";
    let (garnet, _) = convert_src(src, SourceLang::Python, "python", "save.py");
    assert_parses(&garnet);
    let code = code_lines(&garnet);
    assert!(code.contains(&"def save(path, data) {"), "{garnet}");
    assert!(code.contains(&"return 0"), "{garnet}");
}

#[test]
fn python_decorated_definition_is_kept_whole() {
    let src =
        "@cached\ndef load():\n    def inner():\n        return persist()\n    return inner()\n";
    let (garnet, _) = convert_src(src, SourceLang::Python, "python", "load.py");
    assert_parses(&garnet);
    assert_inactive(&garnet, "persist()");
    assert_inactive(&garnet, "inner()");
}

#[test]
fn python_decorated_method_is_kept_whole_and_not_dropped() {
    let src = "class Store:\n    @staticmethod\n    def build():\n        return persist()\n";
    let (garnet, _) = convert_src(src, SourceLang::Python, "python", "store.py");
    assert_parses(&garnet);
    assert_inactive(&garnet, "persist()");
    assert!(
        garnet.contains("@staticmethod"),
        "the decorated method survives as a todo:\n{garnet}"
    );
}

#[test]
fn python_module_level_block_keeps_its_definitions_inside() {
    let src = "if FAST:\n    def run():\n        return persist()\n";
    let (garnet, _) = convert_src(src, SourceLang::Python, "python", "fast.py");
    assert_parses(&garnet);
    assert_inactive(&garnet, "persist()");
}

// A guard, not a red test: the emitter already quarantines `if x: ...` because it
// does not parse as Garnet. The frontend now keeps it whole itself.
#[test]
fn python_compound_statement_on_one_line_is_not_active() {
    let src = "def run(x):\n    if x: persist()\n    return 0\n";
    let (garnet, _) = convert_src(src, SourceLang::Python, "python", "run.py");
    assert_parses(&garnet);
    assert_inactive(&garnet, "persist()");
    assert!(code_lines(&garnet).contains(&"return 0"), "{garnet}");
}

#[test]
fn python_tab_indentation_is_refused() {
    let err = try_convert(
        "def run():\n\treturn persist()\n",
        SourceLang::Python,
        "python",
        "run.py",
    )
    .unwrap_err();
    assert!(err.contains("tab") && err.contains("line 2"), "{err}");
}

#[test]
fn go_one_line_function_ends_where_its_brace_closes() {
    let src =
        "package main\n\nfunc one() int { return 1 }\n\nfunc run() int {\n\treturn persist()\n}\n";
    let (garnet, _) = convert_src(src, SourceLang::Go, "go", "one.go");
    assert_parses(&garnet);
    let code = code_lines(&garnet);
    assert!(code.contains(&"return 1"), "{garnet}");
    assert!(code.contains(&"return persist()"), "{garnet}");
    assert!(code.iter().any(|l| l.starts_with("fn run(")), "{garnet}");
}

#[test]
fn go_comment_or_raw_string_at_top_level_hides_its_functions() {
    let src = "package main\n\n/*\nfunc old() int {\n\treturn persist()\n}\n*/\n\nvar tmpl = `\nfunc gen() int {\n\treturn persist()\n}\n`\n\nfunc run() int {\n\treturn 0\n}\n";
    let (garnet, _) = convert_src(src, SourceLang::Go, "go", "tmpl.go");
    assert_parses(&garnet);
    assert_inactive(&garnet, "persist()");
    assert!(code_lines(&garnet).contains(&"return 0"), "{garnet}");
}

#[test]
fn go_statement_continued_by_a_trailing_operator_is_kept_whole() {
    let src =
        "package main\n\nfunc run() int {\n\ttotal := 1 +\n\t\tpersist()\n\treturn total\n}\n";
    let (garnet, _) = convert_src(src, SourceLang::Go, "go", "run.go");
    assert_parses(&garnet);
    assert_inactive(&garnet, "persist()");
    assert!(code_lines(&garnet).contains(&"return total"), "{garnet}");
}

#[test]
fn go_bare_block_is_kept_whole() {
    let src =
        "package main\n\nfunc run() int {\n\tx := 0\n\t{\n\t\tx = persist()\n\t}\n\treturn x\n}\n";
    let (garnet, _) = convert_src(src, SourceLang::Go, "go", "run.go");
    assert_parses(&garnet);
    assert_inactive(&garnet, "persist()");
    assert!(code_lines(&garnet).contains(&"return x"), "{garnet}");
}

#[test]
fn ruby_block_opened_in_expression_position_is_kept_whole() {
    let src =
        "def run(ready)\n  x = if ready\n    persist()\n  else\n    0\n  end\n  return x\nend\n";
    let (garnet, _) = convert_src(src, SourceLang::Ruby, "ruby", "run.rb");
    assert_parses(&garnet);
    assert_inactive(&garnet, "persist()");
    assert!(code_lines(&garnet).contains(&"return x"), "{garnet}");
}

#[test]
fn ruby_statement_spanning_lines_is_kept_whole() {
    let cases = [
        "  x = compute(\n    persist()\n  )\n",
        "  items.map { |i|\n    persist(i)\n  }\n",
        "  total = 1 +\n    persist()\n",
        "  result = items\n    .map(&:persist)\n",
    ];
    for body in cases {
        let src = format!("def run(items)\n{body}  return 0\nend\n");
        let (garnet, _) = convert_src(&src, SourceLang::Ruby, "ruby", "run.rb");
        assert_parses(&garnet);
        assert_inactive(&garnet, "persist");
        assert!(
            code_lines(&garnet).contains(&"return 0"),
            "{body}:\n{garnet}"
        );
    }
}

#[test]
fn ruby_def_header_the_frontend_does_not_read_is_kept_whole() {
    for src in [
        "def self.build(x)\n  persist(x)\nend\n",
        "def build x\n  persist(x)\nend\n",
    ] {
        let (garnet, _) = convert_src(src, SourceLang::Ruby, "ruby", "build.rb");
        assert_parses(&garnet);
        assert_inactive(&garnet, "persist(x)");
    }
}

#[test]
fn ruby_endless_method_is_refused() {
    let err = try_convert("def one = persist()\n", SourceLang::Ruby, "ruby", "one.rb").unwrap_err();
    assert!(err.contains("endless") && err.contains("line 1"), "{err}");
}

#[test]
fn ruby_top_level_block_keeps_its_definitions_inside() {
    let src = "if ENV[\"FAST\"]\n  def run\n    persist()\n  end\nend\n";
    let (garnet, _) = convert_src(src, SourceLang::Ruby, "ruby", "fast.rb");
    assert_parses(&garnet);
    assert_inactive(&garnet, "persist()");
}

// Codex delta round 3 on #607: a line ending in a keyword that takes an operand
// continues the statement. Go inserts no semicolon after `go`, `defer` or the
// other keywords outside break/continue/fallthrough/return; a Ruby modifier
// (`if`, `unless`, `while`, `until`, `rescue`) at a line end takes the next line.

#[test]
fn go_line_ending_in_a_keyword_continues_its_statement() {
    for keyword in ["go", "defer"] {
        let src = format!(
            "package main\n\nfunc run() int {{\n\t{keyword}\n\t\tpersist()\n\treturn 0\n}}\n"
        );
        let (garnet, _) = convert_src(&src, SourceLang::Go, "go", "run.go");
        assert_parses(&garnet);
        assert_inactive(&garnet, "persist()");
        assert!(
            code_lines(&garnet).contains(&"return 0"),
            "{keyword}:\n{garnet}"
        );
    }
}

#[test]
fn ruby_line_ending_in_a_modifier_continues_its_statement() {
    for body in [
        "  return 7 if\n    persist()\n",
        "  x = 1 unless\n    persist()\n",
        "  x = fetch rescue\n    persist()\n",
    ] {
        let src = format!("def run\n{body}  return 0\nend\n");
        let (garnet, _) = convert_src(&src, SourceLang::Ruby, "ruby", "run.rb");
        assert_parses(&garnet);
        assert_inactive(&garnet, "persist()");
        assert!(
            code_lines(&garnet).contains(&"return 0"),
            "{body}:\n{garnet}"
        );
    }
}

// Codex lane B, round 3 on #607 (7b788fd2).

#[test]
fn python_keyword_followed_by_any_non_identifier_character_opens_its_block() {
    for header in [
        "if\tenabled:",
        "if{1}:",
        "for\tx in items:",
        "while(ready):",
    ] {
        let src = format!(
            "def save(enabled, items, ready):\n    {header}\n        persist()\n    return 0\n"
        );
        let (garnet, _) = convert_src(&src, SourceLang::Python, "python", "save.py");
        assert_parses(&garnet);
        assert_inactive(&garnet, "persist()");
        assert!(
            code_lines(&garnet).contains(&"return 0"),
            "{header}:\n{garnet}"
        );
    }
    let src = "def save():\n    try:\n        attempt()\n    except* ValueError:\n        persist()\n    return 0\n";
    let (garnet, _) = convert_src(src, SourceLang::Python, "python", "save.py");
    assert_parses(&garnet);
    assert_inactive(&garnet, "persist()");
}

#[test]
fn python_fstring_field_holding_its_quote_or_a_comment_is_refused() {
    for src in [
        "def save():\n    text = f\"\"\"{ \"\"\"\npersist()\n\"\"\" }\"\"\"\n    return 0\n",
        "def save(d):\n    text = f\"\"\"{d[\"\"\"key\"\"\"]}\"\"\"\n    return 0\n",
        "def save(d):\n    text = t\"{d[\"key\"]}\"\n    return 0\n",
        "def save(x):\n    text = f\"\"\"{x # }\n}\"\"\"\n    return 0\n",
    ] {
        let err = try_convert(src, SourceLang::Python, "python", "save.py").unwrap_err();
        assert!(
            err.contains("string") && err.contains("line 2"),
            "{src:?}: {err}"
        );
    }
}

#[test]
fn python_constructor_is_kept_not_dropped() {
    for src in [
        "class Store:\n    def __init__(self):\n        persist()\n",
        "class Store:\n    def __init__(self):\n        if ready:\n            persist()\n",
    ] {
        let (garnet, _) = convert_src(src, SourceLang::Python, "python", "store.py");
        assert_parses(&garnet);
        assert!(
            garnet.contains("persist()"),
            "the constructor's body survives:\n{garnet}"
        );
    }
}

#[test]
fn ruby_blank_or_comment_line_does_not_end_a_continued_statement() {
    for body in [
        "  enabled &&\n\n    persist()\n",
        "  enabled &&\n  # still the same expression\n    persist()\n",
        "  return 7 if\n  # condition follows\n    persist()\n",
    ] {
        let src = format!("def save(enabled)\n{body}  return 0\nend\n");
        let (garnet, _) = convert_src(&src, SourceLang::Ruby, "ruby", "save.rb");
        assert_parses(&garnet);
        assert_inactive(&garnet, "persist()");
        assert!(
            code_lines(&garnet).contains(&"return 0"),
            "{body}:\n{garnet}"
        );
    }
}

#[test]
fn ruby_heredoc_and_percent_literal_forms_are_refused() {
    for (line, what) in [
        ("  value = <<text", "heredoc"),
        ("  value = %q hello ", "percent literal"),
        ("  value = %q=hello=", "percent literal"),
        ("  value = % hello ", "percent literal"),
    ] {
        let src = format!("def save\n{line}\n  return 0\nend\n");
        let err = try_convert(&src, SourceLang::Ruby, "ruby", "save.rb").unwrap_err();
        assert!(
            err.contains(what) && err.contains("line 2"),
            "{line}: {err}"
        );
    }
}

#[test]
fn ruby_definition_with_more_code_on_its_end_line_is_kept_whole() {
    for src in [
        "def save\n  persist()\nend if false\n",
        "class Store\n  def save\n    persist()\n  end\nend if false\n",
        "def run(xs)\n  xs.each do |x|\n    persist(x)\n  end if false\n  0\nend\n",
    ] {
        let (garnet, _) = convert_src(src, SourceLang::Ruby, "ruby", "save.rb");
        assert_parses(&garnet);
        assert_inactive(&garnet, "persist(");
    }
}

#[test]
fn an_import_sharing_its_line_keeps_what_follows() {
    let (go, _) = convert_src(
        "package example\nimport \"fmt\"; func save() { fmt.Println(\"saved\") }\n",
        SourceLang::Go,
        "go",
        "save.go",
    );
    assert_parses(&go);
    assert!(go.contains("saved"), "the definition is kept:\n{go}");
    let (ruby, _) = convert_src(
        "require \"x\"; def save\n  persist()\nend\n",
        SourceLang::Ruby,
        "ruby",
        "save.rb",
    );
    assert_parses(&ruby);
    assert!(
        ruby.contains("persist()"),
        "the definition is kept:\n{ruby}"
    );
    let (python, _) = convert_src(
        "import os; value = persist()\n",
        SourceLang::Python,
        "python",
        "save.py",
    );
    assert_parses(&python);
    assert!(
        python.contains("persist()"),
        "the statement is kept:\n{python}"
    );
}

// Codex lane B, round 4 on #607 (6a3f3523).

#[test]
fn python_escape_before_a_brace_does_not_hide_a_replacement_field() {
    for src in [
        "def save():\n    text = rf\"\"\"\\{ \"\"\"\n    persist()\n\"\"\" }\"\"\"\n",
        "def save(d):\n    text = rf\"\\{d[\"key\"]}\"\n    return 0\n",
        "def save():\n    text = rt\"\"\"\\{ \"\"\"\n    persist()\n\"\"\" }\"\"\"\n",
    ] {
        let err = try_convert(src, SourceLang::Python, "python", "save.py").unwrap_err();
        assert!(
            err.contains("own quote") && err.contains("line 2"),
            "{src:?}: {err}"
        );
    }
    // A named character escape in a non-raw f-string is not a field.
    let (garnet, _) = convert_src(
        "def save():\n    text = f\"\\N{EM DASH}\"\n    return 0\n",
        SourceLang::Python,
        "python",
        "save.py",
    );
    assert_parses(&garnet);
}

#[test]
fn python_inline_suite_keeps_its_clauses() {
    for header in ["if ready: pass", "for x in items: pass"] {
        let src = format!(
            "def save(ready, items):\n    {header}\n    else:\n        persist()\n    return 0\n"
        );
        let (garnet, checklist) = convert_src(&src, SourceLang::Python, "python", "save.py");
        assert_parses(&garnet);
        assert_inactive(&garnet, "persist()");
        assert_eq!(
            1,
            garnet.matches("@migrate_todo:").count(),
            "{header}: one whole statement:\n{garnet}\n{checklist}"
        );
    }
}

#[test]
fn ruby_spaced_percent_or_slash_after_a_word_is_refused() {
    for line in ["  puts  %q=hello", "  puts\t%q=hello", "  puts  /hello"] {
        let src = format!("def save\n{line}\n  persist()\n=\n  return 0\nend\n");
        let err = try_convert(&src, SourceLang::Ruby, "ruby", "save.rb").unwrap_err();
        assert!(
            err.contains("ambiguous") && err.contains("line 2"),
            "{line:?}: {err}"
        );
    }
}

#[test]
fn ruby_require_is_skipped_only_in_its_exact_form() {
    for src in [
        "require = def save\n  persist()\nend\n",
        "require(\"x\") && (def save\n persist()\nend)\n",
        "require! do\n def save\n  persist()\n end\nend\n",
        "require(\"x\") do\n def save\n  persist()\n end\nend\n",
    ] {
        let (garnet, _) = convert_src(src, SourceLang::Ruby, "ruby", "save.rb");
        assert_parses(&garnet);
        assert!(
            garnet.contains("persist()"),
            "kept, not dropped: {src:?}\n{garnet}"
        );
    }
}

#[test]
fn go_function_header_is_read_lexically() {
    for src in [
        "package example\nfunc save() int /* { return 0 } func injected() { persist() } */ { return 1 }\n",
        "package example\nfunc save() int /* { return 0 }\nfunc injected() { persist() }\n*/ {\n return 1\n}\n",
        "package example\nfunc save(x int /* ) int { return 0 }\nfunc injected() { persist() }\n*/ ) int {\n return 1\n}\n",
    ] {
        let (garnet, _) = convert_src(src, SourceLang::Go, "go", "save.go");
        assert_parses(&garnet);
        assert_inactive(&garnet, "persist()");
        assert_inactive(&garnet, "injected");
    }
    // A struct return type's braces are not the body's.
    let (garnet, _) = convert_src(
        "package example\nfunc save() struct { value int } {\n persist()\n return struct{value int}{0}\n}\n",
        SourceLang::Go,
        "go",
        "save.go",
    );
    assert_parses(&garnet);
    assert_inactive(&garnet, "value int } {");
}

#[test]
fn go_struct_with_a_raw_tag_across_lines_is_kept_whole() {
    let src = "package example\ntype Store struct {\n Field int `\n}\nfunc injected() { persist() }\n`\n}\n";
    let (garnet, _) = convert_src(src, SourceLang::Go, "go", "store.go");
    assert_parses(&garnet);
    assert_inactive(&garnet, "persist()");
    assert_inactive(&garnet, "injected");
}

// Codex lane B, round 5 on #607 (23b5bcf0).

#[test]
fn ruby_method_name_continued_on_the_next_line_is_not_a_keyword() {
    for src in [
        "def save\n  if false\n    obj.\n      end\n    persist()\n  end\nend\n",
        "def save\n  if ready\n    obj. # continuation\n      end\n    persist()\n  end\nend\n",
        "if ready\n  obj.\n    end\n  def save\n    persist()\n  end\nend\n",
    ] {
        let (garnet, _) = convert_src(src, SourceLang::Ruby, "ruby", "save.rb");
        assert_parses(&garnet);
        assert_inactive(&garnet, "persist()");
    }
}

#[test]
fn ruby_regex_with_interpolation_is_refused() {
    for line in ["  text = /#{/x/}", "  x = /#{\"/\"}/", "  value = /#{1 # /"] {
        let src = format!("def save\n{line}\n  persist()\n  x/\nend\n");
        let err = try_convert(&src, SourceLang::Ruby, "ruby", "save.rb").unwrap_err();
        assert!(
            err.contains("regular expression") && err.contains("line 2"),
            "{line}: {err}"
        );
    }
}

#[test]
fn source_text_that_reads_as_garnet_interpolation_is_never_active() {
    for (src, lang, name, file) in [
        (
            "def save():\n    return \"#{persist()}\"\n",
            SourceLang::Python,
            "python",
            "save.py",
        ),
        (
            "def save():\n    return r\"#{persist()}\"\n",
            SourceLang::Python,
            "python",
            "save.py",
        ),
        (
            "package example\nfunc save() string { return \"#{persist()}\" }\n",
            SourceLang::Go,
            "go",
            "save.go",
        ),
        (
            "def save\n  puts '#{persist()}'\nend\n",
            SourceLang::Ruby,
            "ruby",
            "save.rb",
        ),
        (
            "def save\n  return '#{persist()}'\nend\n",
            SourceLang::Ruby,
            "ruby",
            "save.rb",
        ),
    ] {
        let (garnet, _) = convert_src(src, lang, name, file);
        assert_parses(&garnet);
        assert_inactive(&garnet, "#{");
        assert!(
            garnet.contains("persist()"),
            "kept as a to-do: {src:?}\n{garnet}"
        );
    }
}

#[test]
fn python_bare_carriage_return_line_ending_is_refused() {
    let err = try_convert(
        "# comment\rdef save():\r    return 7\r",
        SourceLang::Python,
        "python",
        "save.py",
    )
    .unwrap_err();
    assert!(
        err.contains("carriage return") && err.contains("line 1"),
        "{err}"
    );
}

#[test]
fn go_header_ends_where_go_ends_a_declaration() {
    for src in [
        "package example\nfunc external(); func save() { persist() }\n",
        "package example\nfunc external() /*\n*/ func save() { persist() }\n",
    ] {
        let (garnet, _) = convert_src(src, SourceLang::Go, "go", "save.go");
        assert_parses(&garnet);
        // The bodyless declaration is kept as a to-do with its text...
        assert!(garnet.contains("func external()"), "{src:?}\n{garnet}");
        assert!(
            !code_lines(&garnet)
                .iter()
                .any(|l| l.starts_with("fn external(")),
            "external has no body: {src:?}\n{garnet}"
        );
        // ...and save keeps its own body, inside it.
        let code = code_lines(&garnet);
        let at = code
            .iter()
            .position(|l| l.starts_with("fn save("))
            .unwrap_or_else(|| panic!("save is converted: {src:?}\n{garnet}"));
        assert_eq!(code[at + 1], "persist()", "{src:?}\n{garnet}");
        assert_eq!(code[at + 2], "}", "{src:?}\n{garnet}");
    }
}

// Codex lane B, round 6 on #607 (19e5b89b; stopped by the content filter before
// its report, the cases are from its log): a statement copied through as source
// text keeps the source's quoting, which Garnet can read differently, so string
// contents became code.

#[test]
fn copied_source_text_with_a_quote_is_never_active() {
    for (src, lang, name, file) in [
        (
            "def save():\n    \"\"\"x\"; persist(); \"y\"\"\"\n",
            SourceLang::Python,
            "python",
            "save.py",
        ),
        (
            "def save():\n    7\n    \"\"\"x\"; persist(); \"y\"\"\"\n",
            SourceLang::Python,
            "python",
            "save.py",
        ),
        (
            "def save():\n    \"\"\"x\"; persist(); #\"\"\"\n",
            SourceLang::Python,
            "python",
            "save.py",
        ),
        (
            "def save(x):\n    x = 'a\"; persist(); \"b'\n    return x\n",
            SourceLang::Python,
            "python",
            "save.py",
        ),
        (
            "package example\nfunc save() string {\n\tx := `a\"; persist(); \"b`\n\treturn x\n}\n",
            SourceLang::Go,
            "go",
            "save.go",
        ),
        (
            "def save\n  x = 'a\"; persist(); \"b'\n  return x\nend\n",
            SourceLang::Ruby,
            "ruby",
            "save.rb",
        ),
    ] {
        let (garnet, _) = convert_src(src, lang, name, file);
        assert_parses(&garnet);
        assert_inactive(&garnet, "persist()");
        assert!(
            garnet.contains("persist()"),
            "kept as a to-do: {src:?}\n{garnet}"
        );
    }
}

// Codex lane B, round 7 on #607 (a9273c92).

#[test]
fn ruby_alias_and_undef_operands_are_names() {
    for line in [
        "alias end foo",
        "alias foo end",
        "undef end",
        "undef foo, end",
    ] {
        let src = format!("def save\n  if false\n    {line}\n    persist()\n  end\nend\n");
        let (garnet, _) = convert_src(&src, SourceLang::Ruby, "ruby", "save.rb");
        assert_parses(&garnet);
        assert_inactive(&garnet, "persist()");
    }
}

#[test]
fn ruby_interpolation_beyond_plain_expressions_is_refused() {
    for line in [
        "  value = \"#{ /}/; \"#\" }\"",
        "  value = \"#{a / b}\"",
        "  value = \"#{x ? 1 : 2}\"",
        "  value = \"#{%w[a]}\"",
    ] {
        let src = format!("def save\n{line}\n  return 0\nend\n");
        let err = try_convert(&src, SourceLang::Ruby, "ruby", "save.rb").unwrap_err();
        assert!(
            err.contains("interpolation") && err.contains("line 2"),
            "{line}: {err}"
        );
    }
    let (garnet, _) = convert_src(
        "def save(a)\n  value = \"#{a.name}[#{a[0] + 1}]\"\n  return 0\nend\n",
        SourceLang::Ruby,
        "ruby",
        "save.rb",
    );
    assert_parses(&garnet);
}

#[test]
fn a_statement_sharing_a_line_with_eval_is_kept() {
    for (src, lang, name, file) in [
        (
            "eval \"1\"; def save\n  persist()\nend\n",
            SourceLang::Ruby,
            "ruby",
            "save.rb",
        ),
        (
            "instance_eval \"1\"; def save\n  persist()\nend\n",
            SourceLang::Ruby,
            "ruby",
            "save.rb",
        ),
        (
            "exec(\"1\"); value = persist()\n",
            SourceLang::Python,
            "python",
            "save.py",
        ),
    ] {
        let (garnet, _) = convert_src(src, lang, name, file);
        assert_parses(&garnet);
        assert!(garnet.contains("persist()"), "kept: {src:?}\n{garnet}");
    }
}

// Codex lane B, round 8 on #607 (a0b55b50; stopped by the content filter, the
// cases are from its drafted checks).

#[test]
fn ruby_alias_or_undef_operands_on_another_line_are_refused() {
    // Round 8 carried the operand mode over `,` and `\`; round 9 found Ruby also
    // takes operands after a bare newline. The subset now requires an `alias` or
    // `undef` to complete its operands on its own line.
    for line in [
        "undef foo,\n      end",
        "undef foo, # comment\n      end",
        "alias foo \\\n      end",
        "alias foo\n      end",
        "alias\n      foo end",
        "undef\n      end",
    ] {
        let src = format!("def save\n  if false\n    {line}\n    persist()\n  end\nend\n");
        let err = try_convert(&src, SourceLang::Ruby, "ruby", "save.rb").unwrap_err();
        assert!(
            err.contains("alias or undef") && err.contains("line 3"),
            "{line:?}: {err}"
        );
    }
}

#[test]
fn python_triple_quoted_string_inside_a_replacement_field_is_refused() {
    for src in [
        "def save():\n    value = f\"\"\"{''' ' } \"\"\"#\n    persist()\n    # '''}\"\"\"\n",
        "def save():\n    value = f'''{\"\"\" \" } '''#\n    persist()\n    # \"\"\"}'''\n",
        "def save():\n    value = t\"\"\"{''' ' } \"\"\"#\n    persist()\n    # '''}\"\"\"\n",
    ] {
        let err = try_convert(src, SourceLang::Python, "python", "save.py").unwrap_err();
        assert!(
            err.contains("triple-quoted") && err.contains("line 2"),
            "{src:?}: {err}"
        );
    }
}

#[test]
fn go_block_comment_holding_a_newline_ends_a_statement() {
    for src in [
        "package example\nimport _ \"fmt\" /*\n*/ func save() { println(7) }\n",
        "package example\nimport (_ \"fmt\") /*\n*/ func save() { println(7) }\n",
    ] {
        let (garnet, _) = convert_src(src, SourceLang::Go, "go", "save.go");
        assert_parses(&garnet);
        let code = code_lines(&garnet);
        let at = code
            .iter()
            .position(|l| l.starts_with("fn save("))
            .unwrap_or_else(|| {
                panic!("save is converted, not skipped with the import: {src:?}\n{garnet}")
            });
        assert_eq!(
            code[at + 1],
            "println(7)",
            "the body stays inside save: {src:?}\n{garnet}"
        );
    }
}

#[test]
fn eval_stands_for_its_statement_only_as_a_single_call() {
    for (src, lang, name, file, kept) in [
        (
            "eval(\"1\") and def save() persist() end\n",
            SourceLang::Ruby,
            "ruby",
            "save.rb",
            "def save",
        ),
        (
            "eval(\"1\") { def save; persist(); end }\n",
            SourceLang::Ruby,
            "ruby",
            "save.rb",
            "def save",
        ),
        (
            "method_missing(def save() persist() end)\n",
            SourceLang::Ruby,
            "ruby",
            "save.rb",
            "def save",
        ),
        (
            "eval(\"1\") or persist()\n",
            SourceLang::Python,
            "python",
            "save.py",
            "or persist()",
        ),
        (
            "exec(\n\"1\"\n); value = persist()\n",
            SourceLang::Python,
            "python",
            "save.py",
            "value = persist()",
        ),
    ] {
        let (garnet, checklist) = convert_src(src, lang, name, file);
        assert_parses(&garnet);
        // The rest of the statement is kept as a to-do, never active or dropped.
        assert_inactive(&garnet, "persist()");
        assert!(
            checklist.contains(kept),
            "to-do keeps `{kept}`: {src:?}\n{checklist}"
        );
    }
}

// Codex lane B, round 9 on #607 (3b6d1254).

#[test]
fn ruby_special_forms_never_swallow_a_following_statement() {
    for line in [
        "puts 1; def save; persist(); end",
        "print 1; def save; persist(); end",
        "yield; def save; persist(); end",
        "yielding(def save; persist(); end)",
    ] {
        let src = format!("def outer\n  {line}\nend\n");
        let (garnet, checklist) = convert_src(&src, SourceLang::Ruby, "ruby", "outer.rb");
        assert_parses(&garnet);
        assert_inactive(&garnet, "persist()");
        assert!(
            checklist.contains("def save"),
            "kept as a to-do: {line}\n{checklist}"
        );
    }
}

#[test]
fn python_fstring_inside_a_replacement_field_is_refused() {
    let src =
        "def save():\n    value = f\"\"\"{f'{''' } \"\"\"#\n    persist()\n    # '''}'}\"\"\"\n";
    let err = try_convert(src, SourceLang::Python, "python", "save.py").unwrap_err();
    assert!(err.contains("f-string") && err.contains("line 2"), "{err}");
}

// Codex lane B, round 10 on #607 (d2f0c3e0).

#[test]
fn ruby_definition_inside_another_statement_is_never_lost() {
    for src in [
        "def outer\n  puts def save() persist() end\nend\n",
        "def outer\n  print def save() persist() end\nend\n",
        "def outer(x = def save; persist; end)\n  0\nend\n",
        "def outer(x =\n  def save; persist; end)\n  0\nend\n",
        "def outer\n  return def save() persist() end\nend\n",
    ] {
        let (garnet, checklist) = convert_src(src, SourceLang::Ruby, "ruby", "outer.rb");
        assert_parses(&garnet);
        assert_inactive(&garnet, "persist");
        assert!(
            checklist.contains("def save"),
            "kept as a to-do: {src:?}\n{checklist}"
        );
    }
}

// Codex lane B, round 11 on #607 (a366caed).

#[test]
fn ruby_definition_header_or_chain_across_lines_is_kept_whole() {
    for src in [
        "def\n  outer(x = def save; persist; end)\n  0\nend\n",
        "def save\n  persist()\nend\n  .to_s\n",
        "def save\n  persist()\nend\n  &.to_s\n",
        "class Store\n  def m\n    persist()\n  end\nend\n  .freeze\n",
        "module Tools\n  def m\n    persist()\n  end\nend\n  .freeze\n",
    ] {
        let (garnet, checklist) = convert_src(src, SourceLang::Ruby, "ruby", "save.rb");
        assert_parses(&garnet);
        assert_inactive(&garnet, "persist");
        assert!(
            checklist.contains("persist"),
            "kept as a to-do: {src:?}\n{checklist}"
        );
    }
}

// From probing Codex's round-11 corpus: a Ruby parameter list read character by
// character could take a `)` in a comment or string for its end, and a carriage
// return in a kept note relied on Garnet's lexer to stay inside a comment.

#[test]
fn ruby_parameter_list_with_a_comment_or_string_is_kept_whole() {
    for src in [
        "def save(value = 1 # )\n)\n  persist()\nend\n",
        "def save(value = \") #\")\n  persist()\nend\n",
    ] {
        let (garnet, checklist) = convert_src(src, SourceLang::Ruby, "ruby", "save.rb");
        assert_parses(&garnet);
        assert_inactive(&garnet, "persist()");
        assert!(
            checklist.contains("def save"),
            "kept as a to-do: {src:?}\n{checklist}"
        );
    }
}

#[test]
fn every_line_of_a_kept_note_is_a_comment_whatever_its_line_ends() {
    let (garnet, _) = convert_src(
        "def\rsave(x = def inner; persist; end)\nend\n",
        SourceLang::Ruby,
        "ruby",
        "save.rb",
    );
    assert_parses(&garnet);
    for line in garnet.split(['\n', '\r']) {
        let code = line.trim();
        assert!(
            code.is_empty() || code.starts_with('#') || !code.contains("persist"),
            "a note line escaped its comment: {line:?}\n{garnet}"
        );
    }
}

// Codex lane B, round 12 on #607 (7542330d): parameter lists were read by
// characters, so a string, regex or form feed in a signature could turn text into
// parameters, functions or a dropped definition. Only a plain parameter list is
// converted; otherwise the whole definition is kept.

#[test]
fn a_parameter_list_that_is_not_plain_keeps_its_definition_whole() {
    for (src, lang, name, file) in [
        (
            "def save(value=\"x, hidden()):\\ndef other(y\"):\n    keep()\n",
            SourceLang::Python,
            "python",
            "save.py",
        ),
        (
            "def save(value=(1, 2)):\n    keep()\n",
            SourceLang::Python,
            "python",
            "save.py",
        ),
        (
            "def save(value=\"x, x) { hidden() } def other(y=1\"):\n    keep()\n",
            SourceLang::Python,
            "python",
            "save.py",
        ),
        (
            "def save(value = /x, hidden=1/)\n  keep()\nend\n",
            SourceLang::Ruby,
            "ruby",
            "save.rb",
        ),
    ] {
        let (garnet, checklist) = convert_src(src, lang, name, file);
        assert_parses(&garnet);
        assert_inactive(&garnet, "keep()");
        assert_inactive(&garnet, "persist");
        assert_inactive(&garnet, "hidden");
        assert_inactive(&garnet, "other");
        assert!(
            checklist.contains("def "),
            "kept as a to-do: {src:?}\n{checklist}"
        );
    }
    // Plain parameter lists still convert.
    for (src, lang, name, file, header) in [
        (
            "def save(a: int, b=1, *, c=2):\n    return a\n",
            SourceLang::Python,
            "python",
            "save.py",
            "def save(a: Int, b, c) {",
        ),
        (
            "def save(a, b = 1)\n  a\nend\n",
            SourceLang::Ruby,
            "ruby",
            "save.rb",
            "def save(a, b) {",
        ),
    ] {
        let (garnet, _) = convert_src(src, lang, name, file);
        assert_parses(&garnet);
        assert!(code_lines(&garnet).contains(&header), "{src:?}\n{garnet}");
    }
}

/// Codex round 13: a comma inside a parameter default (a Python lambda's own
/// parameters, a Ruby array) does not start another parameter, and a default
/// holding `=` (an assignment inside it) keeps the definition whole.
#[test]
fn a_comma_inside_a_default_does_not_add_a_parameter() {
    for (src, lang, name, file, header) in [
        (
            "def save(value=lambda left, right: left):\n    keep()\n",
            SourceLang::Python,
            "python",
            "save.py",
            None,
        ),
        (
            "def save(value=[x := 1, 2]):\n    keep()\n",
            SourceLang::Python,
            "python",
            "save.py",
            None,
        ),
        (
            "def save(value = [1, hidden = 2])\n  keep()\nend\n",
            SourceLang::Ruby,
            "ruby",
            "save.rb",
            None,
        ),
        (
            "def save(value=[1, 2], count: int = 3):\n    keep()\n",
            SourceLang::Python,
            "python",
            "save.py",
            Some("def save(value, count: Int) {"),
        ),
        (
            "def save(value = [1, 2], count = 3)\n  keep()\nend\n",
            SourceLang::Ruby,
            "ruby",
            "save.rb",
            Some("def save(value, count) {"),
        ),
    ] {
        let (garnet, checklist) = convert_src(src, lang, name, file);
        assert_parses(&garnet);
        let headers: Vec<&str> = code_lines(&garnet)
            .into_iter()
            .filter(|l| l.starts_with("def save("))
            .collect();
        match header {
            Some(expected) => assert_eq!(headers, [expected], "{src:?}\n{garnet}"),
            None => {
                assert!(headers.is_empty(), "{src:?}\n{garnet}");
                assert_inactive(&garnet, "keep()");
                assert!(
                    checklist.contains("def "),
                    "kept as a to-do: {src:?}\n{checklist}"
                );
            }
        }
    }
}

/// Codex round 14: a converted function has exactly the source's named
/// parameters. `self` stays a parameter (Garnet takes it explicitly), a Go
/// receiver becomes the first parameter (Go's method expression `T.M`), and a
/// list Garnet cannot write (Python `*args`/`**kwargs`; a Go variadic, an
/// unnamed parameter list or an unnamed receiver) keeps the definition whole.
#[test]
fn a_converted_function_keeps_every_named_parameter() {
    let py = |src: &'static str| (src, SourceLang::Python, "python", "save.py");
    let go = |src: &'static str| (src, SourceLang::Go, "go", "save.go");
    for ((src, lang, name, file), header) in [
        (
            py("def save(self: int):\n    return 1\n"),
            Some("def save(self: Int) {"),
        ),
        (
            py("def save(self=1):\n    return 1\n"),
            Some("def save(self) {"),
        ),
        (py("def save(*items):\n    return items\n"), None),
        (py("def save(**items):\n    return items\n"), None),
        (
            py("def save(a: int, b=1, *, c=2):\n    return a\n"),
            Some("def save(a: Int, b, c) {"),
        ),
        (
            go("package main\n\nfunc (s *S) save(x int) int {\n\treturn x\n}\n"),
            Some("fn save(s: Box<S>, x: Int) -> Int {"),
        ),
        (
            go("package main\n\nfunc (S) save(x int) int {\n\treturn x\n}\n"),
            None,
        ),
        (
            go("package main\n\nfunc save(xs ...int) int {\n\treturn 0\n}\n"),
            None,
        ),
        (
            go("package main\n\nfunc save(int, string) int {\n\treturn 0\n}\n"),
            None,
        ),
        (
            go("package main\n\nfunc save(chan int) int {\n\treturn 0\n}\n"),
            None,
        ),
        (
            go("package main\n\nfunc save(a, b int) int {\n\treturn a\n}\n"),
            Some("fn save(a, b: Int) -> Int {"),
        ),
    ] {
        let (garnet, checklist) = convert_src(src, lang, name, file);
        assert_parses(&garnet);
        let headers: Vec<&str> = code_lines(&garnet)
            .into_iter()
            .filter(|l| l.starts_with("def save(") || l.starts_with("fn save("))
            .collect();
        match header {
            Some(expected) => assert_eq!(headers, [expected], "{src:?}\n{garnet}"),
            None => {
                assert!(headers.is_empty(), "{src:?}\n{garnet}");
                assert_inactive(&garnet, "return");
                assert!(
                    checklist.contains("save"),
                    "kept as a to-do: {src:?}\n{checklist}"
                );
            }
        }
    }
}

/// Codex round 15: Ruby code is read as ASCII. A non-ASCII character outside a
/// string or comment (a heredoc delimiter such as `<<終`, an identifier, a
/// symbol) refuses the file, so no ASCII-only check can disagree with the lexer;
/// a string or comment may still hold any character.
#[test]
fn ruby_code_outside_strings_and_comments_is_ascii() {
    for src in [
        "text = <<終\ndef save\n  persist()\nend\n終\n",
        "def save\n  text = <<終\n  persist()\n終\n  return 1\nend\n",
        "def save\n  text = <<-終\n  persist()\n  終\n  return 1\nend\n",
        "def 保存\n  persist()\nend\n",
        "def save\n  x = :終\n  persist()\nend\n",
        "def save\n  puts \"#{終}\"\nend\n",
    ] {
        match try_convert(src, SourceLang::Ruby, "ruby", "save.rb") {
            Err(_) => {}
            Ok(garnet) => panic!("converted: {src:?}\n{garnet}"),
        }
    }
    // A string or comment may hold any character.
    let (garnet, _) = convert_src(
        "# 終わり\ndef save\n  puts \"終\"\n  persist() # 保存\nend\n",
        SourceLang::Ruby,
        "ruby",
        "save.rb",
    );
    assert_parses(&garnet);
    assert!(code_lines(&garnet).contains(&"def save() {"), "{garnet}");
}

/// Codex round 15: a Go parameter written as a spaced type (`Box [int]`, an
/// instantiated generic; `pkg .Type`, a qualified name) is an unnamed
/// parameter, as go/parser reads it, not a name `Box` or `pkg`: the function is
/// kept whole. A name before an array or slice type is still a name.
#[test]
fn a_spaced_go_type_is_not_a_parameter_name() {
    for src in [
        "package main\n\nfunc keep(Box [int]) {\n\treturn\n}\n",
        "package main\n\ntype Box[T any] struct{}\n\nfunc (Box [T]) keep() {\n\treturn\n}\n",
        "package main\n\nfunc keep(pkg .Type) {\n\treturn\n}\n",
        "package main\n\nfunc keep(a int, Box [int]) {\n\treturn\n}\n",
    ] {
        let (garnet, checklist) = convert_src(src, SourceLang::Go, "go", "keep.go");
        assert_parses(&garnet);
        assert!(
            !code_lines(&garnet)
                .iter()
                .any(|l| l.starts_with("fn keep(")),
            "{src:?}\n{garnet}"
        );
        assert_inactive(&garnet, "return");
        assert!(
            checklist.contains("keep"),
            "kept as a to-do: {src:?}\n{checklist}"
        );
    }
    // A name before an array or slice type is a name.
    let src = "package main\n\nfunc keep(xs [3]int, ys []string) {\n\treturn\n}\n";
    let (garnet, _) = convert_src(src, SourceLang::Go, "go", "keep.go");
    assert_parses(&garnet);
    assert!(
        code_lines(&garnet).contains(&"fn keep(xs: Array<Int>, ys: Array<String>) -> () {"),
        "{src:?}\n{garnet}"
    );
}

/// Codex round 16: a Ruby statement in a body stays inside its block. One that
/// closes a block opened before it or starts a clause of that block after a
/// `;` (`0; end; def second`, `nil; ensure`) keeps the enclosing definition
/// whole. A number is not a method name: in `return 1?` the `?` is the ternary
/// operator, which continues the statement on the next line.
#[test]
fn a_ruby_statement_stays_inside_its_block() {
    for src in [
        "def first\n  0; end; def second\n  persist()\nend\n",
        "def save\n  nil; ensure\n    persist()\nend\n",
        "def save\n  nil; rescue\n    persist()\nend\n",
        "class Box\n  0; end; class Other\n  persist()\nend\n",
        "def save\n  return 1?\n    persist() : 0\nend\n",
        "def save\n  x = 2.5?\n    persist() : 0\nend\n",
    ] {
        let (garnet, _) = convert_src(src, SourceLang::Ruby, "ruby", "save.rb");
        assert_parses(&garnet);
        assert_inactive(&garnet, "persist()");
        assert!(
            !code_lines(&garnet).iter().any(|l| l.contains('?')),
            "{src:?}\n{garnet}"
        );
    }
    // A block closed on its own line, a `rescue` modifier and a predicate
    // method name are still read.
    for (src, active) in [
        (
            "def save\n  begin; x; rescue; y; end\n  persist()\nend\n",
            "persist()",
        ),
        (
            "def save\n  x = foo rescue nil\n  persist()\nend\n",
            "persist()",
        ),
        ("def save\n  return x.empty?\nend\n", "return x.empty?"),
    ] {
        let (garnet, _) = convert_src(src, SourceLang::Ruby, "ruby", "save.rb");
        assert_parses(&garnet);
        let code = code_lines(&garnet);
        assert!(code.contains(&"def save() {"), "{src:?}\n{garnet}");
        assert!(code.contains(&active), "{src:?}\n{garnet}");
    }
}

/// Codex round 17: the rule holds for a nested definition too (one whose body
/// closes the enclosing block as well keeps that block whole), and a trailing
/// `in` (a pattern match) takes the next line, like any operator.
#[test]
fn a_nested_ruby_definition_and_a_trailing_in_stay_contained() {
    for src in [
        "def outer\n  def inner\n    0; end; end; def second\n  persist()\nend\n",
        "class Box\n  def inner\n    0; end; end; def second\n  persist()\nend\n",
        "def save\n  x in\n    Persisted()\nend\n",
        "def save\n  x in # a pattern\n\n    # on the next code line\n    Persisted()\nend\n",
    ] {
        let (garnet, _) = convert_src(src, SourceLang::Ruby, "ruby", "save.rb");
        assert_parses(&garnet);
        assert_inactive(&garnet, "persist()");
        assert_inactive(&garnet, "Persisted()");
    }
    // A nested definition closed on its own line keeps the enclosing one read,
    // and the statement after it active (Garnet has no nested `def`, so the
    // emitter keeps `inner` itself as a to-do).
    let (garnet, _) = convert_src(
        "def outer\n  def inner\n    0\n  end\n  persist()\nend\n",
        SourceLang::Ruby,
        "ruby",
        "save.rb",
    );
    assert_parses(&garnet);
    let code = code_lines(&garnet);
    assert!(code.contains(&"def outer() {"), "{garnet}");
    assert!(code.contains(&"persist()"), "{garnet}");
}

/// Codex round 18: a `do` is a loop's separator only while the loop's
/// condition is open: a `;` at the loop's bracket depth ends the condition, so
/// a later `do` (`while false; work do`) opens a block and its `end` closes
/// that block, not the loop.
#[test]
fn a_do_after_a_loop_condition_opens_a_block() {
    for src in [
        "def save\n  while false; work do\n  end\n  persist()\n  end\nend\n",
        "def save\n  until true; work do\n  end\n  persist()\n  end\nend\n",
        "def save\n  for x in [1]; work do\n  end\n  persist()\n  end\nend\n",
        "def save\n  while false; end; work do\n    persist()\n  end\nend\n",
        // Controls: a `;` inside the condition's parentheses, and the loop's
        // own `do` before a block's.
        "def save\n  while (a; b) do\n    persist()\n  end\nend\n",
        "def save\n  while false do work do\n  end\n  persist()\n  end\nend\n",
    ] {
        let (garnet, _) = convert_src(src, SourceLang::Ruby, "ruby", "save.rb");
        assert_parses(&garnet);
        assert_inactive(&garnet, "persist()");
    }
}

/// Codex round 19: after `.`, `&.`, `::` or `def` the method name is the next
/// token of any kind, an operator name (`obj.[]`, `obj.!`, `obj.-@`) or the
/// `obj.()` call included; a word after it (`do`) is a keyword again and opens
/// a block.
#[test]
fn an_operator_method_name_does_not_hide_a_block() {
    for call in [
        "obj.[]",
        "obj.[]()",
        "obj.!",
        "obj.~",
        "obj.-@",
        "obj.+@",
        "obj.\n    []",
        "obj&.[]",
        "obj::[]",
        "obj.()",
        // Controls: a word name and an operator name with an argument.
        "obj.call",
        "obj.[](0)",
    ] {
        let src = format!("def save\n  {call} do\n    persist()\n  end\nend\n");
        let (garnet, _) = convert_src(&src, SourceLang::Ruby, "ruby", "save.rb");
        assert_parses(&garnet);
        assert_inactive(&garnet, "persist()");
    }
}

/// Codex round 20: Ruby reads a form feed or vertical tab as a blank and stops
/// at `^D`, `^Z` or NUL, where the converter did neither. Ruby code outside a
/// string or comment may hold no control character but tab and carriage
/// return: the file is refused.
#[test]
fn a_control_character_in_ruby_code_refuses_the_file() {
    let mut sources = Vec::new();
    for gap in ['\x0c', '\x0b'] {
        for prefix in ["obj.", "obj&.", "obj::"] {
            sources.push(format!(
                "def save\n  while false\n    {prefix}{gap}end\n    persist()\n  end\nend\n"
            ));
        }
        sources.push(format!(
            "def outer\n  def{gap}end\n    persist()\n  end\nend\n"
        ));
    }
    for stop in ['\x04', '\x1a', '\0'] {
        sources.push(format!("def save\n  keep()\nend\n{stop}\npersist()\n"));
    }
    // A header split by a form feed or vertical tab (round 12, kept whole then).
    for gap in ['\x0c', '\x0b'] {
        sources.push(format!(
            "def{gap}\nouter(x = def save; persist; end)\nend\n"
        ));
    }
    for src in &sources {
        match try_convert(src, SourceLang::Ruby, "ruby", "save.rb") {
            Err(e) => assert!(e.contains("control character"), "{src:?}: {e}"),
            Ok(garnet) => panic!("converted: {src:?}\n{garnet}"),
        }
    }
    // A control character inside a string or comment is text.
    let (garnet, _) = convert_src(
        "def save\n  puts \"a\\fb\x0c\"\n  persist() # \x0b\nend\n",
        SourceLang::Ruby,
        "ruby",
        "save.rb",
    );
    assert_parses(&garnet);
    assert!(
        code_lines(&garnet)
            .iter()
            .any(|l| l.starts_with("persist()")),
        "{garnet}"
    );
}

/// Codex round 21 (from the draft of a run stopped by a content filter): Ruby
/// reads a carriage return inside a line as a blank, where some lexer checks
/// counted only spaces and tabs. `value\r/ work do /1/` was read as a regular
/// expression hiding the block's `do`, and `def foo()\r= x` was not seen as an
/// endless method. Every Ruby blank check now uses one set: space, tab and
/// carriage return.
#[test]
fn a_carriage_return_is_a_ruby_blank_everywhere() {
    // Each source with a carriage return converts as the same source with a
    // space does, and neither leaves the block's body active.
    let outcome = |src: &str| match try_convert(src, SourceLang::Ruby, "ruby", "save.rb") {
        Err(e) => {
            assert!(
                e.contains("ambiguous /") || e.contains("endless method"),
                "{src:?}: {e}"
            );
            None
        }
        Ok(garnet) => {
            assert_parses(&garnet);
            assert_inactive(&garnet, "persist()");
            Some(code_lines(&garnet).len())
        }
    };
    for (cr, space) in [
        (
            "def save\n  value\r/ work do /1/; 0\n    persist()\n  end\nend\n",
            "def save\n  value / work do /1/; 0\n    persist()\n  end\nend\n",
        ),
        (
            "def save\n  value\r\t/ work do /1/; 0\n    persist()\n  end\nend\n",
            "def save\n  value \t/ work do /1/; 0\n    persist()\n  end\nend\n",
        ),
        (
            "def save\n  def foo()\r= keep()\n  persist()\nend\n",
            "def save\n  def foo() = keep()\n  persist()\nend\n",
        ),
    ] {
        assert_eq!(outcome(cr), outcome(space), "{cr:?} / {space:?}");
    }
}

/// Codex round 22: whether a `/` or `%` divides or opens a literal was decided
/// by scanning bytes backward, which disagreed with the lexer's own record of
/// the token before: after `ok?`, `ok!`, `$!` or a keyword-shaped method name
/// (`x.if`) the `/` divides, but the scan read a regular expression that hid
/// the block's `do`. Both now come from the token before, as the lexer read it.
#[test]
fn a_division_after_a_value_does_not_hide_a_block() {
    for head in [
        "ok? / work do 1/2",
        "ok! / work do 1/2",
        "$! / work do 1/2",
        "x.if / work do 1/2",
        "x.end / work do 1/2",
        // Controls: a word, an instance variable, a number, and `%`.
        "value / work do 1/2",
        "@v / work do 1/2",
        "4 / work do 1/2",
        "ok? % work do 1%2",
    ] {
        let src = format!("def save\n  {head}\n    persist()\n  end\nend\n");
        let (garnet, _) = convert_src(&src, SourceLang::Ruby, "ruby", "save.rb");
        assert_parses(&garnet);
        assert_inactive(&garnet, "persist()");
    }
    // After `return` an operand starts: `/ 2 /` is a regular expression that
    // ends its line, so the next line is a statement of its own.
    let (garnet, _) = convert_src(
        "def save\n  return / 2 /\n  persist()\nend\n",
        SourceLang::Ruby,
        "ruby",
        "save.rb",
    );
    assert_parses(&garnet);
    assert!(code_lines(&garnet).contains(&"persist()"), "{garnet}");
}

/// Codex round 23: an operator method name (`obj.!`, `obj&.~`, `obj::!`,
/// `obj.==`) is one name token and leaves a value, so a `/` after it divides;
/// and a `\` that ends a line is a blank, as Ruby reads it, so the next line
/// starts in the state the line before ended in (`value \` then `/ work`).
#[test]
fn an_operator_name_and_a_backslash_keep_the_lexer_state() {
    let mut sources = Vec::new();
    for name in [
        "obj.!", "obj.~", "obj&.!", "obj&.~", "obj::!", "obj::~", "obj.==", "obj.<=>",
        // Controls.
        "obj.[]", "obj.+@", "obj.!()",
    ] {
        sources.push(format!(
            "def save\n  {name} / work do 1/2\n    persist()\n  end\nend\n"
        ));
    }
    for value in ["value", "4", "ok?", "value\t"] {
        sources.push(format!(
            "def save\n  {value} \\\n  / work do 1/2\n    persist()\n  end\nend\n"
        ));
    }
    // A loop condition continues across `\`: its `do` there is the separator.
    sources.push("def save\n  while ready \\\n  do\n    persist()\n  end\nend\n".into());
    // An operator method defined by name is kept whole.
    sources.push("class Box\n  def ==(other)\n    persist()\n  end\nend\n".into());
    for src in &sources {
        let (garnet, _) = convert_src(src, SourceLang::Ruby, "ruby", "save.rb");
        assert_parses(&garnet);
        assert_inactive(&garnet, "persist()");
    }
    // An operator symbol (`:/`, `:<=>`) is one token: its `/` opens no regular
    // expression, so the file converts and the statement after it is read.
    for src in [
        "def save\n  list.map(&:/)\n  persist()\nend\n",
        "def save\n  send(:<=>, a)\n  persist()\nend\n",
        "def save\n  y = [:/, 1]\n  persist()\nend\n",
    ] {
        let (garnet, _) = convert_src(src, SourceLang::Ruby, "ruby", "save.rb");
        assert_parses(&garnet);
        assert!(
            code_lines(&garnet).contains(&"persist()"),
            "{src:?}\n{garnet}"
        );
    }
    convert_src(
        "class Tms\n  def /(x); memberwise(:/, x) end\nend\n",
        SourceLang::Ruby,
        "ruby",
        "tms.rb",
    );
    // `value \` then `/x/` is `value /x/`, which Ruby reads ambiguously.
    let err = try_convert(
        "def save\n  value \\\n  /x/\nend\n",
        SourceLang::Ruby,
        "ruby",
        "save.rb",
    )
    .unwrap_err();
    assert!(err.contains("ambiguous /"), "{err}");
}

/// Codex round 24: after an identifier and a blank (a method call's first
/// argument, as Ruby reads it) `:` with an operator name is a symbol (`use :/`)
/// and `?` with a character is a character literal (`use ?/`), so the `/` opens
/// no regular expression that could hide the block's `do`.
#[test]
fn an_operator_symbol_argument_does_not_hide_a_block() {
    for head in [
        "use :/",
        "obj.use :/",
        "obj&.use :/",
        "obj::use :/",
        "use \\\n  :/",
        "use :%",
        "use ?/",
        // Controls.
        "use(:/)",
        "use [:/]",
        "use :slash",
        "use :<=>",
    ] {
        let src = format!("def save\n  {head} do 1/2\n    persist()\n  end\nend\n");
        match try_convert(&src, SourceLang::Ruby, "ruby", "save.rb") {
            Err(e) => assert!(
                e.contains("character literal") || e.contains("ternary colon"),
                "{src:?}: {e}"
            ),
            Ok(garnet) => {
                assert_parses(&garnet);
                assert_inactive(&garnet, "persist()");
            }
        }
    }
}

/// Codex round 25 (from the draft of a run stopped by a content filter): after
/// an identifier and a blank, Ruby reads `:` as a symbol after a method name
/// but as a ternary colon after a local variable, which the converter cannot
/// tell apart. Where the two readings differ in structure (an operator that
/// starts a literal, `:/` `:%` `` :` ``, or a keyword that opens a block, `:if`)
/// the file is refused; after a value the `:` is a ternary colon. `?` followed
/// by `\` is a character literal (`?\C-/`), refused.
#[test]
fn an_ambiguous_ruby_colon_is_refused() {
    let in_block =
        |line: &str| format!("def save\n  use do |local|\n    {line}\n    persist()\n  end\nend\n");
    let mut refused = vec![
        in_block("x = (true ? local :/end/)"),
        in_block("(true ? local :%r{end})"),
        in_block("(true ? local :%q{end})"),
        in_block("(true ? local\r:/end/)"),
        in_block("x = (true ? local :if true then 1 end)"),
    ];
    for head in ["use ?\\C-/", "use ?\\M-/", "x = ?\\C-/ ; work"] {
        refused.push(format!(
            "def save\n  {head} do 1/2\n    persist()\n  end\nend\n"
        ));
    }
    for src in &refused {
        match try_convert(src, SourceLang::Ruby, "ruby", "save.rb") {
            Err(e) => assert!(
                e.contains("ternary colon") || e.contains("character literal"),
                "{src:?}: {e}"
            ),
            Ok(garnet) => panic!("converted: {src:?}\n{garnet}"),
        }
    }
    // Not ambiguous: after a value, or with a blank after `:`, it is a ternary
    // colon; a symbol that no ternary reading could change is read as one.
    for src in [
        in_block("(true ? (local) :/end/)"),
        in_block("(true ? local : /end/)"),
    ] {
        let (garnet, _) = convert_src(&src, SourceLang::Ruby, "ruby", "save.rb");
        assert_parses(&garnet);
        assert_inactive(&garnet, "persist()");
    }
    for src in [
        "class Box\n  alias_method :<<, :push\nend\n",
        "class Box\n  attr_accessor :next\nend\n",
    ] {
        try_convert(src, SourceLang::Ruby, "ruby", "box.rb").unwrap();
    }
}

/// Codex round 26: listing which `:` forms read differently as a symbol and as
/// a ternary colon missed operators and keywords that take an operand
/// (`local :! /end/`, `local :defined? /end/`). After an identifier and a blank
/// the lexer now reads the rest of the line both ways and refuses the file when
/// the two readings differ in structure.
#[test]
fn a_colon_read_two_ways_must_agree() {
    let in_block =
        |line: &str| format!("def save\n  use do |local|\n    {line}\n    persist()\n  end\nend\n");
    for line in [
        "(true ? local :! /end/)",
        "(true ? local :defined? /end/)",
        "(true ? local :~ /end/)",
        "(true ? local :- /end/)",
    ] {
        let src = in_block(line);
        match try_convert(&src, SourceLang::Ruby, "ruby", "save.rb") {
            Err(e) => assert!(e.contains("ternary colon"), "{src:?}: {e}"),
            Ok(garnet) => panic!("converted: {src:?}\n{garnet}"),
        }
    }
    // Where both readings agree, or a ternary cannot follow, it converts.
    for line in [
        "(true ? local :foo)",
        "alias_method :<<, :push",
        "attr_accessor :next",
        "delete :in",
        "send :foo, 1",
        // A binary operator cannot start an expression after a ternary colon.
        "undef_method :<<",
        "alias_method :==, :eql?",
    ] {
        let (garnet, _) = convert_src(&in_block(line), SourceLang::Ruby, "ruby", "save.rb");
        assert_parses(&garnet);
        assert_inactive(&garnet, "persist()");
    }
}

/// Codex round 27 (a scope note it did not count as a finding): when the `:`
/// fork, or a `\` continuation, starts inside the parentheses of a loop
/// condition, the loop's `do` was counted as a block opener, and the statement
/// after the enclosing block (`after_loop()`) was converted inside its loop. The
/// state handed on keeps the condition's bracket depth relative to where the
/// next reading starts.
#[test]
fn a_loop_condition_survives_a_fork_inside_its_parentheses() {
    for src in [
        "def save(local)\n  items.each do |item|\n    while (true ? local :foo) do\n      persist()\n    end\n  end\n  after_loop()\nend\n",
        "def save(local)\n  items.each do |item|\n    while (ready \\\n      ) do\n      persist()\n    end\n  end\n  after_loop()\nend\n",
        // Control: the spaced colon, read as a ternary colon.
        "def save(local)\n  items.each do |item|\n    while (true ? local : foo) do\n      persist()\n    end\n  end\n  after_loop()\nend\n",
    ] {
        let (garnet, _) = convert_src(src, SourceLang::Ruby, "ruby", "save.rb");
        assert_parses(&garnet);
        // `after_loop()` follows the `for` loop's closing brace, in `save`.
        let code = code_lines(&garnet);
        let at = code.iter().position(|l| *l == "after_loop()");
        assert!(
            at.is_some_and(|at| at > 0 && code[at - 1] == "}"),
            "{src:?}\n{garnet}"
        );
    }
}
