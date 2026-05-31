//! S32 Layer 2 — GODEBUG-style runtime settings via the `GARNET_DEBUG` env var.
//!
//! `GARNET_DEBUG=key=value,key2=value2` flips runtime **defaults**. Like Go's
//! `GODEBUG`, this is the semantic-time compatibility layer that sits beside
//! editions (the parse-time layer). Two invariants make it safe:
//!   * **Unknown keys warn, never error** — forward compatibility: an older
//!     binary tolerates settings introduced by a newer one.
//!   * **Settings never change program meaning, the AST, or the capability
//!     manifest.** They only flip a *default* (here: diagnostic verbosity).
//!
//! There is intentionally no manifest `[runtime]` table in v0.8 (that would be
//! a spec change); the env var is the whole surface for now.

/// Parsed `GARNET_DEBUG` settings.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RuntimeSettings {
    /// `diagnostics=verbose` (also accepts `1`/`on`) flips `garnet check` from
    /// its concise summary (the default) to a per-function capability dump.
    /// Genuinely behavioral for the CLI, but cosmetic to the *program*: it
    /// touches neither the AST nor the capability manifest.
    pub verbose_diagnostics: bool,
    /// Keys we did not recognize, retained for a single forward-compat advisory.
    pub unknown_keys: Vec<String>,
}

impl RuntimeSettings {
    /// Read settings from the process environment (`GARNET_DEBUG`). An unset or
    /// empty variable yields defaults.
    pub fn from_env() -> Self {
        match std::env::var("GARNET_DEBUG") {
            Ok(raw) => Self::parse(&raw),
            Err(_) => Self::default(),
        }
    }

    /// Parse a `GARNET_DEBUG` value. Tolerant by design: blank entries are
    /// skipped and unrecognized keys are recorded (for `unknown_key_warning`)
    /// rather than rejected.
    pub fn parse(raw: &str) -> Self {
        let mut settings = Self::default();
        for entry in raw.split(',') {
            let entry = entry.trim();
            if entry.is_empty() {
                continue;
            }
            let (key, value) = match entry.split_once('=') {
                Some((k, v)) => (k.trim(), v.trim()),
                None => (entry, ""),
            };
            match key {
                "diagnostics" => {
                    settings.verbose_diagnostics = matches!(value, "verbose" | "1" | "on")
                }
                _ => settings.unknown_keys.push(key.to_string()),
            }
        }
        settings
    }

    /// A one-line advisory naming any unrecognized keys, or `None` when all keys
    /// were understood. Printing this is never fatal.
    pub fn unknown_key_warning(&self) -> Option<String> {
        if self.unknown_keys.is_empty() {
            return None;
        }
        Some(format!(
            "GARNET_DEBUG: ignoring unknown key(s): {} \
             (forward-compatible — unknown settings never error)",
            self.unknown_keys.join(", ")
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_concise() {
        let s = RuntimeSettings::default();
        assert!(!s.verbose_diagnostics);
        assert!(s.unknown_keys.is_empty());
        assert!(s.unknown_key_warning().is_none());
    }

    #[test]
    fn diagnostics_verbose_flips_the_default() {
        assert!(RuntimeSettings::parse("diagnostics=verbose").verbose_diagnostics);
        assert!(RuntimeSettings::parse("diagnostics=on").verbose_diagnostics);
        assert!(RuntimeSettings::parse("diagnostics=1").verbose_diagnostics);
        assert!(!RuntimeSettings::parse("diagnostics=concise").verbose_diagnostics);
    }

    #[test]
    fn unknown_keys_warn_but_do_not_error() {
        let s = RuntimeSettings::parse("diagnostics=verbose,http2=off,gc=2");
        assert!(s.verbose_diagnostics);
        assert_eq!(s.unknown_keys, vec!["http2".to_string(), "gc".to_string()]);
        let w = s.unknown_key_warning().expect("warning expected");
        assert!(w.contains("http2") && w.contains("gc"));
    }

    #[test]
    fn blank_and_whitespace_entries_are_ignored() {
        let s = RuntimeSettings::parse("  , diagnostics = verbose , ");
        assert!(s.verbose_diagnostics);
        assert!(s.unknown_keys.is_empty());
    }
}
