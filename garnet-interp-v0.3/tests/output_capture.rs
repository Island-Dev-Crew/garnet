//! W-PLAY additive output-capture sink: when a capture is active on the
//! current thread, `print`/`println` program output lands in the capture
//! buffer instead of process stdout; when no capture is active the natives
//! keep their byte-identical stdout path (proven by the untouched
//! workspace suite continuing to pass).

use garnet_interp::Interpreter;

const HELLO: &str = r#"
@caps()
def main() {
  println("Hello from Garnet!")
  0
}
"#;

#[test]
fn capture_collects_println_output() {
    garnet_interp::output::capture_start();
    let mut interp = Interpreter::new();
    interp
        .load_source_with_entry_caps(HELLO, "main")
        .unwrap_or_else(|e| panic!("load failed: {e:?}"));
    interp
        .call_entry("main", vec![])
        .unwrap_or_else(|e| panic!("main failed: {e:?}"));
    let captured = garnet_interp::output::capture_take();
    assert_eq!(Some("Hello from Garnet!\n".to_string()), captured);
}

#[test]
fn capture_collects_print_without_newline() {
    garnet_interp::output::capture_start();
    let mut interp = Interpreter::new();
    interp
        .load_source_with_entry_caps(
            "@caps()\ndef main() {\n  print(\"a\")\n  print(\"b\")\n  0\n}\n",
            "main",
        )
        .unwrap_or_else(|e| panic!("load failed: {e:?}"));
    interp
        .call_entry("main", vec![])
        .unwrap_or_else(|e| panic!("main failed: {e:?}"));
    assert_eq!(
        Some("ab".to_string()),
        garnet_interp::output::capture_take()
    );
}

#[test]
fn take_without_start_is_none() {
    assert_eq!(None, garnet_interp::output::capture_take());
}

#[test]
fn capture_is_reset_after_take() {
    garnet_interp::output::capture_start();
    assert_eq!(Some(String::new()), garnet_interp::output::capture_take());
    assert_eq!(None, garnet_interp::output::capture_take());
}
