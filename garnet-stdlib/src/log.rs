//! `std::log` — leveled log-line formatting (Layer 1, no caps).
//!
//! v0.7 ships the *formatting* surface only: each function returns a
//! formatted `[LEVEL] message` line. Routing those lines to a file sink
//! needs `@caps(fs)` and is deferred to v0.8 (labeled in the Layer Policy
//! doc). `@stability(experimental)`.

fn line(level: &str, message: &str) -> String {
    format!("[{level}] {message}")
}

pub fn info(message: &str) -> String {
    line("INFO", message)
}

pub fn warn(message: &str) -> String {
    line("WARN", message)
}

pub fn error(message: &str) -> String {
    line("ERROR", message)
}

pub fn debug(message: &str) -> String {
    line("DEBUG", message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levels_format_with_prefix() {
        assert_eq!(info("up"), "[INFO] up");
        assert_eq!(warn("slow"), "[WARN] slow");
        assert_eq!(error("boom"), "[ERROR] boom");
        assert_eq!(debug("trace"), "[DEBUG] trace");
    }

    #[test]
    fn empty_message_still_has_level() {
        assert_eq!(info(""), "[INFO] ");
    }

    #[test]
    fn message_body_is_preserved_verbatim() {
        assert_eq!(warn("a [bracketed] thing"), "[WARN] a [bracketed] thing");
    }
}
