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
        assert!(
            code_lines(&garnet)
                .iter()
                .any(|l| l.starts_with("fn save(")),
            "save keeps its own body: {src:?}\n{garnet}"
        );
        assert!(
            !code_lines(&garnet)
                .iter()
                .any(|l| l.starts_with("fn external(")),
            "external has no body: {src:?}\n{garnet}"
        );
    }
}
