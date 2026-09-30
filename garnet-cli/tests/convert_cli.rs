//! T5b (C5-12, C1-18): `garnet convert` names its output per language, prints one
//! summary, and states only what holds: a construct count and whether the
//! output parses, never a "clean translation" percentage.

use std::path::Path;
use std::process::{Command, Output};

fn convert(dir: &Path, lang: &str, file: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_garnet"))
        .current_dir(dir)
        .args(["convert", lang, file])
        .output()
        .unwrap()
}

#[test]
fn outputs_are_named_per_language_so_a_second_conversion_does_not_overwrite() {
    let dir = tempfile::TempDir::new().unwrap();
    std::fs::write(dir.path().join("sample.py"), "def one():\n    return 1\n").unwrap();
    std::fs::write(dir.path().join("sample.rb"), "def one\n  return 1\nend\n").unwrap();
    assert!(convert(dir.path(), "python", "sample.py").status.success());
    assert!(convert(dir.path(), "ruby", "sample.rb").status.success());
    for name in [
        "sample.python.garnet",
        "sample.python.garnet.lineage.json",
        "sample.python.garnet.migrate_todo.md",
        "sample.python.garnet.metrics.json",
        "sample.ruby.garnet",
        "sample.ruby.garnet.lineage.json",
        "sample.ruby.garnet.migrate_todo.md",
        "sample.ruby.garnet.metrics.json",
    ] {
        assert!(dir.path().join(name).is_file(), "missing {name}");
    }
    assert!(!dir.path().join("sample.garnet").exists());
}

#[test]
fn summary_counts_constructs_reports_parsing_and_prints_once() {
    let dir = tempfile::TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("loop.py"),
        "def total(items):\n    s = 0\n    for x in items:\n        s = s + x\n    return s\n",
    )
    .unwrap();
    let out = convert(dir.path(), "python", "loop.py");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "{stdout}");
    assert_eq!(
        stdout.lines().filter(|l| l.starts_with("converted")).count(),
        1,
        "one summary: {stdout}"
    );
    assert!(stdout.contains("constructs mapped without a migration to-do"), "{stdout}");
    assert!(stdout.contains("output parses: yes"), "{stdout}");
    assert!(stdout.contains("run garnet check"), "{stdout}");
    for stale in ["clean translation", "clean-translate", "remove the @sandbox"] {
        assert!(!stdout.contains(stale), "{stale:?} in {stdout}");
    }
}
