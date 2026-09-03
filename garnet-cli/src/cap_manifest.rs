//! S36 — the capability manifest.
//!
//! A per-program / per-package artifact derived from S35's
//! [`garnet_check::CapabilitySurface`], serialized as **deterministic JSON**
//! (no `serde`, per the `manifest.rs` determinism stance; reusing
//! `crate::diagnostics::json_escape`). This is the artifact S37 `diff-caps`
//! compares across revisions and S38 `seal` embeds. It is distinct from the
//! build [`crate::manifest::Manifest`], which carries source/AST hashes but no
//! capability surface.

use crate::cmd::verify_gate::{collect_targets_with_omissions, ScanOmissions};
use crate::diagnostics::json_escape;
use crate::{edition_manifest, read_file};
use garnet_check::{capability_surface, CapabilitySurface};
use std::collections::BTreeSet;
use std::path::Path;

/// Schema identifier baked into every capability manifest. Bump when the shape
/// changes; older consumers reject manifests they do not recognize.
pub const SCHEMA: &str = "garnet-capability-manifest-v1";

/// Language-neutral draft profile seeded by S98 for RFC-0001. This does not
/// replace Garnet's S36 schema; it is an export/profile that other toolchains
/// can implement against while the standard remains a draft.
pub const STANDARD_SCHEMA: &str = "capability-manifest/v1";

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

    /// Deterministic language-neutral draft profile for RFC-0001.
    ///
    /// The profile intentionally exports only the declared capability surface
    /// available today. `source_span` is `null` until the CST/span migration can
    /// provide stable source coordinates without inventing precision.
    pub fn to_standard_profile_json(&self) -> String {
        let aggregate = json_str_array(&self.surface.aggregate);
        let entries = self
            .surface
            .per_function
            .iter()
            .map(|(name, caps)| {
                format!(
                    "{{\"kind\":\"function\",\"name\":\"{}\",\"capabilities\":{},\"source_span\":null}}",
                    json_escape(name),
                    json_str_array(caps)
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let limitations = json_str_array(&[
            "declared-surface only; does not prove absence of undeclared authority".to_string(),
            "draft/reference seed; no OWASP/LF adoption claimed".to_string(),
        ]);
        format!(
            "{{\"schema\":\"{}\",\"status\":\"draft-reference-seed\",\
             \"producer\":{{\"name\":\"garnet\",\"manifest_schema\":\"{}\"}},\
             \"surface\":{{\"aggregate\":{},\"entries\":[{}],\"wildcard\":{}}},\
             \"limitations\":{}}}",
            json_escape(STANDARD_SCHEMA),
            json_escape(&self.schema),
            aggregate,
            entries,
            self.surface.has_wildcard,
            limitations
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
    Ok(surface_for_path_with_omissions(path)?.0)
}

/// [`surface_for_path`] plus the tally of directories the walk refused to read.
///
/// Crown C B-1: a caller that GATES on this surface must disclose the tally. A
/// `.garnet` file under a skipped directory declares authority that is simply
/// unread — which is not the same claim as the diff-caps `scope` string, and
/// is not covered by it: `scope` disclaims *undeclared* authority.
pub fn surface_for_path_with_omissions(
    path: &Path,
) -> Result<(CapabilitySurface, ScanOmissions), String> {
    let (targets, omissions) = collect_targets_with_omissions(path).map_err(|e| e.to_string())?;
    if targets.is_empty() {
        return Err(format!("no .garnet files found under {}", path.display()));
    }
    let mut surfaces = Vec::with_capacity(targets.len());
    for target in &targets {
        let src = read_file(target)?;
        let resolved = edition_manifest::resolve_edition_for(target)?;
        if let Some(warning) = resolved.warning {
            eprintln!("{warning}");
        }
        let edition = resolved.edition;
        let module = garnet_parser::parse_source_with_edition(&src, edition)
            .map_err(|e| format!("parse error in {}: {e}", target.display()))?;
        surfaces.push(capability_surface(&module));
    }
    if surfaces.len() == 1 {
        #[allow(clippy::expect_used)]
        // INVARIANT: guarded by the len() == 1 check on the previous line —
        // pop() on a one-element Vec cannot return None.
        let only = surfaces.pop().expect("one surface");
        return Ok((only, omissions));
    }
    Ok((merge_surfaces(surfaces), omissions))
}

/// Render a slice of strings as a JSON array of escaped strings.
pub(crate) fn json_str_array(items: &[String]) -> String {
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

    #[test]
    fn standard_profile_keeps_draft_scope_and_declared_surface() {
        let m = CapabilityManifest::from_surface(surface(&["fs"], &[("main", &["fs"])], false));
        let json = m.to_standard_profile_json();
        assert!(json.contains(r#""schema":"capability-manifest/v1""#));
        assert!(json.contains(r#""status":"draft-reference-seed""#));
        assert!(json.contains(r#""manifest_schema":"garnet-capability-manifest-v1""#));
        assert!(json.contains(r#""source_span":null"#));
        assert!(json.contains("does not prove absence of undeclared authority"));
    }
}
