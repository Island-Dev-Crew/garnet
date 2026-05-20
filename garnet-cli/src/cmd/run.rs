//! `garnet run <file>` — parse, load, and invoke `main` if it exists.

use super::{cache_file_label, record, surface_prior};
use crate::read_file;
use garnet_interp::Interpreter;
use garnet_vm::{run_source_with_options, RunOptions};
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RunMode {
    Interp,
    Vm,
}

pub fn run(args: &[String]) -> ExitCode {
    let (mode, path) = match parse_args(args) {
        Ok(parsed) => parsed,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::from(2);
        }
    };
    let started = Instant::now();
    let src = match read_file(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(1);
        }
    };
    let file_label = cache_file_label(&path);
    surface_prior(&src);
    match mode {
        RunMode::Interp => run_interpreter(&file_label, &src, started),
        RunMode::Vm => run_vm(&file_label, &src, started),
    }
}

fn parse_args(args: &[String]) -> Result<(RunMode, PathBuf), String> {
    let mut mode = RunMode::Interp;
    let mut path: Option<PathBuf> = None;
    for arg in args {
        match arg.as_str() {
            "--interp" => mode = RunMode::Interp,
            "--vm" => mode = RunMode::Vm,
            "--help" | "-h" => {
                return Err("usage: garnet run [--interp|--vm] <file.garnet>".to_string())
            }
            other if other.starts_with("--") => return Err(format!("unknown run flag: {other}")),
            other => {
                if path.is_some() {
                    return Err("usage: garnet run [--interp|--vm] <file.garnet>".to_string());
                }
                path = Some(PathBuf::from(other));
            }
        }
    }
    path.map(|path| (mode, path))
        .ok_or_else(|| "usage: garnet run [--interp|--vm] <file.garnet>".to_string())
}

fn run_interpreter(file_label: &str, src: &str, started: Instant) -> ExitCode {
    let mut interp = Interpreter::new();
    if let Err(e) = interp.load_source(src) {
        eprintln!("load error: {e}");
        record(
            "run",
            file_label,
            src,
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
                record("run", file_label, src, "ok", None, started, 0);
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("runtime error: {e}");
                record(
                    "run",
                    file_label,
                    src,
                    "runtime_err",
                    Some(format!("{e}")),
                    started,
                    1,
                );
                ExitCode::from(1)
            }
        }
    } else {
        record("run", file_label, src, "ok", None, started, 0);
        ExitCode::SUCCESS
    }
}

fn run_vm(file_label: &str, src: &str, started: Instant) -> ExitCode {
    match run_source_with_options(src, RunOptions { emit_stdout: true }) {
        Ok(result) => {
            if result.called_entry {
                println!("=> {}", result.value.display());
            }
            record("run", file_label, src, "ok", None, started, 0);
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("vm error: {error}");
            record(
                "run",
                file_label,
                src,
                "runtime_err",
                Some(format!("{error}")),
                started,
                1,
            );
            ExitCode::from(1)
        }
    }
}
