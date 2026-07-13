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
    // A PERMISSIVE instance (the explicit opt-out from strict-by-default): with
    // the process latch off, its frame-less host calls are allowed. This also
    // proves the global latch dominates a permissive instance once set.
    let interp = Interpreter::new_permissive();

    // Baseline: global latch OFF → a frame-less host call on a permissive
    // instance is permitted (it reaches the IO and fails only because the path
    // is absent — NOT a caps trap).
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

    // The gate stays closed: the SAME frame-less host call on the SAME
    // permissive instance now traps on caps, proving both that
    // `set_strict_no_frame(false)` did not re-open it AND that the global latch
    // overrides a permissive instance.
    let denied = interp.eval_expr_src("read_file(\"/garnet_a5_latch_absent\")");
    let denied_msg = format!("{denied:?}");
    assert!(denied.is_err(), "strict must deny the host call");
    assert!(
        denied_msg.contains(FS_TRAP),
        "the latch must remain closed after a false re-open attempt: {denied_msg}"
    );
}
