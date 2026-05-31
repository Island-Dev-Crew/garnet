//! S43 — docs-as-tests integration test (runs the built binary).
//!
//! `garnet doctest <file>` extracts ```garnet fences from `///` doc comments,
//! loads the file's definitions, runs each example, and checks any `# => value`
//! assertion. It exits 0 only if every example passes.

use std::path::{Path, PathBuf};
use std::process::Command;

fn garnet() -> Command {
    Command::new(env!("CARGO_BIN_EXE_garnet"))
}

fn fresh(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("garnet_s43_{tag}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn write(dir: &Path, name: &str, body: &str) -> PathBuf {
    let p = dir.join(name);
    std::fs::write(&p, body).unwrap();
    p
}

#[test]
fn passing_example_exits_zero() {
    let dir = fresh("pass");
    let p = write(
        dir.as_path(),
        "math.garnet",
        "/// Double a number.\n\
         ///\n\
         /// ```garnet\n\
         /// double(21)\n\
         /// # => 42\n\
         /// ```\n\
         def double(x) { x * 2 }\n",
    );
    let out = garnet().arg("doctest").arg(&p).output().unwrap();
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(out.status.success(), "passing example -> exit 0: {s}");
    assert!(s.contains("1 example checked, 1 passed, 0 failed"), "{s}");
}

#[test]
fn wrong_assertion_exits_one() {
    let dir = fresh("fail");
    let p = write(
        dir.as_path(),
        "math.garnet",
        "/// Double a number.\n\
         ///\n\
         /// ```garnet\n\
         /// double(21)\n\
         /// # => 43\n\
         /// ```\n\
         def double(x) { x * 2 }\n",
    );
    let out = garnet().arg("doctest").arg(&p).output().unwrap();
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(!out.status.success(), "wrong assertion -> exit 1: {s}");
    assert!(s.contains("0 passed, 1 failed"), "{s}");
    assert!(s.contains("expected `43`, got `42`"), "{s}");
}

#[test]
fn json_reports_pass_and_fail() {
    let dir = fresh("json");
    let p = write(
        dir.as_path(),
        "math.garnet",
        "/// ```garnet\n\
         /// 1 + 1\n\
         /// # => 2\n\
         /// ```\n\
         def ok_fn() { 0 }\n\
         /// ```garnet\n\
         /// 1 + 1\n\
         /// # => 3\n\
         /// ```\n\
         def bad_fn() { 0 }\n",
    );
    let out = garnet()
        .args(["doctest", "--format", "json"])
        .arg(&p)
        .output()
        .unwrap();
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(!out.status.success(), "one failure -> exit 1: {s}");
    assert!(s.contains(r#""status":"pass""#), "{s}");
    assert!(s.contains(r#""status":"fail""#), "{s}");
    assert!(s.contains(r#""ok":false"#), "{s}");
}

#[test]
fn no_examples_is_not_a_failure() {
    let dir = fresh("none");
    let p = write(dir.as_path(), "plain.garnet", "def main() { 0 }\n");
    let out = garnet().arg("doctest").arg(&p).output().unwrap();
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(out.status.success(), "no examples -> exit 0: {s}");
    assert!(s.contains("no `garnet` doc examples found"), "{s}");
}

#[test]
fn advertised_demonstrator_passes() {
    // The shipped example must itself be green — executable docs, dogfooded.
    let manifest = env!("CARGO_MANIFEST_DIR");
    let example = Path::new(manifest)
        .parent()
        .unwrap()
        .join("examples")
        .join("documented_math.garnet");
    let out = garnet().arg("doctest").arg(&example).output().unwrap();
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(out.status.success(), "demonstrator must pass: {s}");
    assert!(s.contains("3 passed, 0 failed"), "{s}");
}
