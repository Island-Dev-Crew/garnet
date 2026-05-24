//! `std::env` — process environment access (Layer 1, cap: `env`).
//!
//! The `env` capability is new in S17 (v0.7); the matching known-capability
//! entry lives in `garnet-check-v0.3`, so a Garnet function calling these
//! must declare `@caps(env)`. Host helpers over `std::env`.
//! `@stability(experimental)`.

/// Read an environment variable; `None` if unset or not valid Unicode.
pub fn get(key: &str) -> Option<String> {
    std::env::var(key).ok()
}

/// Set an environment variable for this process.
pub fn set(key: &str, value: &str) {
    std::env::set_var(key, value);
}

/// Snapshot all environment variables as `(key, value)` pairs.
pub fn vars() -> Vec<(String, String)> {
    std::env::vars().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Distinct keys per test: env is process-global and `cargo test` runs
    // tests on parallel threads, so sharing a key would race.

    #[test]
    fn set_then_get_roundtrips() {
        let key = "GARNET_S17_ENV_TEST_RT";
        assert_eq!(get(key), None);
        set(key, "value-42");
        assert_eq!(get(key), Some("value-42".to_string()));
    }

    #[test]
    fn get_unset_is_none() {
        assert_eq!(get("GARNET_S17_DEFINITELY_UNSET_KEY_XYZ"), None);
    }

    #[test]
    fn vars_contains_a_set_var() {
        let key = "GARNET_S17_ENV_TEST_VARS";
        set(key, "present");
        let all = vars();
        assert!(all.iter().any(|(k, v)| k == key && v == "present"));
    }
}
