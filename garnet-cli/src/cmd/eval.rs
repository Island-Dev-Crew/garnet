//! `garnet eval "<expr>"` — evaluate a single expression.

use super::record;
use garnet_interp::Interpreter;
use std::process::ExitCode;
use std::time::Instant;

pub fn run(src: &str) -> ExitCode {
    let started = Instant::now();
    let interp = Interpreter::new();
    // Firewalled: an interpreter panic (e.g. the `i64::MIN.abs()` overflow)
    // degrades to a controlled exit-1 diagnostic instead of aborting the
    // process — the `garnet run` lane's guarantee, now on `eval` too.
    match crate::panic_firewall::firewalled(|| interp.eval_expr_src(src)) {
        Ok(Ok(v)) => {
            println!("{}", v.display());
            record("eval", "<inline>", src, "ok", None, started, 0);
            ExitCode::SUCCESS
        }
        Ok(Err(e)) => {
            eprintln!("{e}");
            record(
                "eval",
                "<inline>",
                src,
                "runtime_err",
                Some(format!("{e}")),
                started,
                1,
            );
            ExitCode::from(1)
        }
        Err(panic_msg) => {
            eprintln!("runtime error: {panic_msg}");
            record(
                "eval",
                "<inline>",
                src,
                "panic",
                Some(panic_msg),
                started,
                1,
            );
            ExitCode::from(1)
        }
    }
}
