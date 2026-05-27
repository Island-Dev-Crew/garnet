//! S22 runtime dispatch proof.
//!
//! These tests exercise the S21-deferred stdlib families from Garnet source,
//! not just by direct Rust calls into `garnet_stdlib`. They are deliberately
//! source-level because S22 is the "can programs actually call this?" slice.

use garnet_interp::{Interpreter, Value};

const NS_DNS: [u8; 16] = [
    0x6b, 0xa7, 0xb8, 0x10, 0x9d, 0xad, 0x11, 0xd1, 0x80, 0xb4, 0x00, 0xc0, 0x4f, 0xd4, 0x30, 0xc8,
];

fn run(src: &str) -> Value {
    let mut interp = Interpreter::new();
    interp.load_source(src).expect("load source");
    interp.call("main", vec![]).expect("call main")
}

fn array_items(value: Value) -> Vec<Value> {
    match value {
        Value::Array(items) => items.borrow().clone(),
        other => panic!("expected Array, got {other:?}"),
    }
}

fn expect_str(value: &Value) -> &str {
    match value {
        Value::Str(s) => s.as_str(),
        other => panic!("expected String, got {other:?}"),
    }
}

fn expect_int(value: &Value) -> i64 {
    match value {
        Value::Int(i) => *i,
        other => panic!("expected Int, got {other:?}"),
    }
}

fn expect_bool(value: &Value) -> bool {
    match value {
        Value::Bool(b) => *b,
        other => panic!("expected Bool, got {other:?}"),
    }
}

#[test]
fn s22_json_regex_uuid_and_log_execute_from_source() {
    let result = run(r#"
        @caps()
        def main() {
          let doc = std::json::parse("{\"name\":\"garnet\",\"score\":7}")
          let name = std::json::get(doc, "name")
          let patched = std::json::set(doc, "stage", "s22")
          let compact = std::json::stringify(patched)
          let hits = std::regex::find_all("\\d+", "a12 b345")
          let replaced = std::regex::replace("\\s+", "a  b   c", "_")
          let ok = std::regex::match("^gar", name)
          let id = std::uuid::new_v5("6ba7b810-9dad-11d1-80b4-00c04fd430c8", "garnet-lang.org")
          let line = std::log::info("stdlib")
          [name, compact, hits.len(), replaced, ok, id, line]
        }
        "#);

    let items = array_items(result);
    assert_eq!(expect_str(&items[0]), "garnet");
    assert!(
        expect_str(&items[1]).contains(r#""stage":"s22""#),
        "json stringify should include patched key: {}",
        expect_str(&items[1])
    );
    assert_eq!(expect_int(&items[2]), 2);
    assert_eq!(expect_str(&items[3]), "a_b_c");
    assert!(expect_bool(&items[4]));
    assert_eq!(
        expect_str(&items[5]),
        garnet_stdlib::uuid::new_v5(&NS_DNS, "garnet-lang.org")
    );
    assert_eq!(expect_str(&items[6]), "[INFO] stdlib");
}

#[test]
fn s22_env_dispatch_roundtrips_with_unique_key() {
    let key = format!("GARNET_S22_ENV_{}", std::process::id());
    std::env::remove_var(&key);
    let src = format!(
        r#"
        @caps(env)
        def main() {{
          std::env::set("{key}", "ready")
          [std::env::get("{key}"), std::env::vars().len()]
        }}
        "#
    );

    let items = array_items(run(&src));
    assert_eq!(expect_str(&items[0]), "ready");
    assert!(
        expect_int(&items[1]) > 0,
        "env vars should expose host entries"
    );
}

#[test]
fn s22_process_dispatch_spawns_waits_and_reports_exit_code() {
    let cmd = if cfg!(windows) {
        "cmd /c exit 0"
    } else {
        "true"
    };
    let src = format!(
        r#"
        @caps(proc)
        def main() {{
          let proc = std::process::spawn("{cmd}")
          let status = std::process::wait(proc)
          std::process::exit_code(status)
        }}
        "#
    );

    assert_eq!(expect_int(&run(&src)), 0);
}

#[test]
fn s22_memory_constructors_return_live_mnemos_handles() {
    let result = run(r#"
        @caps()
        def main() {
          let work_store = memory::working("scratch")
          work_store.push("alpha")
          work_store.push("beta")

          let episode_store = memory::episodic("events")
          episode_store.append(std::log::info("boot"))
          let recent = episode_store.recent(1)

          let semantic_store = memory::semantic("facts")
          semantic_store.insert([1.0, 0.0], "x-axis")
          semantic_store.insert([0.0, 1.0], "y-axis")
          let found = semantic_store.search([1.0, 0.0], 1)

          let procedure_store = memory::procedural("flows")
          procedure_store.register("build", "compile")

          [work_store.len(), recent.len(), found.len(), procedure_store.find("build")]
        }
        "#);

    let items = array_items(result);
    assert_eq!(expect_int(&items[0]), 2);
    assert_eq!(expect_int(&items[1]), 1);
    assert_eq!(expect_int(&items[2]), 1);
    assert_eq!(expect_str(&items[3]), "compile");
}
