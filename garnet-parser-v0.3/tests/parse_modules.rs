//! Module and use-declaration parser tests (Mini-Spec §3).

use garnet_parser::ast::{Item, UseImports};
use garnet_parser::parse_source;

#[test]
fn parses_use_simple_module() {
    let m = parse_source("use Memory::Core").unwrap();
    match &m.items[0] {
        Item::Use(ud) => {
            assert_eq!(ud.path, vec!["Memory", "Core"]);
            assert!(matches!(ud.imports, UseImports::Module));
        }
        _ => panic!("expected use"),
    }
}

#[test]
fn parses_use_named_imports() {
    let m = parse_source("use Memory::Core::{Store, Index}").unwrap();
    match &m.items[0] {
        Item::Use(ud) => {
            assert_eq!(ud.path, vec!["Memory", "Core"]);
            match &ud.imports {
                UseImports::Named(names) => {
                    assert_eq!(names, &vec!["Store".to_string(), "Index".to_string()]);
                }
                _ => panic!("expected named imports"),
            }
        }
        _ => panic!("expected use"),
    }
}

#[test]
fn parses_use_glob_import() {
    let m = parse_source("use Memory::Core::*").unwrap();
    match &m.items[0] {
        Item::Use(ud) => {
            assert_eq!(ud.path, vec!["Memory", "Core"]);
            assert!(matches!(ud.imports, UseImports::Glob));
        }
        _ => panic!("expected use"),
    }
}

#[test]
fn parses_inline_module() {
    let m = parse_source("module Helpers { def greet(name) { name } }").unwrap();
    match &m.items[0] {
        Item::Module(md) => {
            assert_eq!(md.name, "Helpers");
            assert!(!md.safe);
            assert_eq!(md.items.len(), 1);
        }
        _ => panic!("expected module"),
    }
}

#[test]
fn parses_public_module() {
    let m = parse_source("pub module Api { def endpoint() { 0 } }").unwrap();
    match &m.items[0] {
        Item::Module(md) => {
            assert!(md.public);
        }
        _ => panic!("expected module"),
    }
}

#[test]
fn parses_safe_module_annotation() {
    let m = parse_source("@safe module Crypto { fn hash(own d: Bytes) -> Hash { d.hash() } }")
        .unwrap();
    match &m.items[0] {
        Item::Module(md) => {
            assert!(md.safe);
        }
        _ => panic!("expected module"),
    }
}

#[test]
fn parses_file_level_safe_annotation() {
    let m = parse_source("@safe\n\ndef hash(data) { data.hash() }").unwrap();
    assert!(m.safe);
}

// ── Error paths ──

#[test]
fn errors_on_unclosed_module() {
    assert!(parse_source("module Foo { def bar() { 0 }").is_err());
}

#[test]
fn errors_on_malformed_use() {
    assert!(parse_source("use ::Foo").is_err());
}

#[test]
fn errors_on_use_missing_path() {
    assert!(parse_source("use").is_err());
}
