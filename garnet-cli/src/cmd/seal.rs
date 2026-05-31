//! `garnet seal <file.garnet>` — emit an in-toto seal attestation (S38).
//!
//! Wrap, don't rebuild: produces the in-toto predicate over the deterministic
//! build manifest + the capability manifest; `cosign` signs it (detected, not
//! required). The capability manifest is the native SBOM-equivalent.

use crate::cap_manifest::CapabilityManifest;
use crate::manifest::Manifest;
use crate::seal::{cosign_available, statement_json};
use crate::{edition_manifest, read_file};
use garnet_check::capability_surface;
use std::path::PathBuf;
use std::process::ExitCode;

pub fn run(path: PathBuf) -> ExitCode {
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

    println!("{}", statement_json(program, &build, &caps, cosign));
    if cosign {
        eprintln!(
            "garnet seal: cosign available — sign this predicate with: \
             cosign attest --predicate <output> --type custom"
        );
    } else {
        eprintln!(
            "garnet seal: cosign not installed — in-toto predicate emitted UNSIGNED \
             (wrap-don't-rebuild: install cosign to attest; Garnet does not sign supply-chain itself)"
        );
    }
    ExitCode::SUCCESS
}
