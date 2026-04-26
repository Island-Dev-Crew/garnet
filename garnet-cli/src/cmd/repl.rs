//! `garnet repl [file]` — interactive REPL (optionally preloading a file).

use crate::read_file;
use std::path::PathBuf;
use std::process::ExitCode;

pub fn run(preload: Option<PathBuf>) -> ExitCode {
    let mut repl = garnet_interp::repl::Repl::new();
    if let Some(p) = preload {
        match read_file(&p) {
            Ok(src) => {
                if let Err(e) = repl.preload(&src) {
                    eprintln!("preload error: {e}");
                    return ExitCode::from(1);
                }
                println!("preloaded {}", p.display());
            }
            Err(e) => {
                eprintln!("{e}");
                return ExitCode::from(1);
            }
        }
    }
    if let Err(e) = repl.run_stdio() {
        eprintln!("REPL IO error: {e}");
        return ExitCode::from(1);
    }
    ExitCode::SUCCESS
}
