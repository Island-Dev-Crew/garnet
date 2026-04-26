//! `garnet verify <file> <manifest.json> [--signature]` — verify the
//! manifest matches the source (and signature, if `--signature` or if the
//! manifest carries one).

use crate::manifest::Manifest;
use crate::read_file;
use std::path::PathBuf;
use std::process::ExitCode;

pub fn run(file: PathBuf, manifest_path: PathBuf, require_signature: bool) -> ExitCode {
    let src = match read_file(&file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(1);
        }
    };
    let module = match garnet_parser::parse_source(&src) {
        Ok(m) => m,
        Err(e) => {
            let report = miette::Report::new(e).with_source_code(src);
            eprintln!("{report:?}");
            return ExitCode::from(1);
        }
    };
    let on_disk = match read_file(&manifest_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(1);
        }
    };
    let stored = match Manifest::from_canonical_json(&on_disk) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("manifest parse error: {e}");
            return ExitCode::from(1);
        }
    };
    let recomputed = Manifest::build(&src, &module);

    if stored.source_hash != recomputed.source_hash {
        eprintln!(
            "FAIL source_hash mismatch:\n  stored:     {}\n  recomputed: {}",
            stored.source_hash, recomputed.source_hash
        );
        return ExitCode::from(2);
    }
    if stored.ast_hash != recomputed.ast_hash {
        eprintln!(
            "FAIL ast_hash mismatch:\n  stored:     {}\n  recomputed: {}",
            stored.ast_hash, recomputed.ast_hash
        );
        return ExitCode::from(2);
    }
    if stored.schema != recomputed.schema {
        eprintln!(
            "FAIL schema mismatch: stored={}, expected={}",
            stored.schema, recomputed.schema
        );
        return ExitCode::from(2);
    }

    // v3.4.1 ManifestSig — signature verification.
    //
    // `--signature` flag forces a signed manifest + valid signature. Without
    // the flag, an unsigned manifest passes (backwards compat) but a signed
    // manifest whose signature is invalid is ALWAYS rejected — a tampered
    // signed manifest cannot quietly pass by dropping the flag.
    if stored.is_signed() {
        match stored.verify_signature() {
            Ok(()) => {
                println!("OK {} matches manifest + signature valid", file.display());
                println!("  source_hash = {}", stored.source_hash);
                println!("  ast_hash    = {}", stored.ast_hash);
                println!("  signed_by   = {}", stored.signer_pubkey);
                return ExitCode::SUCCESS;
            }
            Err(e) => {
                eprintln!("FAIL signature verification: {e}");
                return ExitCode::from(2);
            }
        }
    } else if require_signature {
        eprintln!("FAIL manifest is unsigned but --signature was required");
        return ExitCode::from(2);
    }

    println!("OK {} matches manifest (unsigned)", file.display());
    println!("  source_hash = {}", stored.source_hash);
    println!("  ast_hash    = {}", stored.ast_hash);
    ExitCode::SUCCESS
}
