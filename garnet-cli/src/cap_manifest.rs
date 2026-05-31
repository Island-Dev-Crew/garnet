//! S36 — the capability manifest.
//!
//! A per-program / per-package artifact derived from S35's
//! [`garnet_check::CapabilitySurface`], serialized as **deterministic JSON**
//! (no `serde`, per the `manifest.rs` determinism stance; reusing
//! `crate::diagnostics::json_escape`). This is the artifact S37 `diff-caps`
//! compares across revisions and S38 `seal` embeds. It is distinct from the
//! build [`crate::manifest::Manifest`], which carries source/AST hashes but no
//! capability surface.

use crate::cmd::verify_gate::collect_targets;
use crate::diagnostics::json_escape;
use crate::{edition_manifest, read_file};
use garnet_check::{capability_surface, CapabilitySurface};
use std::collections::BTreeSet;
use std::path::Path;

/// Schema identifier baked into every capability manifest. Bump when the shape
/// changes; older consumers reject manifests they do not recognize.
pub const SCHEMA: &str = "garnet-capability-manifest-v1";

/// The declared capability surface of a program or package, as a versioned
/// manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityManifest {
    pub schema: String,
    pub surface: CapabilitySurface,
}

impl CapabilityManifest {
    /// Wrap a [`CapabilitySurface`] in a schema-versioned manifest.
    pub fn from_surface(surface: CapabilitySurface) -> Self {
        CapabilityManifest {
            schema: SCHEMA.to_string(),
            surface,
        }
    }

    /// Deterministic JSON: `{schema, aggregate, functions:[{name,caps}], wildcard}`.
    /// Field order is fixed; the surface is already sorted + deduplicated (S35).
    pub fn to_json(&self) -> String {
        let aggregate = json_str_array(&self.surface.aggregate);
        let functions = self
            .surface
            .per_function
            .iter()
            .map(|(name, caps)| {
                format!(
                    "{{\"name\":\"{}\",\"caps\":{}}}",
                    json_escape(name),
                    json_str_array(caps)
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "{{\"schema\":\"{}\",\"aggregate\":{},\"functions\":[{}],\"wildcard\":{}}}",
            json_escape(&self.schema),
            aggregate,
            functions,
            self.surface.has_wildcard
        )
    }
}

/// Merge per-program surfaces into a single package surface: union aggregate,
/// sorted + deduplicated `(name, caps)` functions, OR-ed wildcard.
pub fn merge_surfaces(surfaces: Vec<CapabilitySurface>) -> CapabilitySurface {
    let mut aggregate: BTreeSet<String> = BTreeSet::new();
    let mut functions: BTreeSet<(String, Vec<String>)> = BTreeSet::new();
    let mut has_wildcard = false;
    for surface in surfaces {
        aggregate.extend(surface.aggregate);
        for entry in surface.per_function {
            functions.insert(entry);
        }
        has_wildcard |= surface.has_wildcard;
    }
    CapabilitySurface {
        aggregate: aggregate.into_iter().collect(),
        per_function: functions.into_iter().collect(),
        has_wildcard,
    }
}

/// Build the merged capability surface for a path — a `.garnet` file or every
/// `.garnet` under a directory — edition-aware (S32). The shared entry used by
/// `garnet caps`, `garnet diff-caps`, and `garnet verify --caps-baseline`.
/// Returns a usage / parse / IO error message on failure.
pub fn surface_for_path(path: &Path) -> Result<CapabilitySurface, String> {
    let targets = collect_targets(path).map_err(|e| e.to_string())?;
    if targets.is_empty() {
        return Err(format!("no .garnet files found under {}", path.display()));
    }
    let mut surfaces = Vec::with_capacity(targets.len());
    for target in &targets {
        let src = read_file(target)?;
        let edition = match edition_manifest::resolve_edition_for(target) {
            Ok(resolved) => {
                if let Some(warning) = resolved.warning {
                    eprintln!("{warning}");
                }
                resolved.edition
            }
            Err(message) => return Err(message),
        };
        let module = garnet_parser::parse_source_with_edition(&src, edition)
            .map_err(|e| format!("parse error in {}: {e}", target.display()))?;
        surfaces.push(capability_surface(&module));
    }
    Ok(if surfaces.len() == 1 {
        surfaces.pop().expect("one surface")
    } else {
        merge_surfaces(surfaces)
    })
}

/// Render a slice of strings as a JSON array of escaped strings.
fn json_str_array(items: &[String]) -> String {
    let inner = items
        .iter()
        .map(|s| format!("\"{}\"", json_escape(s)))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{inner}]")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn surface(
        aggregate: &[&str],
        per_fn: &[(&str, &[&str])],
        wildcard: bool,
    ) -> CapabilitySurface {
        CapabilitySurface {
            aggregate: aggregate.iter().map(|s| s.to_string()).collect(),
            per_function: per_fn
                .iter()
                .map(|(n, caps)| (n.to_string(), caps.iter().map(|c| c.to_string()).collect()))
                .collect(),
            has_wildcard: wildcard,
        }
    }

    #[test]
    fn manifest_json_shape_and_schema() {
        let m = CapabilityManifest::from_surface(surface(
            &["fs", "net"],
            &[("main", &[]), ("reader", &["fs"])],
            false,
        ));
        let json = m.to_json();
        assert!(json.contains(r#""schema":"garnet-capability-manifest-v1""#));
        assert!(json.contains(r#""aggregate":["fs","net"]"#), "{json}");
        assert!(json.contains(r#"{"name":"main","caps":[]}"#), "{json}");
        assert!(
            json.contains(r#"{"name":"reader","caps":["fs"]}"#),
            "{json}"
        );
        assert!(json.contains(r#""wildcard":false"#));
    }

    #[test]
    fn manifest_json_is_deterministic() {
        let m = CapabilityManifest::from_surface(surface(&["net", "fs"], &[("a", &["fs"])], true));
        assert_eq!(m.to_json(), m.to_json());
        assert!(m.to_json().contains(r#""wildcard":true"#));
    }

    #[test]
    fn merge_unions_and_dedups() {
        let merged = merge_surfaces(vec![
            surface(&["fs"], &[("a", &["fs"])], false),
            surface(&["net"], &[("b", &["net"])], true),
            surface(&["fs"], &[("a", &["fs"])], false), // duplicate of the first
        ]);
        assert_eq!(merged.aggregate, vec!["fs", "net"]);
        // (a,[fs]) deduped; (b,[net]) kept; sorted by name.
        assert_eq!(merged.per_function.len(), 2);
        assert_eq!(merged.per_function[0].0, "a");
        assert_eq!(merged.per_function[1].0, "b");
        assert!(merged.has_wildcard);
    }

    #[test]
    fn json_escapes_unusual_cap_names() {
        // A user-defined Capability::Other could carry odd characters.
        let m = CapabilityManifest::from_surface(surface(&["wei\"rd"], &[], false));
        assert!(m.to_json().contains(r#"\"rd"#), "{}", m.to_json());
    }
}
