//! File I/O primitives (cap: `fs`). Mini-Spec v1.0 §11.2 + Security V2 spec §1.6.
//!
//! Each function expects the caller to have `@caps(fs)` — enforced by
//! the CapCaps checker at the source layer. The Rust implementations
//! here perform NO cap check of their own; they trust the checker.
//!
//! Behaviour is UTF-8-strict for the string-returning variants; see
//! [`read_bytes`] / [`write_bytes`] for raw binary I/O.

use crate::StdError;
use std::fs;
use std::path::Path;

pub fn read_file<P: AsRef<Path>>(path: P) -> Result<String, StdError> {
    fs::read_to_string(path.as_ref())
        .map_err(|e| StdError::Io(format!("read_file({}): {e}", path.as_ref().display())))
}

pub fn write_file<P: AsRef<Path>>(path: P, contents: &str) -> Result<(), StdError> {
    fs::write(path.as_ref(), contents)
        .map_err(|e| StdError::Io(format!("write_file({}): {e}", path.as_ref().display())))
}

pub fn read_bytes<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, StdError> {
    fs::read(path.as_ref())
        .map_err(|e| StdError::Io(format!("read_bytes({}): {e}", path.as_ref().display())))
}

pub fn write_bytes<P: AsRef<Path>>(path: P, data: &[u8]) -> Result<(), StdError> {
    fs::write(path.as_ref(), data)
        .map_err(|e| StdError::Io(format!("write_bytes({}): {e}", path.as_ref().display())))
}

pub fn list_dir<P: AsRef<Path>>(path: P) -> Result<Vec<String>, StdError> {
    let entries = fs::read_dir(path.as_ref())
        .map_err(|e| StdError::Io(format!("list_dir({}): {e}", path.as_ref().display())))?;
    let mut names = Vec::new();
    for ent in entries {
        let ent = ent.map_err(StdError::from)?;
        names.push(ent.file_name().to_string_lossy().into_owned());
    }
    names.sort();
    Ok(names)
}

pub fn exists<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().exists()
}

pub fn remove_file<P: AsRef<Path>>(path: P) -> Result<(), StdError> {
    fs::remove_file(path.as_ref())
        .map_err(|e| StdError::Io(format!("remove_file({}): {e}", path.as_ref().display())))
}

pub fn create_dir_all<P: AsRef<Path>>(path: P) -> Result<(), StdError> {
    fs::create_dir_all(path.as_ref())
        .map_err(|e| StdError::Io(format!("create_dir_all({}): {e}", path.as_ref().display())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn roundtrip_string() {
        let d = tempdir().unwrap();
        let p = d.path().join("hello.txt");
        write_file(&p, "garnet v1.0").unwrap();
        assert_eq!(read_file(&p).unwrap(), "garnet v1.0");
    }

    #[test]
    fn roundtrip_bytes() {
        let d = tempdir().unwrap();
        let p = d.path().join("data.bin");
        let payload = &[0u8, 1, 2, 3, 254, 255];
        write_bytes(&p, payload).unwrap();
        assert_eq!(read_bytes(&p).unwrap(), payload);
    }

    #[test]
    fn list_dir_sorted() {
        let d = tempdir().unwrap();
        write_file(d.path().join("b.txt"), "").unwrap();
        write_file(d.path().join("a.txt"), "").unwrap();
        write_file(d.path().join("c.txt"), "").unwrap();
        let entries = list_dir(d.path()).unwrap();
        assert_eq!(entries, vec!["a.txt", "b.txt", "c.txt"]);
    }

    #[test]
    fn read_missing_file_returns_io_error() {
        match read_file("definitely/does/not/exist") {
            Err(StdError::Io(_)) => {}
            other => panic!("expected Io error, got {other:?}"),
        }
    }

    #[test]
    fn remove_file_works() {
        let d = tempdir().unwrap();
        let p = d.path().join("rm.txt");
        write_file(&p, "bye").unwrap();
        assert!(exists(&p));
        remove_file(&p).unwrap();
        assert!(!exists(&p));
    }

    #[test]
    fn create_nested_dirs() {
        let d = tempdir().unwrap();
        let nested = d.path().join("a/b/c/d");
        create_dir_all(&nested).unwrap();
        assert!(nested.exists());
    }
}
