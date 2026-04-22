//! Actor parser tests (Mini-Spec §9).

use garnet_parser::ast::{ActorItem, Item, MemoryKind};
use garnet_parser::parse_source;

#[test]
fn parses_minimal_actor() {
    let src = r#"
        actor Greeter {
            protocol hello(name: String) -> String
            on hello(name) { name }
        }
    "#;
    let m = parse_source(src).unwrap();
    match &m.items[0] {
        Item::Actor(a) => {
            assert_eq!(a.name, "Greeter");
            assert_eq!(a.items.len(), 2);
        }
        _ => panic!("expected actor"),
    }
}

#[test]
fn parses_public_actor() {
    let src = r#"
        pub actor Worker {
            protocol ping() -> Int
            on ping() { 1 }
        }
    "#;
    let m = parse_source(src).unwrap();
    match &m.items[0] {
        Item::Actor(a) => assert!(a.public),
        _ => panic!("expected actor"),
    }
}

#[test]
fn parses_actor_with_multiple_protocols() {
    let src = r#"
        actor BuildAgent {
            protocol build(spec: BuildSpec) -> BuildResult
            protocol status() -> AgentStatus
            on build(spec) { spec }
            on status() { 0 }
        }
    "#;
    let m = parse_source(src).unwrap();
    match &m.items[0] {
        Item::Actor(a) => {
            let protocols = a
                .items
                .iter()
                .filter(|it| matches!(it, ActorItem::Protocol(_)))
                .count();
            let handlers = a
                .items
                .iter()
                .filter(|it| matches!(it, ActorItem::Handler(_)))
                .count();
            assert_eq!(protocols, 2);
            assert_eq!(handlers, 2);
        }
        _ => panic!("expected actor"),
    }
}

#[test]
fn parses_actor_with_memory_unit() {
    let src = r#"
        actor Logger {
            memory episodic log : EpisodeStore<Event>
            protocol append(e: Event) -> Int
            on append(e) { log.push(e) }
        }
    "#;
    let m = parse_source(src).unwrap();
    match &m.items[0] {
        Item::Actor(a) => {
            let mem_items = a
                .items
                .iter()
                .filter_map(|it| match it {
                    ActorItem::Memory(md) => Some(md.kind),
                    _ => None,
                })
                .collect::<Vec<_>>();
            assert_eq!(mem_items, vec![MemoryKind::Episodic]);
        }
        _ => panic!("expected actor"),
    }
}

#[test]
fn parses_actor_with_let_binding() {
    let src = r#"
        actor Counter {
            let mut count = 0
            protocol tick() -> Int
            on tick() { count }
        }
    "#;
    let m = parse_source(src).unwrap();
    match &m.items[0] {
        Item::Actor(a) => {
            let lets = a
                .items
                .iter()
                .filter(|it| matches!(it, ActorItem::Let(_)))
                .count();
            assert_eq!(lets, 1);
        }
        _ => panic!("expected actor"),
    }
}

#[test]
fn parses_handler_with_string_interpolation() {
    let src = r#"
        actor Greeter {
            protocol hello(name: String) -> String
            on hello(name) {
                let greeting = "Hello, #{name}!"
                greeting
            }
        }
    "#;
    parse_source(src).unwrap();
}

#[test]
fn parses_protocol_without_return_type() {
    let src = r#"
        actor FireAndForget {
            protocol signal(payload: Bytes)
            on signal(payload) { payload }
        }
    "#;
    parse_source(src).unwrap();
}

#[test]
fn parses_empty_actor() {
    // Structurally valid per grammar even if semantically useless
    let src = "actor Empty { }";
    let m = parse_source(src).unwrap();
    match &m.items[0] {
        Item::Actor(a) => assert_eq!(a.items.len(), 0),
        _ => panic!("expected actor"),
    }
}

#[test]
fn parses_handler_with_annotations() {
    let src = r#"
        actor Analyzer {
            protocol analyze(doc: Doc) -> Result
            on analyze(doc) { doc }
        }
    "#;
    parse_source(src).unwrap();
}

// ── Error paths ──

#[test]
fn errors_on_unclosed_actor() {
    assert!(parse_source("actor Bad { protocol foo() -> Int ").is_err());
}

#[test]
fn errors_on_protocol_without_return_arrow_and_paren() {
    // An entirely malformed protocol declaration
    assert!(parse_source("actor Bad { protocol oops }").is_err());
}

#[test]
fn errors_on_handler_missing_block() {
    assert!(parse_source("actor Bad { on ping }").is_err());
}
