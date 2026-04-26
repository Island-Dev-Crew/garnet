//! `garnet parse <file>` — parse a file and print a structural summary.

use super::{describe_item, record, surface_prior};
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
    match garnet_parser::parse_source(&src) {
        Ok(module) => {
            println!(
                "parsed {} ({} items, safe={})",
                path.display(),
                module.items.len(),
                module.safe
            );
            for item in &module.items {
                println!("  - {}", describe_item(item));
            }
            record(
                "parse",
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
            let report = miette::Report::new(e).with_source_code(src.clone());
            eprintln!("{report:?}");
            record(
                "parse",
                &path.display().to_string(),
                &src,
                "parse_err",
                Some("UnexpectedToken".to_string()),
                started,
                1,
            );
            ExitCode::from(1)
        }
    }
}
