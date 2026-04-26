//! `garnet run <file>` — parse, load, and invoke `main` if it exists.

use super::{record, surface_prior};
use crate::read_file;
use garnet_interp::Interpreter;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

pub fn run(path: PathBuf) -> ExitCode {
    let started = Instant::now();
    let src = match read_file(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(1);
        }
    };
    surface_prior(&src);
    let mut interp = Interpreter::new();
    if let Err(e) = interp.load_source(&src) {
        eprintln!("load error: {e}");
        record(
            "run",
            &path.display().to_string(),
            &src,
            "parse_err",
            Some(format!("{e}")),
            started,
            1,
        );
        return ExitCode::from(1);
    }
    // If a `main` function exists, call it; otherwise just exit success.
    if interp.global.get("main").is_some() {
        match interp.call("main", vec![]) {
            Ok(v) => {
                println!("=> {}", v.display());
                record(
                    "run",
                    &path.display().to_string(),
                    &src,
                    "ok",
                    None,
                    started,
                    0,
                );
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("runtime error: {e}");
                record(
                    "run",
                    &path.display().to_string(),
                    &src,
                    "runtime_err",
                    Some(format!("{e}")),
                    started,
                    1,
                );
                ExitCode::from(1)
            }
        }
    } else {
        record(
            "run",
            &path.display().to_string(),
            &src,
            "ok",
            None,
            started,
            0,
        );
        ExitCode::SUCCESS
    }
}
