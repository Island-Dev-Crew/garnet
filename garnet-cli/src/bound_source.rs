//! Identity-bound source reads for fail-closed discovery and preload.

use same_file::Handle;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub(crate) struct BoundSource {
    pub(crate) path: PathBuf,
    pub(crate) text: String,
}

/// Open, validate, and read one regular source through a single retained file
/// handle. The path is checked again against a second identity handle before
/// bytes are read, so a file-to-symlink/rename swap is RED. Once the identities
/// match, later path replacement is harmless: bytes come from the already
/// validated handle and are carried forward without reopening the path.
pub(crate) fn read_bound_source(path: &Path) -> io::Result<BoundSource> {
    read_bound_source_with_hook(path, || Ok(()))
}

fn read_bound_source_with_hook<F>(path: &Path, before_identity_check: F) -> io::Result<BoundSource>
where
    F: FnOnce() -> io::Result<()>,
{
    let mut opened = Handle::from_path(path)?;
    before_identity_check()?;

    let current_metadata = std::fs::symlink_metadata(path)?;
    if current_metadata.file_type().is_symlink() || !current_metadata.file_type().is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{} is not a real regular source file", path.display()),
        ));
    }
    let current = Handle::from_path(path)?;
    if opened != current {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{} changed identity during validation", path.display()),
        ));
    }

    let mut text = String::new();
    opened.as_file_mut().read_to_string(&mut text)?;
    Ok(BoundSource {
        path: path.to_path_buf(),
        text,
    })
}

#[cfg(test)]
mod tests {
    use super::{read_bound_source, read_bound_source_with_hook};

    #[test]
    fn validated_bytes_are_returned_from_the_bound_handle() {
        let dir = tempfile::TempDir::new().unwrap();
        let source = dir.path().join("source.garnet");
        std::fs::write(&source, "def original() { 1 }\n").unwrap();

        let bound = read_bound_source(&source).unwrap();
        assert_eq!(bound.path, source);
        assert_eq!(bound.text, "def original() { 1 }\n");
    }

    #[cfg(unix)]
    #[test]
    fn swapping_a_validated_source_to_a_symlink_is_rejected() {
        use std::os::unix::fs::symlink;

        let dir = tempfile::TempDir::new().unwrap();
        let source = dir.path().join("source.garnet");
        let replacement = dir.path().join("replacement.garnet");
        std::fs::write(&source, "def original() { 1 }\n").unwrap();
        std::fs::write(&replacement, "def replacement() { 2 }\n").unwrap();

        let result = read_bound_source_with_hook(&source, || {
            std::fs::remove_file(&source)?;
            symlink(&replacement, &source)
        });

        assert!(result.is_err(), "a deterministic symlink swap must be RED");
    }
}
