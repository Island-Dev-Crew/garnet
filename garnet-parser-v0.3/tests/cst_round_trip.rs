//! CST round-trip integration tests for byte-identical reconstruction of all examples.

use garnet_parser::parse_source_cst;
use std::fs;
use std::path::Path;

fn check_file(path: &Path) {
    let src = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", path.display(), e));

    let (_module, cst_root) = parse_source_cst(&src)
        .unwrap_or_else(|e| panic!("failed to parse {}: {:?}", path.display(), e));

    let round_trip = cst_root.to_string(&src);

    assert_eq!(
        src,
        round_trip,
        "round-trip mismatch in file: {}",
        path.display()
    );
}

#[test]
fn test_parser_examples_round_trip() {
    let dir = Path::new("examples");
    let mut files_tested = 0;
    if dir.is_dir() {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "garnet") {
                check_file(&path);
                files_tested += 1;
            }
        }
    }
    assert!(files_tested > 0, "expected to find parser example files");
}

#[test]
fn test_workspace_examples_round_trip() {
    // Relative to parser crate directory (current dir during cargo test of the package)
    let dir = Path::new("../examples");
    let mut files_tested = 0;
    if dir.is_dir() {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "garnet") {
                check_file(&path);
                files_tested += 1;
            }
        }
    }
    assert!(files_tested > 0, "expected to find workspace example files");
}
