//! `std::log` — leveled log-line formatting + file sink (Layer 1).
//!
//! [`info`]/[`warn`]/[`error`]/[`debug`] are the pure *formatting* surface (no
//! caps): each returns a `[LEVEL] message` line. [`to_file`] (S23 → S24) is the
//! **file sink**: it formats the same line and appends it to a file, so it
//! requires `@caps(fs)`. `@stability(experimental)`.

use crate::StdError;
use std::io::Write;

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

/// File sink: format `[level] message` and **append** it (plus a newline) to
/// `path`, creating the file if it does not exist. Returns the formatted line
/// (same value [`info`] et al. return) so a caller can both persist and use it.
/// Requires `@caps(fs)`. IO failures surface as [`StdError::Io`].
pub fn to_file(path: &str, level: &str, message: &str) -> Result<String, StdError> {
    let rendered = line(level, message);
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| StdError::Io(format!("log to_file open `{path}`: {e}")))?;
    writeln!(file, "{rendered}")
        .map_err(|e| StdError::Io(format!("log to_file write `{path}`: {e}")))?;
    Ok(rendered)
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

    fn unique_temp_path(tag: &str) -> std::path::PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "garnet_s24_{tag}_{}_{nanos}.log",
            std::process::id()
        ))
    }

    #[test]
    fn to_file_appends_formatted_lines_in_order() {
        let path = unique_temp_path("append");
        let p = path.to_str().unwrap();

        assert_eq!(to_file(p, "INFO", "first").unwrap(), "[INFO] first");
        to_file(p, "WARN", "second").unwrap();
        to_file(p, "ERROR", "third").unwrap();

        let contents = std::fs::read_to_string(&path).unwrap();
        assert_eq!(contents, "[INFO] first\n[WARN] second\n[ERROR] third\n");

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn to_file_does_not_truncate_existing_content() {
        let path = unique_temp_path("notrunc");
        let p = path.to_str().unwrap();
        std::fs::write(&path, "preexisting\n").unwrap();

        to_file(p, "INFO", "added").unwrap();

        let contents = std::fs::read_to_string(&path).unwrap();
        assert_eq!(contents, "preexisting\n[INFO] added\n");

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn to_file_errors_on_unwritable_path() {
        // A directory cannot be opened for writing as a file.
        let dir = std::env::temp_dir();
        match to_file(dir.to_str().unwrap(), "INFO", "x") {
            Err(StdError::Io(_)) => {}
            other => panic!("expected Io error writing to a directory, got {other:?}"),
        }
    }
}
