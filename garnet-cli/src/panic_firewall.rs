//! Process-abort firewall for the interpreter-invoking CLI lanes.
//!
//! ## The gap this closes
//!
//! The `run` lane ([`crate::cmd::run`]) evaluates a program on a large-stack
//! `garnet-interp` thread and turns a thread panic into a controlled exit via
//! `JoinHandle::join`. The `eval`, `repl`, `test`, and `doctest` lanes instead
//! invoke the interpreter on the **main thread**, so before this module a panic
//! inside the interpreter aborted the whole process (`eval`/`test`/`doctest`
//! exit `101`) or killed the entire interactive session (`repl`). A concrete
//! reproducer: `garnet eval "(0 - 9223372036854775807 - 1).abs()"` — the
//! `i64::MIN.abs()` overflow panics, and the unfirewalled lane aborted with the
//! raw Rust panic instead of a controlled diagnostic.
//!
//! ## Why `catch_unwind`, not the run lane's spawn-and-join
//!
//! The interpreter's environment is `Rc`-based (`garnet_interp`'s `Env` holds
//! `Rc<Env>` parents), so `Interpreter` is **`!Send`** and cannot be moved onto
//! or shared with a spawned thread. The run lane sidesteps this by constructing
//! the interpreter *inside* the spawned thread (nothing `!Send` crosses the
//! boundary), but the `repl` lane must keep one interpreter alive across many
//! lines on the main thread, and `test`/`doctest` reuse a per-file interpreter.
//! [`std::panic::catch_unwind`] recovers an unwinding panic **in place**, which
//! is the only option that fits a `!Send`, main-thread interpreter.
//!
//! ## Scope (honest boundary)
//!
//! This catches **unwinding** panics: `panic!`, `unwrap`/`expect` on `None`, an
//! out-of-contract `unreachable!`, the `i64::MIN.abs()` overflow, and the like.
//! It does **not** catch a stack overflow from unbounded recursion — that is a
//! non-unwinding abort (`SIGSEGV`/`SIGABRT`) that `catch_unwind` cannot recover.
//! Deep-but-finite recursion is mitigated by the run lane's large stack; truly
//! unbounded recursion is the `@bounded`/`@max_depth` enforcement story (S89),
//! not a panic-firewall question. Named, not faked.

use std::any::Any;
use std::cell::Cell;
use std::panic::{self, AssertUnwindSafe};
use std::sync::Once;

thread_local! {
    /// True while *this* thread is executing inside [`firewalled`]. The custom
    /// panic hook below stays quiet only for a firewalled thread, so any other
    /// concurrent code — notably the other tests in this crate's test binary
    /// and `#[should_panic]` tests — still gets normal panic reporting.
    static IN_FIREWALL: Cell<bool> = const { Cell::new(false) };
}

/// Install, exactly once per process, a panic hook that suppresses the default
/// `thread '…' panicked at …` line and backtrace note **only** for a panic
/// raised on a thread currently inside [`firewalled`]; the lane prints its own
/// clean `runtime error: …` line instead. Any non-firewalled panic — and any
/// panic at all when `RUST_BACKTRACE` is set — falls through to the previously
/// installed hook unchanged, so failure reporting and developer backtraces are
/// never swallowed. This is intentionally cleaner than the run lane's
/// pass-through (which still prints the raw panic line); unifying the run lane
/// onto this hook is a possible follow-up.
fn install_hook_once() {
    static HOOK: Once = Once::new();
    HOOK.call_once(|| {
        let previous = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            let firewalled_here = IN_FIREWALL.with(Cell::get);
            let want_backtrace = std::env::var_os("RUST_BACKTRACE").is_some();
            if !firewalled_here || want_backtrace {
                previous(info);
            }
        }));
    });
}

/// Run `f` on the current thread, converting an unwinding panic into a clean
/// `Err(message)` rather than letting it abort the process.
///
/// `AssertUnwindSafe` is sound here: the interpreter uses interior mutability
/// (`RefCell`/`Rc`), which is not `UnwindSafe`, but unwinding drops every borrow
/// guard, and the caught panic leaves the interpreter in a state the caller
/// either discards (`eval`/`test`/`doctest` build a fresh interpreter per item)
/// or treats as a reported error and continues from (`repl`) — no torn value is
/// observed across the boundary.
pub(crate) fn firewalled<T>(f: impl FnOnce() -> T) -> Result<T, String> {
    install_hook_once();
    let restore = IN_FIREWALL.with(Cell::get);
    IN_FIREWALL.with(|flag| flag.set(true));
    let result = panic::catch_unwind(AssertUnwindSafe(f));
    // Runs whether `f` returned or unwound (catch_unwind absorbs the unwind),
    // and restores the prior value so a nested firewall cannot un-arm the outer.
    IN_FIREWALL.with(|flag| flag.set(restore));
    result.map_err(message_of)
}

/// Best-effort human-readable text from a caught panic payload. Rust panic
/// payloads are `&'static str` (from `panic!("lit")`) or `String` (from
/// `panic!("{}", x)` / overflow checks); anything else is reported generically.
fn message_of(payload: Box<dyn Any + Send>) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "interpreter panicked".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passes_through_a_normal_return() {
        let r = firewalled(|| 2 + 2);
        assert_eq!(r, Ok(4));
    }

    #[test]
    fn catches_a_str_panic_and_returns_its_message() {
        let r: Result<(), String> = firewalled(|| panic!("boom-str"));
        assert_eq!(r, Err("boom-str".to_string()));
    }

    #[test]
    fn catches_a_formatted_string_panic() {
        let n = 7;
        let r: Result<(), String> = firewalled(|| panic!("boom-{n}"));
        assert_eq!(r, Err("boom-7".to_string()));
    }

    #[test]
    fn catches_an_arithmetic_overflow_panic() {
        // The exact class the lanes must survive: i64::MIN.abs() overflows.
        let m = i64::MIN;
        let r: Result<i64, String> = firewalled(|| m.abs());
        assert!(r.is_err(), "overflow must be caught, got {r:?}");
        assert!(
            r.as_ref().unwrap_err().contains("overflow"),
            "message should name the overflow, got {r:?}"
        );
    }

    #[test]
    fn flag_is_cleared_after_a_caught_panic() {
        let _ = firewalled(|| panic!("x"));
        // A subsequent normal call must still work (flag restored, no poisoning).
        assert_eq!(firewalled(|| 1), Ok(1));
        assert!(
            !IN_FIREWALL.with(Cell::get),
            "firewall flag must be cleared"
        );
    }
}
