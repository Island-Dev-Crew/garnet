//! W-PLAY Task 1 proving traps: `run_source` returns REAL program output,
//! and authority failures surface as diagnostics — never fabricated results.

use garnet_wasm::{run_source, run_source_json, ExitClass};

/// The canonical hello — mirrors `examples/hello.garnet` and must produce
/// the same output the CLI records for it.
const HELLO: &str = r#"
@caps()
def main() {
  println("Hello from Garnet!")
  0
}
"#;

#[test]
fn hello_returns_the_cli_recorded_output() {
    let result = run_source(HELLO);
    assert_eq!(ExitClass::Ok, result.exit_class, "{:?}", result.diagnostic);
    assert_eq!("Hello from Garnet!\n", result.stdout);
    assert_eq!(None, result.diagnostic);
}

#[test]
fn json_surface_is_stable() {
    let parsed: serde_json::Value =
        serde_json::from_str(&run_source_json(HELLO)).expect("valid JSON");
    assert_eq!("garnet.wasm.run/1", parsed["schema"]);
    assert_eq!("ok", parsed["exit_class"]);
    assert_eq!("Hello from Garnet!\n", parsed["stdout"]);
}

#[test]
fn parse_error_is_a_load_error_with_diagnostic() {
    let result = run_source("def main( {");
    assert_eq!(ExitClass::LoadError, result.exit_class);
    assert!(result.diagnostic.is_some());
    assert_eq!("", result.stdout);
}

#[test]
fn runtime_error_carries_partial_real_output() {
    let src = r#"
@caps()
def main() {
  println("before the error")
  1 / 0
}
"#;
    let result = run_source(src);
    assert_eq!(
        ExitClass::RuntimeError,
        result.exit_class,
        "{:?}",
        result.diagnostic
    );
    assert_eq!("before the error\n", result.stdout);
    assert!(result.diagnostic.is_some());
}

#[test]
fn undeclared_authority_is_trapped_not_granted() {
    // `main` declares no capabilities; a proc-authority call inside it must
    // trap at runtime exactly as the native interpreter traps it.
    let src = r#"
@caps()
def main() {
  proc::run("echo hi")
  0
}
"#;
    let result = run_source(src);
    assert_ne!(ExitClass::Ok, result.exit_class);
    let diagnostic = result.diagnostic.unwrap_or_default();
    assert!(
        diagnostic.to_lowercase().contains("cap")
            || diagnostic.to_lowercase().contains("proc")
            || diagnostic.to_lowercase().contains("undefined"),
        "diagnostic should name the authority failure: {diagnostic}"
    );
}

#[test]
fn consecutive_runs_do_not_leak_output() {
    let first = run_source(HELLO);
    let second = run_source(HELLO);
    assert_eq!(first.stdout, second.stdout);
    assert_eq!("Hello from Garnet!\n", second.stdout);
}
