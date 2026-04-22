//! Memory declaration parser tests (Mini-Spec §4).

use garnet_parser::ast::{Item, MemoryKind, TypeExpr};
use garnet_parser::parse_source;

#[test]
fn parses_working_memory_unit() {
    let m = parse_source("memory working scratch : Buffer").unwrap();
    assert_eq!(m.items.len(), 1);
    match &m.items[0] {
        Item::Memory(md) => {
            assert_eq!(md.kind, MemoryKind::Working);
            assert_eq!(md.name, "scratch");
            match &md.store {
                TypeExpr::Named { path, args, .. } => {
                    assert_eq!(path, &vec!["Buffer".to_string()]);
                    assert!(args.is_empty());
                }
                _ => panic!("expected named type"),
            }
        }
        _ => panic!("expected memory decl"),
    }
}

#[test]
fn parses_episodic_memory_unit() {
    let m = parse_source("memory episodic session_log : EpisodeStore<Interaction>").unwrap();
    match &m.items[0] {
        Item::Memory(md) => {
            assert_eq!(md.kind, MemoryKind::Episodic);
            assert_eq!(md.name, "session_log");
            match &md.store {
                TypeExpr::Named { path, args, .. } => {
                    assert_eq!(path, &vec!["EpisodeStore".to_string()]);
                    assert_eq!(args.len(), 1);
                }
                _ => panic!("expected named type"),
            }
        }
        _ => panic!("expected memory decl"),
    }
}

#[test]
fn parses_semantic_with_nested_generics() {
    let m =
        parse_source("memory semantic embeddings : Map<String, Vector<Embedding>>").unwrap();
    match &m.items[0] {
        Item::Memory(md) => {
            assert_eq!(md.kind, MemoryKind::Semantic);
            match &md.store {
                TypeExpr::Named { path, args, .. } => {
                    assert_eq!(path, &vec!["Map".to_string()]);
                    assert_eq!(args.len(), 2);
                    // Second arg must itself be generic
                    match &args[1] {
                        TypeExpr::Named { args: inner, .. } => assert_eq!(inner.len(), 1),
                        _ => panic!("expected nested generic"),
                    }
                }
                _ => panic!("expected named type"),
            }
        }
        _ => panic!("expected memory decl"),
    }
}

#[test]
fn parses_procedural_memory_unit() {
    let m = parse_source("memory procedural workflows : WorkflowStore<Trace>").unwrap();
    match &m.items[0] {
        Item::Memory(md) => {
            assert_eq!(md.kind, MemoryKind::Procedural);
        }
        _ => panic!("expected memory decl"),
    }
}

// ── Error paths ──

#[test]
fn errors_on_missing_kind() {
    assert!(parse_source("memory foobar : Buffer").is_err());
}

#[test]
fn errors_on_missing_colon() {
    assert!(parse_source("memory working scratch Buffer").is_err());
}

#[test]
fn errors_on_missing_type() {
    assert!(parse_source("memory working scratch :").is_err());
}

#[test]
fn errors_on_bad_kind_keyword() {
    assert!(parse_source("memory ephemeral log : Store").is_err());
}
