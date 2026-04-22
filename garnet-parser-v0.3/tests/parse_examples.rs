//! Integration tests — round-trip parse each example file from disk.

use garnet_parser::parse_source;

fn read_example(name: &str) -> String {
    let path = format!("examples/{}", name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", path, e))
}

#[test]
fn parses_memory_units_example() {
    let src = read_example("memory_units.garnet");
    let m = parse_source(&src).unwrap();
    assert!(m.items.len() >= 4, "expected at least 4 memory declarations");
}

#[test]
fn parses_greeter_actor_example() {
    let src = read_example("greeter_actor.garnet");
    let m = parse_source(&src).unwrap();
    assert_eq!(m.items.len(), 1, "expected exactly 1 actor");
}

#[test]
fn parses_build_agent_example() {
    let src = read_example("build_agent.garnet");
    let m = parse_source(&src).unwrap();
    assert_eq!(m.items.len(), 1, "expected exactly 1 actor");
}

#[test]
fn parses_safe_module_example() {
    let src = read_example("safe_module.garnet");
    let m = parse_source(&src).unwrap();
    assert!(m.safe, "@safe flag should be set at file level");
    assert!(m.items.len() >= 3, "expected at least enum + struct + fn");
}

#[test]
fn parses_control_flow_example() {
    let src = read_example("control_flow.garnet");
    let m = parse_source(&src).unwrap();
    assert!(
        m.items.len() >= 5,
        "expected multiple function definitions in control flow example"
    );
}

#[test]
fn parses_error_handling_example() {
    let src = read_example("error_handling.garnet");
    let m = parse_source(&src).unwrap();
    assert!(m.items.len() >= 2, "expected at least 2 functions");
}
