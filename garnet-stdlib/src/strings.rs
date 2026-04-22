//! String operations (no caps required — pure compute).

use crate::StdError;

pub fn split(s: &str, delim: &str) -> Vec<String> {
    if delim.is_empty() {
        // Split into individual Unicode graphemes (best-effort chars here).
        return s.chars().map(|c| c.to_string()).collect();
    }
    s.split(delim).map(|part| part.to_string()).collect()
}

pub fn replace(s: &str, old: &str, new: &str) -> Result<String, StdError> {
    if old.is_empty() {
        return Err(StdError::InvalidInput(
            "replace: `old` substring must be non-empty".into(),
        ));
    }
    Ok(s.replace(old, new))
}

pub fn to_lower(s: &str) -> String {
    s.to_lowercase()
}

pub fn to_upper(s: &str) -> String {
    s.to_uppercase()
}

pub fn trim(s: &str) -> String {
    s.trim().to_string()
}

pub fn starts_with(s: &str, prefix: &str) -> bool {
    s.starts_with(prefix)
}

pub fn ends_with(s: &str, suffix: &str) -> bool {
    s.ends_with(suffix)
}

pub fn contains(s: &str, needle: &str) -> bool {
    s.contains(needle)
}

pub fn chars(s: &str) -> Vec<char> {
    s.chars().collect()
}

pub fn len_chars(s: &str) -> usize {
    s.chars().count()
}

pub fn len_bytes(s: &str) -> usize {
    s.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_basic() {
        assert_eq!(split("a,b,c", ","), vec!["a", "b", "c"]);
        assert_eq!(split("abc", ","), vec!["abc"]);
        assert_eq!(split("", ","), vec![""]);
    }

    #[test]
    fn split_empty_delim_gives_chars() {
        assert_eq!(split("abc", ""), vec!["a", "b", "c"]);
    }

    #[test]
    fn replace_basic() {
        assert_eq!(replace("hello world", "world", "there").unwrap(), "hello there");
        assert_eq!(replace("aaaa", "a", "b").unwrap(), "bbbb");
    }

    #[test]
    fn replace_empty_old_is_invalid() {
        match replace("hello", "", "x") {
            Err(StdError::InvalidInput(_)) => {}
            other => panic!("expected InvalidInput, got {other:?}"),
        }
    }

    #[test]
    fn case_conversion() {
        assert_eq!(to_lower("Hello"), "hello");
        assert_eq!(to_upper("hello"), "HELLO");
        // Unicode-aware
        assert_eq!(to_upper("straße"), "STRASSE");
    }

    #[test]
    fn trim_ops() {
        assert_eq!(trim("  hello  "), "hello");
        assert_eq!(trim("\t\nhi\r\n"), "hi");
    }

    #[test]
    fn prefix_suffix_contains() {
        assert!(starts_with("garnet", "gar"));
        assert!(!starts_with("garnet", "net"));
        assert!(ends_with("garnet", "net"));
        assert!(contains("garnet", "rne"));
        assert!(!contains("garnet", "xyz"));
    }

    #[test]
    fn char_vs_byte_length() {
        assert_eq!(len_chars("hello"), 5);
        assert_eq!(len_bytes("hello"), 5);
        // Multi-byte
        assert_eq!(len_chars("héllo"), 5);
        assert_eq!(len_bytes("héllo"), 6); // é is 2 bytes UTF-8
    }
}
