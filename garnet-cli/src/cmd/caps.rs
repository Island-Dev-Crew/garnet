//! `garnet caps <path>` — emit the S36 capability manifest as deterministic JSON.
//!
//! Edition-aware parse (S32) + `capability_surface` (S35) over a single `.garnet`
//! file (per-program) or every `.garnet` under a directory (per-package, merged),
//! wrapped in a schema-versioned [`CapabilityManifest`]. The artifact S37
//! `diff-caps` compares and S38 `seal` embeds.

use crate::cap_manifest::{self, CapabilityManifest};
use crate::cmd::verify_gate::collect_targets;
use crate::{edition_manifest, read_file};
use garnet_check::capability_surface;
use std::path::PathBuf;
use std::process::ExitCode;

pub fn run(path: PathBuf) -> ExitCode {
    let targets = match collect_targets(&path) {
        Ok(t) if !t.is_empty() => t,
        Ok(_) => {
            eprintln!(
                "garnet caps: no .garnet files found under {}",
                path.display()
            );
            return ExitCode::from(2);
        }
        Err(e) => {
            eprintln!("garnet caps: {e}");
            return ExitCode::from(2);
        }
    };

    let mut surfaces = Vec::with_capacity(targets.len());
    for target in &targets {
        let src = match read_file(target) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("garnet caps: {e}");
                return ExitCode::from(1);
            }
        };
        let edition = match edition_manifest::resolve_edition_for(target) {
            Ok(resolved) => {
                if let Some(warning) = resolved.warning {
                    eprintln!("{warning}");
                }
                resolved.edition
            }
            Err(message) => {
                eprintln!("garnet caps: {message}");
                return ExitCode::from(1);
            }
        };
        match garnet_parser::parse_source_with_edition(&src, edition) {
            Ok(module) => surfaces.push(capability_surface(&module)),
            Err(e) => {
                eprintln!("garnet caps: parse error in {}: {e}", target.display());
                return ExitCode::from(1);
            }
        }
    }

    let surface = if surfaces.len() == 1 {
        surfaces.pop().expect("one surface")
    } else {
        cap_manifest::merge_surfaces(surfaces)
    };
    println!("{}", CapabilityManifest::from_surface(surface).to_json());
    ExitCode::SUCCESS
}
