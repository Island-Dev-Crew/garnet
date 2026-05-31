//! `garnet seal <file.garnet> [--out <path>]` — emit an in-toto seal
//! attestation (S38; `--out` added S51).
//!
//! Wrap, don't rebuild: produces the in-toto predicate over the deterministic
//! build manifest + the capability manifest; `cosign` signs it (detected, not
//! required). The capability manifest is the native SBOM-equivalent.
//!
//! `--out <path>` writes the predicate to a file so it can be fed straight to
//! `cosign attest --predicate <path>` (S51 signed-release lanes): without it the
//! predicate was print-only, and the `cosign attest --predicate <output>` hint
//! had no output path to point at.

use crate::cap_manifest::CapabilityManifest;
use crate::manifest::Manifest;
use crate::seal::{cosign_available, statement_json_with_authorship};
use crate::{edition_manifest, read_file};
use garnet_check::capability_surface;
use std::path::PathBuf;
use std::process::ExitCode;

pub fn run(args: &[String]) -> ExitCode {
    let mut path: Option<PathBuf> = None;
    let mut out: Option<PathBuf> = None;
    let mut authored_by: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--out" => {
                let Some(value) = args.get(i + 1) else {
                    eprintln!("garnet seal: --out requires a <path>");
                    return ExitCode::from(2);
                };
                out = Some(PathBuf::from(value));
                i += 2;
            }
            "--authored-by" => {
                // S65: record an AI-authorship provenance declaration in the
                // predicate, e.g. `ai:claude-opus-4-8`, `ai-assisted:...`,
                // `human:jon`. Self-declared, not detected.
                let Some(value) = args.get(i + 1) else {
                    eprintln!("garnet seal: --authored-by requires a <provenance> (e.g. ai:model)");
                    return ExitCode::from(2);
                };
                authored_by = Some(value.clone());
                i += 2;
            }
            "--help" | "-h" => {
                println!(
                    "usage: garnet seal <file.garnet> [--out <path>] [--authored-by <provenance>]"
                );
                return ExitCode::SUCCESS;
            }
            other if !other.starts_with("--") => {
                path = Some(PathBuf::from(other));
                i += 1;
            }
            other => {
                eprintln!("garnet seal: unknown flag: {other}");
                return ExitCode::from(2);
            }
        }
    }
    let Some(path) = path else {
        eprintln!("usage: garnet seal <file.garnet> [--out <path>] [--authored-by <provenance>]");
        return ExitCode::from(2);
    };

    let src = match read_file(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("garnet seal: {e}");
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
            eprintln!("garnet seal: {message}");
            return ExitCode::from(1);
        }
    };
    let module = match garnet_parser::parse_source_with_edition(&src, edition) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("garnet seal: parse error: {e}");
            return ExitCode::from(1);
        }
    };

    let build = Manifest::build(&src, &module);
    let caps = CapabilityManifest::from_surface(capability_surface(&module));
    let program = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("program");
    let cosign = cosign_available();
    let statement =
        statement_json_with_authorship(program, &build, &caps, cosign, authored_by.as_deref());

    let predicate_ref = if let Some(out_path) = &out {
        if let Err(e) = std::fs::write(out_path, &statement) {
            eprintln!("garnet seal: failed to write {}: {e}", out_path.display());
            return ExitCode::from(1);
        }
        eprintln!("garnet seal: predicate written to {}", out_path.display());
        out_path.display().to_string()
    } else {
        println!("{statement}");
        "<output>".to_string()
    };

    if cosign {
        eprintln!(
            "garnet seal: cosign available — sign this predicate with: \
             cosign attest --predicate {predicate_ref} --type custom"
        );
    } else {
        eprintln!(
            "garnet seal: cosign not installed — in-toto predicate emitted UNSIGNED \
             (wrap-don't-rebuild: install cosign to attest; Garnet does not sign supply-chain itself)"
        );
    }
    ExitCode::SUCCESS
}
