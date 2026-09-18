//! D-107 acceptance — `garnet check` rejects malformed enum variant
//! construction in safe functions.
//!
//! The four probe programs are the register's q1 / q2 / q3 / q7 shapes,
//! written as `garnet check` inputs: a unit variant given a payload, a
//! payload variant used bare, a payload variant given the wrong arity, and a
//! variant that does not exist. Each must exit non-zero with a
//! `safe-mode violation` naming the construction. The control program uses
//! every variant correctly plus an impl-associated function and must exit 0.

use std::process::Command;

fn garnet() -> Command {
    Command::new(env!("CARGO_BIN_EXE_garnet"))
}

fn check(program: &str) -> std::process::Output {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("prog.garnet");
    std::fs::write(&path, program).unwrap();
    garnet().arg("check").arg(&path).output().unwrap()
}

fn combined(out: &std::process::Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

const PRELUDE: &str = "enum Shape { Circle(Float), Rect(Float, Float), Empty }\n\
    @caps()\ndef main() -> int { 0 }\n";

fn assert_rejected(program: &str, needle: &str, label: &str) {
    let out = check(program);
    let text = combined(&out);
    assert!(
        !out.status.success(),
        "{label}: garnet check must exit non-zero; output: {text}"
    );
    assert!(
        text.contains("safe-mode violation") && text.contains(needle),
        "{label}: expected `{needle}` in a safe-mode violation; output: {text}"
    );
}

/// q1 — `Shape::Empty(1.0)`: unit variant given a payload.
#[test]
fn q1_unit_variant_given_payload_fails_check() {
    assert_rejected(
        &format!("{PRELUDE}fn make() -> Shape {{ Shape::Empty(1.0) }}\n"),
        "unit variant `Shape::Empty` takes no payload, given 1",
        "q1",
    );
}

/// q2 — `Shape::Circle` used bare: payload variant with no payload.
#[test]
fn q2_payload_variant_without_payload_fails_check() {
    assert_rejected(
        &format!("{PRELUDE}fn make() -> Shape {{ Shape::Circle }}\n"),
        "payload variant `Shape::Circle` requires 1 field but is used without a payload",
        "q2",
    );
}

/// q3 — `Shape::Circle(1.0, 2.0)`: payload variant given two fields.
#[test]
fn q3_payload_variant_arity_mismatch_fails_check() {
    assert_rejected(
        &format!("{PRELUDE}fn make() -> Shape {{ Shape::Circle(1.0, 2.0) }}\n"),
        "payload variant `Shape::Circle` expects 1 field, given 2",
        "q3",
    );
}

/// q7 — `Shape::Triangle(1.0)`: the variant does not exist.
#[test]
fn q7_unknown_variant_fails_check() {
    assert_rejected(
        &format!("{PRELUDE}fn make() -> Shape {{ Shape::Triangle(1.0) }}\n"),
        "enum `Shape` has no variant `Triangle`",
        "q7",
    );
}

/// Control — well-formed constructions and an impl-associated fn pass.
#[test]
fn well_formed_constructions_pass_check() {
    let program = format!(
        "{PRELUDE}impl Shape {{ fn unit() -> Shape {{ Shape::Circle(1.0) }} }}\n\
         fn make() -> Shape {{\n  let a = Shape::Circle(2.0)\n  let b = Shape::Rect(1.0, 2.0)\n  \
         let c = Shape::Empty\n  Shape::unit()\n}}\n"
    );
    let out = check(&program);
    assert!(
        out.status.success(),
        "control must pass garnet check; output: {}",
        combined(&out)
    );
}

/// D-107b — an `impl` on `Other::Shape` must not whitelist `Target::Shape::ghost()`.
#[test]
fn q8_impl_on_a_same_basename_enum_fails_check_for_the_other_enum() {
    assert_rejected(
        "module Other {\n  enum Shape { Circle(Float), Square(Float) }\n  impl Shape {\n    fn ghost() -> Shape { Shape::Circle(1.0) }\n  }\n}\nmodule Target {\n  enum Shape { Circle(Float), Square(Float) }\n}\nfn make() -> Target::Shape { Target::Shape::ghost() }\n@caps()\ndef main() { 0 }\n",
        "enum `Shape` has no variant `ghost`",
        "q8",
    );
}
