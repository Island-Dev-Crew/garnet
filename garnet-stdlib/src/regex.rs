//! `std::regex` — regular expressions (Layer 1, no caps).
//!
//! Thin host wrapper over the `regex` crate. The Garnet surface name
//! `std::regex::match` maps to [`is_match`] (`match` is a Rust keyword).
//! `@stability(experimental)`.

use crate::StdError;

fn compile_re(pattern: &str) -> Result<::regex::Regex, StdError> {
    ::regex::Regex::new(pattern)
        .map_err(|e| StdError::InvalidInput(format!("regex compile error: {e}")))
}

/// Validate/compile a pattern. Errors on bad syntax; `Ok(())` otherwise.
pub fn compile(pattern: &str) -> Result<(), StdError> {
    compile_re(pattern).map(|_| ())
}

/// True if the pattern matches anywhere in `input`. (`std::regex::match`.)
pub fn is_match(pattern: &str, input: &str) -> Result<bool, StdError> {
    Ok(compile_re(pattern)?.is_match(input))
}

/// All non-overlapping matches of the pattern, left to right.
pub fn find_all(pattern: &str, input: &str) -> Result<Vec<String>, StdError> {
    let re = compile_re(pattern)?;
    Ok(re
        .find_iter(input)
        .map(|m| m.as_str().to_string())
        .collect())
}

/// Replace all matches of the pattern with `replacement`.
pub fn replace(pattern: &str, input: &str, replacement: &str) -> Result<String, StdError> {
    let re = compile_re(pattern)?;
    Ok(re.replace_all(input, replacement).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_accepts_valid_rejects_invalid() {
        assert!(compile(r"\d+").is_ok());
        match compile(r"(unclosed") {
            Err(StdError::InvalidInput(_)) => {}
            other => panic!("expected InvalidInput, got {other:?}"),
        }
    }

    #[test]
    fn match_detects_presence() {
        assert!(is_match(r"\d{3}", "abc123").unwrap());
        assert!(!is_match(r"\d{3}", "ab12").unwrap());
        assert!(is_match(r"^gar", "garnet").unwrap());
    }

    #[test]
    fn find_all_returns_every_match() {
        assert_eq!(
            find_all(r"\d+", "a1b22c333").unwrap(),
            vec!["1", "22", "333"]
        );
        assert!(find_all(r"\d+", "no digits").unwrap().is_empty());
    }

    #[test]
    fn replace_substitutes_all() {
        assert_eq!(replace(r"\s+", "a  b   c", "_").unwrap(), "a_b_c");
        // capture-group reference
        assert_eq!(
            replace(r"(\w+)@(\w+)", "user@host", "$2.$1").unwrap(),
            "host.user"
        );
    }

    #[test]
    fn invalid_pattern_propagates_through_ops() {
        assert!(is_match(r"[", "x").is_err());
        assert!(find_all(r"[", "x").is_err());
        assert!(replace(r"[", "x", "y").is_err());
    }
}
