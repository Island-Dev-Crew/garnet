//! `garnet doctest <file>` — execute the ` ```garnet ` examples embedded in a
//! file's `///` doc comments (S43, docs-as-tests).
//!
//! The "evidence not courtesy" discipline: a documented example is a claim,
//! and an unrun claim rots. This command makes each example executable.
//!
//! ## How it works
//!
//! 1. Parse the file and, for each top-level item, reuse `garnet doc`'s
//!    backward `///` scan (`cmd::doc::extract_doc_comments_before`)
//!    to recover its doc block.
//! 2. Lift the ` ```garnet ` fences from that block
//!    ([`crate::doctest::garnet_fences`]).
//! 3. Load the file's own definitions once
//!    ([`garnet_interp::Interpreter::load_source`]) so a fence can call the
//!    very functions it documents, then evaluate each fence with
//!    [`garnet_interp::Interpreter::eval_expr_src`].
//!
//! A fence passes if it evaluates without error. If it carries a `# => value`
//! marker, the displayed tail value must also equal the expected text.
//!
//! ## Honest scope
//!
//! - Examples run on the tree-walking interpreter, not the VM backend.
//! - Fences see only the file's own definitions plus the stdlib; there is no
//!   cross-file import resolution inside a fence (matching `garnet doc`).
//! - This is a doc-rot guard, not a replacement for the test suite.

use crate::cmd::doc::{extract_doc_comments_before, item_span};
use crate::diagnostics::json_escape;
use crate::doctest::{garnet_fences, Fence};
use crate::read_file;
use garnet_interp::Interpreter;
use std::path::PathBuf;
use std::process::ExitCode;

enum Format {
    Human,
    Json,
}

enum Outcome {
    Pass,
    Fail(String),
}

struct Case {
    item: String,
    /// 1-based line within the source file (not the doc block).
    line: usize,
    outcome: Outcome,
}

pub fn run(args: &[String]) -> ExitCode {
    let mut format = Format::Human;
    let mut file: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--format" => {
                match args.get(i + 1).map(String::as_str) {
                    Some("human") => format = Format::Human,
                    Some("json") => format = Format::Json,
                    other => {
                        eprintln!("--format expects human|json, got {other:?}");
                        return ExitCode::from(2);
                    }
                }
                i += 2;
            }
            "--help" | "-h" => {
                print_help();
                return ExitCode::SUCCESS;
            }
            other if !other.starts_with("--") => {
                file = Some(args[i].clone());
                i += 1;
            }
            other => {
                eprintln!("unknown doctest flag: {other}");
                return ExitCode::from(2);
            }
        }
    }

    let Some(file) = file else {
        print_help();
        return ExitCode::from(2);
    };
    let path = PathBuf::from(&file);
    let src = match read_file(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(1);
        }
    };

    let module = match garnet_parser::parse_source(&src) {
        Ok(m) => m,
        Err(e) => {
            let report = miette::Report::new(e).with_source_code(src);
            eprintln!("{report:?}");
            return ExitCode::from(1);
        }
    };

    // Collect every fence with the item it documents and its absolute source
    // line. The doc block's `///` lines sit contiguously just above the item,
    // so the file line of a fence is `item_line - doc_lines + (start_line - 1)`.
    let mut pending: Vec<(String, Fence, usize)> = Vec::new();
    for item in &module.items {
        let span = item_span(item);
        let doc_block = extract_doc_comments_before(&src, span.start);
        if doc_block.is_empty() {
            continue;
        }
        let item_line = src[..span.start.min(src.len())].matches('\n').count() + 1;
        let doc_lines = doc_block.lines().count();
        let base_line = item_line.saturating_sub(doc_lines);
        let label = crate::cmd::describe_item(item);
        for fence in garnet_fences(&doc_block) {
            let line = base_line + fence.start_line.saturating_sub(1);
            pending.push((label.clone(), fence, line));
        }
    }

    // Load the file's definitions once so fences can call documented functions.
    // A load failure dooms every fence rather than silently reporting none.
    // Firewalled: top-level `let`/`const` initializers are evaluated during
    // `load_source`, so a load-time panic (e.g. `const X = i64::MIN.abs()`)
    // must doom the fences as failures, not abort the whole doctest process.
    let mut interp = Interpreter::new();
    // S114-FIX-2: frame the load under the file's `main` entry so a top-level
    // `let`/`const` initializer is checked against declared `@caps` (parity with
    // `garnet run`) instead of executing fail-open at doctest-load time.
    let load_err = match crate::panic_firewall::firewalled(|| {
        interp.load_source_with_entry_caps(&src, "main")
    }) {
        Ok(result) => result.err().map(|e| e.to_string()),
        Err(panic_msg) => Some(format!("panicked: {panic_msg}")),
    };

    let mut cases: Vec<Case> = Vec::with_capacity(pending.len());
    for (item, fence, line) in pending {
        let outcome = if let Some(ref e) = load_err {
            Outcome::Fail(format!("file failed to load: {e}"))
        } else {
            run_fence(&interp, &fence)
        };
        cases.push(Case {
            item,
            line,
            outcome,
        });
    }

    let passed = cases
        .iter()
        .filter(|c| matches!(c.outcome, Outcome::Pass))
        .count();
    let failed = cases.len() - passed;

    match format {
        Format::Human => report_human(&path, &cases, passed, failed),
        Format::Json => report_json(&path, &cases, passed, failed),
    }

    if failed > 0 {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

/// Evaluate one fence against the loaded interpreter, applying the optional
/// `# =>` value assertion.
fn run_fence(interp: &Interpreter, fence: &Fence) -> Outcome {
    // Firewalled: a panicking fence (e.g. an `i64::MIN.abs()` overflow) fails
    // that one fence and lets the doctest run continue, rather than aborting the
    // whole process mid-suite.
    let evaluated = crate::panic_firewall::firewalled(|| interp.eval_expr_src(&fence.code));
    match evaluated {
        Err(panic_msg) => Outcome::Fail(format!("panicked: {panic_msg}")),
        Ok(Err(e)) => Outcome::Fail(format!("{e}")),
        Ok(Ok(value)) => match &fence.expect {
            None => Outcome::Pass,
            Some(expected) => {
                let got = value.display();
                if &got == expected {
                    Outcome::Pass
                } else {
                    Outcome::Fail(format!("expected `{expected}`, got `{got}`"))
                }
            }
        },
    }
}

fn report_human(path: &std::path::Path, cases: &[Case], passed: usize, failed: usize) {
    if cases.is_empty() {
        println!("doctest {}: no `garnet` doc examples found", path.display());
        return;
    }
    for case in cases {
        match &case.outcome {
            Outcome::Pass => {
                println!("ok    {} (line {})", case.item, case.line);
            }
            Outcome::Fail(reason) => {
                println!("FAIL  {} (line {}): {reason}", case.item, case.line);
            }
        }
    }
    println!(
        "\n{} example{} checked, {passed} passed, {failed} failed",
        cases.len(),
        if cases.len() == 1 { "" } else { "s" },
    );
}

fn report_json(path: &std::path::Path, cases: &[Case], passed: usize, failed: usize) {
    let mut out = String::new();
    out.push_str("{\"tool\":\"garnet doctest\",\"file\":\"");
    out.push_str(&json_escape(&path.display().to_string()));
    out.push_str("\",\"examples\":[");
    for (idx, case) in cases.iter().enumerate() {
        if idx > 0 {
            out.push(',');
        }
        let (status, detail) = match &case.outcome {
            Outcome::Pass => ("pass", String::new()),
            Outcome::Fail(reason) => ("fail", reason.clone()),
        };
        out.push_str(&format!(
            "{{\"item\":\"{}\",\"line\":{},\"status\":\"{}\",\"detail\":\"{}\"}}",
            json_escape(&case.item),
            case.line,
            status,
            json_escape(&detail),
        ));
    }
    out.push_str(&format!(
        "],\"summary\":{{\"total\":{},\"passed\":{passed},\"failed\":{failed}}},\"ok\":{}}}",
        cases.len(),
        failed == 0
    ));
    println!("{out}");
}

fn print_help() {
    println!("usage: garnet doctest [--format human|json] <file.garnet>");
    println!();
    println!("  Execute the ```garnet examples embedded in a file's /// doc");
    println!("  comments. An example passes if it evaluates without error; a");
    println!("  `# => value` marker additionally asserts the tail value.");
    println!();
    println!("  Examples run on the interpreter and see only the file's own");
    println!("  definitions plus the stdlib (no cross-file imports).");
}

#[cfg(test)]
mod tests {
    use super::*;
    use garnet_interp::Interpreter;

    fn eval(src: &str, fence_code: &str, expect: Option<&str>) -> Outcome {
        let mut interp = Interpreter::new();
        interp.load_source(src).unwrap();
        let fence = Fence {
            code: fence_code.to_string(),
            start_line: 1,
            expect: expect.map(str::to_string),
        };
        run_fence(&interp, &fence)
    }

    #[test]
    fn passing_fence_calls_documented_fn() {
        let src = "def double(x) { x * 2 }\n";
        assert!(matches!(eval(src, "double(21)", Some("42")), Outcome::Pass));
    }

    #[test]
    fn wrong_expectation_fails() {
        let src = "def double(x) { x * 2 }\n";
        assert!(matches!(
            eval(src, "double(21)", Some("43")),
            Outcome::Fail(_)
        ));
    }

    #[test]
    fn panicking_fence_fails_instead_of_aborting() {
        // A fence whose evaluation PANICS (i64::MIN.abs() overflow) must fail
        // that fence — not abort the whole doctest process. Before the firewall
        // this test panicked the test binary.
        let outcome = eval("", "(0 - 9223372036854775807 - 1).abs()", None);
        match outcome {
            Outcome::Fail(msg) => assert!(
                msg.contains("panicked") && msg.contains("overflow"),
                "expected a panicked-overflow failure, got: {msg}"
            ),
            Outcome::Pass => panic!("expected Fail (panicked fence), got Pass"),
        }
    }

    #[test]
    fn runtime_error_fails() {
        let src = "def boom() { raise \"nope\" }\n";
        assert!(matches!(eval(src, "boom()", None), Outcome::Fail(_)));
    }
}
