//! `xtask` — workspace task runner.
//!
//! Commands:
//! - `seven-run` — runs `cargo test --workspace --no-fail-fast` seven times
//!   and asserts every run reports the exact same pass/fail counts; non-zero
//!   exit on divergence (flaky test, race, environmental noise).
//! - `truth [--check] [--skip-tests] [--with-tests]` — machine-truth
//!   generator + public-surface drift guard (W-REBUILD RB-0a). See
//!   `truth.rs` for field provenance and the deliberate
//!   `security_test_count` omission.

use std::process::{exit, Command};

mod truth;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(|s| s.as_str()) {
        Some("seven-run") => seven_run(),
        Some("truth") => exit(truth::run(&args[1..])),
        Some(other) => {
            eprintln!("unknown xtask command: {other}");
            print_usage();
            exit(2);
        }
        None => {
            print_usage();
            exit(2);
        }
    }
}

fn print_usage() {
    eprintln!("usage: cargo run -p xtask -- <command>");
    eprintln!();
    eprintln!("commands:");
    eprintln!("  seven-run   run `cargo test --workspace --no-fail-fast` 7 times,");
    eprintln!("              fail on any divergence in pass/fail count");
    eprintln!("  truth       regenerate docs/truth.json + stamp <!-- truth:KEY --> markers");
    eprintln!("              in README.md/FAQ.md (--skip-tests reuses the previous");
    eprintln!("              workspace-test measurement)");
    eprintln!("  truth --check");
    eprintln!("              verify machine truth == docs/truth.json == stamped surfaces;");
    eprintln!("              non-zero exit on any mismatch (--with-tests re-measures)");
}

fn seven_run() {
    println!("xtask seven-run: running test suite 7 times for consistency check");
    let mut runs: Vec<truth::Counts> = Vec::new();
    for run_idx in 1..=7 {
        eprintln!("--- run {run_idx}/7 ---");
        let out = Command::new("cargo")
            .args(["test", "--workspace", "--no-fail-fast"])
            .output()
            .expect("failed to spawn cargo test");
        let stdout = String::from_utf8_lossy(&out.stdout);
        let stderr = String::from_utf8_lossy(&out.stderr);
        let counts = truth::parse_counts(&stdout, &stderr);
        eprintln!(
            "    run {run_idx}: passed={} failed={}",
            counts.passed, counts.failed
        );
        runs.push(counts);
    }
    let first = runs[0];
    let mut all_match = true;
    for (i, r) in runs.iter().enumerate() {
        if *r != first {
            eprintln!(
                "DIVERGENCE: run 1 = ({passed} pass, {failed} fail), run {n} = ({rp} pass, {rf} fail)",
                passed = first.passed,
                failed = first.failed,
                n = i + 1,
                rp = r.passed,
                rf = r.failed
            );
            all_match = false;
        }
    }
    if all_match {
        println!(
            "OK 7x consistency: all runs reported {} passed, {} failed",
            first.passed, first.failed
        );
        exit(0);
    } else {
        eprintln!("FAIL: not all 7 runs produced identical pass/fail counts");
        exit(1);
    }
}
