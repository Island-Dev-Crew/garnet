//! `garnet eval "<expr>"` — evaluate a single expression.

use super::record;
use garnet_interp::Interpreter;
use std::process::ExitCode;
use std::time::Instant;

pub fn run(src: &str) -> ExitCode {
    let started = Instant::now();
    let interp = Interpreter::new();
    match interp.eval_expr_src(src) {
        Ok(v) => {
            println!("{}", v.display());
            record("eval", "<inline>", src, "ok", None, started, 0);
            ExitCode::SUCCESS
        }
        Err(e) => {
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
    }
}
