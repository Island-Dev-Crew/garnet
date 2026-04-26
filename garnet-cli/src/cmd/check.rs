//! `garnet check <file>` — run the safe-mode checker (CapCaps + borrow + audit).

use super::{record, surface_prior};
use crate::read_file;
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
    let module = match garnet_parser::parse_source(&src) {
        Ok(m) => m,
        Err(e) => {
            let report = miette::Report::new(e).with_source_code(src.clone());
            eprintln!("{report:?}");
            record(
                "check",
                &path.display().to_string(),
                &src,
                "parse_err",
                Some("UnexpectedToken".to_string()),
                started,
                1,
            );
            return ExitCode::from(1);
        }
    };
    let report = garnet_check::check_module(&module);
    for err in &report.errors {
        println!("{err}");
    }
    println!(
        "\n{} functions checked, {} boundary call sites, {} diagnostics",
        report.mode_map.len(),
        report.boundary_call_sites,
        report.errors.len()
    );
    if report.ok() {
        record(
            "check",
            &path.display().to_string(),
            &src,
            "ok",
            None,
            started,
            0,
        );
        ExitCode::SUCCESS
    } else {
        record(
            "check",
            &path.display().to_string(),
            &src,
            "check_err",
            Some("safe_violation".to_string()),
            started,
            1,
        );
        ExitCode::from(1)
    }
}
