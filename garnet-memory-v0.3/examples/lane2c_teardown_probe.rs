//! Lane 2C replay harness for memory teardown operation counts.
//!
//! Plain replay:
//! `cargo run -p garnet-memory --example lane2c_teardown_probe --release -- working-clear 256`
//!
//! Set `GARNET_LANE2C_CALLGRIND=1` when the process is running under
//! Callgrind. The probe leaves instrumentation disabled while building the
//! store, then zeroes and enables the counter immediately before teardown.

use garnet_memory::{EpisodeStore, VectorIndex, WorkingStore};
use std::env;
use std::process::{self, Command};

fn start_counting() {
    let pid = process::id().to_string();
    let status = Command::new("callgrind_control")
        .args(["--zero", &pid])
        .status()
        .expect("callgrind_control must be available");
    assert!(status.success(), "callgrind_control --zero failed");

    let status = Command::new("callgrind_control")
        .args(["--instr=on", &pid])
        .status()
        .expect("callgrind_control must be available");
    assert!(status.success(), "callgrind_control --instr=on failed");
}

fn working_clear(size: usize, count_operations: bool) {
    let store = WorkingStore::new();
    for value in 0..size {
        store.push(value);
    }
    if count_operations {
        start_counting();
    }
    store.clear();
}

fn episodic_drop(size: usize, count_operations: bool) {
    let store = EpisodeStore::new();
    for value in 0..size {
        store.append_at(value as u64, value);
    }
    if count_operations {
        start_counting();
    }
    drop(store);
}

fn semantic_drop(size: usize, count_operations: bool) {
    let store = VectorIndex::new();
    for value in 0..size {
        let fraction = value as f32 / size.max(1) as f32;
        store.insert(vec![fraction, 1.0 - fraction], value);
    }
    if count_operations {
        start_counting();
    }
    drop(store);
}

fn main() {
    let mut args = env::args().skip(1);
    let case = args.next().expect("usage: lane2c_teardown_probe CASE SIZE");
    let size = args
        .next()
        .expect("usage: lane2c_teardown_probe CASE SIZE")
        .parse::<usize>()
        .expect("SIZE must be an unsigned integer");
    assert!(
        args.next().is_none(),
        "usage: lane2c_teardown_probe CASE SIZE"
    );

    let count_operations = match env::var("GARNET_LANE2C_CALLGRIND") {
        Err(env::VarError::NotPresent) => false,
        Ok(value) if value == "1" => true,
        Ok(value) => panic!("GARNET_LANE2C_CALLGRIND must be 1 when set, got {value:?}"),
        Err(env::VarError::NotUnicode(_)) => {
            panic!("GARNET_LANE2C_CALLGRIND must contain valid Unicode")
        }
    };

    eprintln!(
        "candidate={} case={} size={} operation_counting={} pid={}",
        env::var("GARNET_HEAD").unwrap_or_else(|_| "unbound".to_string()),
        case,
        size,
        count_operations,
        process::id()
    );

    match case.as_str() {
        "working-clear" => working_clear(size, count_operations),
        "episodic-drop" => episodic_drop(size, count_operations),
        "semantic-drop" => semantic_drop(size, count_operations),
        _ => panic!("unknown case: {case}"),
    }
}
