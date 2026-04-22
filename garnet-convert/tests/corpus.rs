//! End-to-end conversion tests — source → Garnet round-trip per language.
//!
//! Each test drives the full pipeline: parse → lift → idioms → witness
//! → emit. Assertions verify: the output is marked `@sandbox`, declares
//! `@caps()` (empty until human audit), and contains the expected
//! Garnet constructs for the source.

use garnet_convert::{convert, EmitOpts, SourceLang};

fn opts(source_lang: &str, source_file: &str) -> EmitOpts {
    EmitOpts {
        source_lang: source_lang.into(),
        source_file: source_file.into(),
        target_file: source_file.replace(['.'], "_") + ".garnet",
        source_loc: 0,
        strict: false,
        fail_on_todo: false,
        fail_on_untranslatable: false,
    }
}

// ─── Rust → Garnet ───────────────────────────────────────────────────

#[test]
fn rust_simple_fn_roundtrips() {
    let src = "fn hash(data: &[u8]) -> u64 { return 0; }";
    let (out, metrics) = convert(src, SourceLang::Rust, "hash.rs", opts("rust", "hash.rs")).unwrap();
    assert!(out.garnet.contains("@sandbox"));
    assert!(out.garnet.contains("@caps()"));
    assert!(out.garnet.contains("fn hash("));
    assert!(metrics.sandbox_status == garnet_convert::metrics::SandboxStatus::Quarantined);
}

#[test]
fn rust_struct_roundtrips() {
    let src = "struct User { pub id: i64, name: String, }";
    let (out, _) = convert(src, SourceLang::Rust, "user.rs", opts("rust", "user.rs")).unwrap();
    assert!(out.garnet.contains("struct User {"));
    assert!(out.garnet.contains("pub id: Int"));
    assert!(out.garnet.contains("name: String"));
}

#[test]
fn rust_option_type_preserved() {
    let src = "fn f(x: Option<i32>) -> i32 { return 0; }";
    let (out, _) = convert(src, SourceLang::Rust, "f.rs", opts("rust", "f.rs")).unwrap();
    assert!(out.garnet.contains("Option<Int>"));
}

#[test]
fn rust_result_type_preserved() {
    let src = "fn f() -> Result<String, IoError> { return nil; }";
    let (out, _) = convert(src, SourceLang::Rust, "f.rs", opts("rust", "f.rs")).unwrap();
    assert!(out.garnet.contains("Result<String, IoError>"));
}

#[test]
fn rust_unsafe_flagged_in_output() {
    let src = "unsafe { do_thing(); }";
    let (out, metrics) = convert(src, SourceLang::Rust, "u.rs", opts("rust", "u.rs")).unwrap();
    assert!(out.garnet.contains("@untranslatable"));
    assert!(metrics.untranslatable_count > 0);
}

// ─── Ruby → Garnet ───────────────────────────────────────────────────

#[test]
fn ruby_def_roundtrips() {
    let src = "def greet(name)\n  return name\nend\n";
    let (out, _) = convert(src, SourceLang::Ruby, "g.rb", opts("ruby", "g.rb")).unwrap();
    assert!(out.garnet.contains("def greet"));
    assert!(out.garnet.contains("@sandbox"));
}

#[test]
fn ruby_method_missing_flagged_todo() {
    let src = "method_missing(name, *args) { do_it }\n";
    let (out, metrics) = convert(src, SourceLang::Ruby, "d.rb", opts("ruby", "d.rb")).unwrap();
    assert!(out.garnet.contains("@migrate_todo"));
    assert!(out.migrate_todo_md.contains("method_missing"));
    assert!(metrics.migrate_todo_count > 0);
}

#[test]
fn ruby_eval_rejected() {
    let src = "eval(\"1 + 1\")\n";
    let (out, metrics) = convert(src, SourceLang::Ruby, "e.rb", opts("ruby", "e.rb")).unwrap();
    assert!(out.garnet.contains("@untranslatable"));
    assert!(metrics.untranslatable_count > 0);
}

#[test]
fn ruby_class_becomes_struct_plus_impl() {
    let src = "class User\n  def greet\n    return name\n  end\nend\n";
    let (out, _) = convert(src, SourceLang::Ruby, "u.rb", opts("ruby", "u.rb")).unwrap();
    assert!(out.garnet.contains("module User"));
    assert!(out.garnet.contains("struct User"));
    assert!(out.garnet.contains("impl User"));
}

// ─── Python → Garnet ─────────────────────────────────────────────────

#[test]
fn python_typed_def_roundtrips() {
    let src = "def greet(name: str) -> str:\n    return name\n";
    let (out, _) = convert(src, SourceLang::Python, "g.py", opts("python", "g.py")).unwrap();
    assert!(out.garnet.contains("def greet"));
    assert!(out.garnet.contains("-> String"));
}

#[test]
fn python_decorator_flagged_todo() {
    let src = "@cached\ndef f():\n    return 1\n";
    let (out, _) = convert(src, SourceLang::Python, "d.py", opts("python", "d.py")).unwrap();
    assert!(out.garnet.contains("@migrate_todo"));
    assert!(out.migrate_todo_md.contains("decorator"));
}

#[test]
fn python_class_becomes_struct_plus_impl() {
    let src = "class Point:\n    def __init__(self, x, y):\n        self.x = x\n        self.y = y\n    def sum(self):\n        return self.x + self.y\n";
    let (out, _) = convert(src, SourceLang::Python, "p.py", opts("python", "p.py")).unwrap();
    assert!(out.garnet.contains("struct Point"));
    assert!(out.garnet.contains("impl Point"));
    assert!(out.garnet.contains("def sum"));
}

#[test]
fn python_optional_list_preserved() {
    let src = "def f(xs: List[int], y: Optional[str]) -> int:\n    return 0\n";
    let (out, _) = convert(src, SourceLang::Python, "f.py", opts("python", "f.py")).unwrap();
    assert!(out.garnet.contains("Array<Int>"));
    assert!(out.garnet.contains("Option<String>"));
}

// ─── Go → Garnet ─────────────────────────────────────────────────────

#[test]
fn go_func_lifts_to_safe_fn() {
    let src = "package main\nfunc Greet(name string) string { return name }\n";
    let (out, _) = convert(src, SourceLang::Go, "g.go", opts("go", "g.go")).unwrap();
    assert!(out.garnet.contains("fn Greet"));
}

#[test]
fn go_struct_field_visibility_preserved() {
    let src = "package main\ntype User struct {\n  ID int\n  name string\n}\n";
    let (out, _) = convert(src, SourceLang::Go, "u.go", opts("go", "u.go")).unwrap();
    assert!(out.garnet.contains("struct User"));
    assert!(out.garnet.contains("pub ID: Int"));
    assert!(out.garnet.contains("name: String"));
    // Note: lowercase-first-letter Go field is private → no `pub` prefix
    let private_field_line = out
        .garnet
        .lines()
        .find(|l| l.contains("name: String"))
        .unwrap();
    assert!(!private_field_line.contains("pub name"));
}

#[test]
fn go_goroutine_flagged_todo() {
    let src = "package main\nfunc f() {\n  go doWork()\n  return\n}\n";
    let (out, metrics) = convert(src, SourceLang::Go, "g.go", opts("go", "g.go")).unwrap();
    assert!(out.migrate_todo_md.contains("goroutine"));
    assert!(metrics.migrate_todo_count > 0);
}

// ─── Cross-cutting invariants ────────────────────────────────────────

#[test]
fn every_converted_file_starts_with_sandbox() {
    for (src, lang, fname) in [
        ("fn f() { return 0; }", SourceLang::Rust, "a.rs"),
        ("def g\n  return 1\nend", SourceLang::Ruby, "a.rb"),
        ("def h() -> int:\n    return 2\n", SourceLang::Python, "a.py"),
        ("package main\nfunc I() int { return 3 }\n", SourceLang::Go, "a.go"),
    ] {
        let (out, _) = convert(src, lang, fname, opts(lang.as_str(), fname)).unwrap();
        assert!(
            out.garnet.contains("@sandbox"),
            "{lang:?} output missing @sandbox: {}",
            out.garnet
        );
        assert!(
            out.garnet.contains("@caps()"),
            "{lang:?} output missing @caps(): {}",
            out.garnet
        );
    }
}

#[test]
fn strict_mode_rejects_rust_unsafe() {
    let src = "unsafe { do_thing(); }";
    let mut o = opts("rust", "u.rs");
    o.strict = true;
    o.fail_on_untranslatable = true;
    let r = convert(src, SourceLang::Rust, "u.rs", o);
    assert!(r.is_err(), "strict mode should reject Untranslatable");
}

#[test]
fn strict_mode_rejects_ruby_method_missing() {
    let src = "method_missing(name, *args) { do_it }\n";
    let mut o = opts("ruby", "d.rb");
    o.fail_on_todo = true;
    let r = convert(src, SourceLang::Ruby, "d.rb", o);
    assert!(r.is_err(), "fail_on_todo should reject MigrateTodo");
}

#[test]
fn lineage_json_emitted_with_valid_entries() {
    let src = "fn f() { return 0; }";
    let (out, _) = convert(src, SourceLang::Rust, "f.rs", opts("rust", "f.rs")).unwrap();
    assert!(out.lineage_json.contains("\"source_lang\""));
    assert!(out.lineage_json.contains("\"Module\""));
    assert!(out.lineage_json.contains("\"Func\""));
}

#[test]
fn migrate_todo_md_formatted_as_checklist() {
    let src = "method_missing x\n";
    let (out, _) = convert(src, SourceLang::Ruby, "d.rb", opts("ruby", "d.rb")).unwrap();
    assert!(out.migrate_todo_md.contains("- [ ]"));
    assert!(out.migrate_todo_md.contains("@sandbox(unquarantine)"));
}

#[test]
fn source_lang_detection() {
    assert_eq!(SourceLang::from_extension("rs"), Some(SourceLang::Rust));
    assert_eq!(SourceLang::from_extension("rb"), Some(SourceLang::Ruby));
    assert_eq!(SourceLang::from_extension("py"), Some(SourceLang::Python));
    assert_eq!(SourceLang::from_extension("go"), Some(SourceLang::Go));
    assert_eq!(SourceLang::from_extension("js"), None);
    assert_eq!(SourceLang::from_str("rust"), Some(SourceLang::Rust));
}

#[test]
fn witness_hash_differs_across_source_content() {
    let src_a = "fn f() { return 0; }";
    let src_b = "fn g() { return 1; }";
    let (_, m_a) = convert(src_a, SourceLang::Rust, "a.rs", opts("rust", "a.rs")).unwrap();
    let (_, m_b) = convert(src_b, SourceLang::Rust, "b.rs", opts("rust", "b.rs")).unwrap();
    assert_ne!(m_a.witness_hash, m_b.witness_hash);
}

#[test]
fn clean_percent_tracks_todo_count() {
    let clean = "fn f() { return 0; }";
    let dirty = "unsafe { do(); }";
    let (_, m_clean) = convert(clean, SourceLang::Rust, "a.rs", opts("rust", "a.rs")).unwrap();
    let (_, m_dirty) = convert(dirty, SourceLang::Rust, "b.rs", opts("rust", "b.rs")).unwrap();
    assert!(m_clean.clean_translation_percent() > m_dirty.clean_translation_percent());
}
