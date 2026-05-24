//! `garnet parse [--mode ast|cst] <file>` — parse a file and print a summary.

use super::{cache_file_label, describe_item, record, surface_prior};
use crate::read_file;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParseMode {
    Ast,
    Cst,
}

pub fn run(args: &[String]) -> ExitCode {
    let Some((mode, path)) = parse_args(args) else {
        return ExitCode::from(2);
    };

    match mode {
        ParseMode::Ast => run_ast(path),
        ParseMode::Cst => run_cst(path),
    }
}

fn usage() {
    eprintln!("usage: garnet parse [--mode ast|cst] <file.garnet>");
}

fn parse_args(args: &[String]) -> Option<(ParseMode, PathBuf)> {
    let mut mode = ParseMode::Ast;
    let mut file: Option<PathBuf> = None;
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "--mode" => {
                let Some(value) = args.get(i + 1) else {
                    usage();
                    return None;
                };
                mode = parse_mode(value)?;
                i += 2;
            }
            "--mode=ast" => {
                mode = ParseMode::Ast;
                i += 1;
            }
            "--mode=cst" => {
                mode = ParseMode::Cst;
                i += 1;
            }
            other if other.starts_with("--") => {
                eprintln!("garnet parse: unknown flag: {other}");
                usage();
                return None;
            }
            other => {
                if file.is_some() {
                    eprintln!("garnet parse: unexpected extra argument: {other}");
                    usage();
                    return None;
                }
                file = Some(PathBuf::from(other));
                i += 1;
            }
        }
    }

    let Some(path) = file else {
        usage();
        return None;
    };

    Some((mode, path))
}

fn parse_mode(value: &str) -> Option<ParseMode> {
    match value {
        "ast" => Some(ParseMode::Ast),
        "cst" => Some(ParseMode::Cst),
        other => {
            eprintln!("garnet parse: unknown parse mode: {other}");
            usage();
            None
        }
    }
}

fn run_ast(path: PathBuf) -> ExitCode {
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
            record("parse", &file_label, &src, "ok", None, started, 0);
            ExitCode::SUCCESS
        }
        Err(e) => {
            let report = miette::Report::new(e).with_source_code(src.clone());
            eprintln!("{report:?}");
            record(
                "parse",
                &file_label,
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

fn run_cst(path: PathBuf) -> ExitCode {
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

    let parse = garnet_cst::parse_cst(&src);
    let roundtrip = parse.to_source() == src;
    let token_count = garnet_cst::token_infos(parse.syntax()).len();
    println!(
        "parsed {} as cst (tokens={}, errors={}, roundtrip={})",
        path.display(),
        token_count,
        parse.errors.len(),
        roundtrip
    );
    println!("  - root {:?}", parse.syntax().kind());

    if !roundtrip {
        eprintln!("cst parse failed byte-identical round-trip check");
        record(
            "parse",
            &file_label,
            &src,
            "parse_err",
            Some("CstRoundTripMismatch".to_string()),
            started,
            1,
        );
        return ExitCode::from(1);
    }

    if !parse.ok() {
        eprintln!("cst parse recorded {} error(s)", parse.errors.len());
        for error in &parse.errors {
            eprintln!("  byte {}: {}", error.offset, error.message);
        }
        record(
            "parse",
            &file_label,
            &src,
            "parse_err",
            Some("CstSyntaxError".to_string()),
            started,
            1,
        );
        return ExitCode::from(1);
    }

    record("parse", &file_label, &src, "ok", None, started, 0);
    ExitCode::SUCCESS
}
