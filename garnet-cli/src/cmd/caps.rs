//! `garnet caps <path>` — emit the S36 capability manifest as deterministic JSON.
//!
//! Builds the merged capability surface (S35) for a `.garnet` file (per-program)
//! or a directory (per-package) via [`crate::cap_manifest::surface_for_path`],
//! wrapped in a schema-versioned [`CapabilityManifest`]. The artifact S37
//! `diff-caps` compares and S38 `seal` embeds.

use crate::cap_manifest::{surface_for_path, CapabilityManifest};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapsFormat {
    Internal,
    StandardProfile,
}

pub fn run(path: PathBuf, format: CapsFormat) -> ExitCode {
    match surface_for_path(&path) {
        Ok(surface) => {
            let manifest = CapabilityManifest::from_surface(surface);
            let json = match format {
                CapsFormat::Internal => manifest.to_json(),
                CapsFormat::StandardProfile => manifest.to_standard_profile_json(),
            };
            println!("{json}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("garnet caps: {message}");
            if message.contains("no .garnet files") {
                ExitCode::from(2)
            } else {
                ExitCode::from(1)
            }
        }
    }
}
