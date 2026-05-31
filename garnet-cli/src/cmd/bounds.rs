//! `garnet bounds <file.garnet>` — report declared `@bounded(N)` fuel budgets (S39).
//!
//! Wrap-don't-rebuild: `@bounded(N)` declares a CPU/fuel budget of N
//! Wasmtime-fuel units; ENFORCEMENT lowers to Wasmtime fuel metering — the
//! lowering target. `wasmtime` is not present in this environment, so budgets are
//! DECLARED + reported here, not yet runtime fuel-enforced. No fuel meter is
//! faked.

use crate::{edition_manifest, read_file};
use garnet_check::bounded_functions;
use std::path::PathBuf;
use std::process::ExitCode;

pub fn run(path: PathBuf) -> ExitCode {
    let src = match read_file(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("garnet bounds: {e}");
            return ExitCode::from(1);
        }
    };
    let edition = match edition_manifest::resolve_edition_for(&path) {
        Ok(resolved) => {
            if let Some(warning) = resolved.warning {
                eprintln!("{warning}");
            }
            resolved.edition
        }
        Err(message) => {
            eprintln!("garnet bounds: {message}");
            return ExitCode::from(1);
        }
    };
    let module = match garnet_parser::parse_source_with_edition(&src, edition) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("garnet bounds: parse error: {e}");
            return ExitCode::from(1);
        }
    };

    let bounds = bounded_functions(&module);
    println!("garnet bounds for {}", path.display());
    if bounds.is_empty() {
        println!("  no @bounded(...) fuel budgets declared.");
    } else {
        for (name, fuel) in &bounds {
            println!("  {name}: {fuel} fuel units (Wasmtime-fuel budget)");
        }
    }
    println!(
        "\nnote: enforcement lowers to Wasmtime fuel metering (the lowering target). \
         wasmtime is not present in this environment, so budgets are DECLARED + reported, \
         not yet runtime fuel-enforced (wrap-don't-rebuild)."
    );
    ExitCode::SUCCESS
}
