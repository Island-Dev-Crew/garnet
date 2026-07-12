//! Additive output-capture sink (W-PLAY).
//!
//! The browser cannot observe process stdout, so the wasm playground needs
//! `print`/`println` program output as a value. When a capture is active on
//! the current thread the prelude natives append here; when no capture is
//! active they keep their original byte-identical `print!`/`println!` path.
//! Purely additive per the RB-7 accessor precedent: default behavior is
//! unchanged and remains proven by the untouched workspace suite.

use std::cell::RefCell;

thread_local! {
    static CAPTURE: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// Begin capturing program output on the current thread, replacing any
/// capture already in progress with a fresh empty buffer.
pub fn capture_start() {
    CAPTURE.with(|slot| *slot.borrow_mut() = Some(String::new()));
}

/// End the current capture and return its buffer, or `None` when no capture
/// was active. Always leaves the thread in the not-capturing state.
pub fn capture_take() -> Option<String> {
    CAPTURE.with(|slot| slot.borrow_mut().take())
}

/// Route one `print`/`println` emission: into the active capture buffer,
/// or byte-identically to stdout when no capture is active.
pub(crate) fn emit(text: &str, newline: bool) {
    let captured = CAPTURE.with(|slot| {
        if let Some(buffer) = slot.borrow_mut().as_mut() {
            buffer.push_str(text);
            if newline {
                buffer.push('\n');
            }
            true
        } else {
            false
        }
    });
    if !captured {
        if newline {
            println!("{text}");
        } else {
            print!("{text}");
        }
    }
}
