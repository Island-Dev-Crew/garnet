//! A5 — `set_strict_no_frame` is a ONE-WAY LATCH.
//!
//! Once deny-by-default is enabled for a process, nothing can silently re-open
//! it: `set_strict_no_frame(false)` is a deliberate no-op. This lives in its own
//! test binary (a single test) because the flag is a process-global `AtomicBool`;
//! enabling it here must not poison any other test's permissive-default behaviour.

use garnet_interp::{eval::set_strict_no_frame, Interpreter};

/// The trap a frame-less host-authority call raises under strict mode.
const FS_TRAP: &str = "requires @caps(fs)";

#[test]
fn strict_no_frame_is_a_one_way_latch_and_cannot_be_reopened() {
    let interp = Interpreter::new();

    // Baseline: strict OFF (process default) → a frame-less host call is permitted
    // (it reaches the IO and fails only because the path is absent — NOT a caps trap).
    let permissive = interp.eval_expr_src("read_file(\"/garnet_a5_latch_absent\")");
    let permissive_msg = format!("{permissive:?}");
    assert!(permissive.is_err(), "absent file → IO error");
    assert!(
        !permissive_msg.contains(FS_TRAP),
        "strict is off by default; must NOT be a caps trap: {permissive_msg}"
    );

    // Latch ON.
    set_strict_no_frame(true);
    // Attempt to re-open the gate — must be a NO-OP.
    set_strict_no_frame(false);

    // The gate stays closed: the SAME frame-less host call now traps on caps,
    // proving `set_strict_no_frame(false)` did not re-open it.
    let denied = interp.eval_expr_src("read_file(\"/garnet_a5_latch_absent\")");
    let denied_msg = format!("{denied:?}");
    assert!(denied.is_err(), "strict must deny the host call");
    assert!(
        denied_msg.contains(FS_TRAP),
        "the latch must remain closed after a false re-open attempt: {denied_msg}"
    );
}
