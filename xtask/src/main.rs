//! `xtask` — workspace task runner.
//!
//! Currently exposes one command: `cargo run -p xtask -- seven-run`. It runs
//! `cargo test --workspace --no-fail-fast` seven times in a row and asserts
//! that every run reports the *exact same* pass / fail counts. Any
//! divergence indicates non-determinism (flaky test, race condition, or
//! environmental noise) — non-zero exit on divergence so CI can act on it.

use std::process::{exit, Command};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(|s| s.as_str()) {
        Some("seven-run") => seven_run(),
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
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct Counts {
    passed: u64,
    failed: u64,
}

fn seven_run() {
    println!("xtask seven-run: running test suite 7 times for consistency check");
    let mut runs: Vec<Counts> = Vec::new();
    for run_idx in 1..=7 {
        eprintln!("--- run {run_idx}/7 ---");
        let out = Command::new("cargo")
            .args(["test", "--workspace", "--no-fail-fast"])
            .output()
            .expect("failed to spawn cargo test");
        let stdout = String::from_utf8_lossy(&out.stdout);
        let stderr = String::from_utf8_lossy(&out.stderr);
        let counts = parse_counts(&stdout, &stderr);
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

/// Sum every `test result: ok. P passed; F failed` line in the stdout/stderr.
fn parse_counts(stdout: &str, stderr: &str) -> Counts {
    let mut passed = 0u64;
    let mut failed = 0u64;
    for stream in [stdout, stderr] {
        for line in stream.lines() {
            if let Some(rest) = line.trim().strip_prefix("test result:") {
                // Form: "test result: ok. 23 passed; 0 failed; 0 ignored; ..."
                let mut tokens = rest.split_whitespace();
                while let Some(t) = tokens.next() {
                    if let Ok(n) = t.parse::<u64>() {
                        if let Some(label) = tokens.next() {
                            let lab = label.trim_end_matches(';');
                            match lab {
                                "passed" => passed += n,
                                "failed" => failed += n,
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }
    Counts { passed, failed }
}
