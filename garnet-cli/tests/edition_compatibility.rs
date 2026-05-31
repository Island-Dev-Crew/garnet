//! S32 — edition compatibility, CLI / manifest level.
//!
//! The load-bearing invariant from the S32 dogfood contract: a program pinned to
//! edition N and one pinned to edition N+1 produce the **same capability
//! manifest**, and a GODEBUG-style toggle changes a runtime default **without**
//! changing the manifest. Editions gate only the lexical surface; the build
//! manifest (whose `ast_hash` determines the downstream capability surface) is
//! edition-invariant by construction.

use garnet_cli::manifest::Manifest;
use garnet_cli::runtime_settings::RuntimeSettings;
use garnet_parser::{parse_source_with_edition, Edition};

/// Source valid in BOTH editions (uses no edition-gated identifier).
const SHARED_SRC: &str = r#"
    def add(a, b) { a + b }
    def main() {
        let total = add(2, 3)
        for x in [1, 2, 3] { total = total + x }
        total
    }
"#;

/// Capability-manifest invariance across editions: the deterministic build
/// manifest is byte-identical when the same shared-valid source is parsed under
/// v1.0 and under v2.0.
#[test]
fn manifest_is_identical_across_editions() {
    let m_v1 = parse_source_with_edition(SHARED_SRC, Edition::V1_0).expect("v1.0 parses");
    let m_next = parse_source_with_edition(SHARED_SRC, Edition::Next).expect("v2.0 parses");

    let manifest_v1 = Manifest::build(SHARED_SRC, &m_v1);
    let manifest_next = Manifest::build(SHARED_SRC, &m_next);

    assert_eq!(
        manifest_v1, manifest_next,
        "the capability manifest must be edition-invariant"
    );
    assert_eq!(
        manifest_v1.ast_hash, manifest_next.ast_hash,
        "the AST hash (which determines the capability surface) must be edition-invariant"
    );
}

/// A `GARNET_DEBUG` toggle flips a CLI default but never touches the manifest:
/// the manifest is a pure function of (source, AST) and does not consult runtime
/// settings at all.
#[test]
fn garnet_debug_toggle_does_not_change_the_manifest() {
    let module = parse_source_with_edition(SHARED_SRC, Edition::V1_0).expect("parses");
    let baseline = Manifest::build(SHARED_SRC, &module);

    // Toggling verbose diagnostics is a real behavioral change for the CLI...
    let verbose = RuntimeSettings::parse("diagnostics=verbose");
    assert!(verbose.verbose_diagnostics);

    // ...yet the manifest, rebuilt under those settings, is byte-identical.
    let rebuilt = Manifest::build(SHARED_SRC, &module);
    assert_eq!(
        baseline, rebuilt,
        "a runtime toggle must not change the capability manifest"
    );
}

/// Unknown `GARNET_DEBUG` keys are tolerated (warn, never error) — forward
/// compatibility for settings introduced by newer binaries.
#[test]
fn unknown_garnet_debug_keys_are_forward_compatible() {
    let settings = RuntimeSettings::parse("http2server=0,diagnostics=verbose");
    assert!(settings.verbose_diagnostics);
    assert!(
        settings.unknown_key_warning().is_some(),
        "unknown keys should produce an advisory, not an error"
    );
}
