//! Browser-facing Garnet interpreter surface (W-PLAY Task 1).
//!
//! Wraps `garnet_interp::Interpreter` behind one entry point, `run_source`,
//! that loads Garnet source under the `main` entry's `@caps` frame (the same
//! authority gate the CLI run lane applies), invokes `main`, and returns the
//! REAL captured program output plus a diagnostic — never a fabricated
//! result. Output arrives via the interp's additive capture sink because a
//! browser cannot observe process stdout.
//!
//! Honest scope: this crate proves source-in → real-output-out for the
//! playground. `@caps` is enforced at entry exactly as the interpreter
//! enforces it natively; anything the interpreter defers stays deferred.

#![deny(clippy::unwrap_used, clippy::expect_used)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

use serde::Serialize;

/// Outcome class for one `run_source` invocation.
#[derive(Serialize, Debug, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum ExitClass {
    /// Load + `main` completed without a runtime error.
    Ok,
    /// The source failed to parse/load (includes entry-caps load rejections).
    LoadError,
    /// `main` raised a runtime error (includes capability traps).
    RuntimeError,
}

/// JSON-serializable result of running Garnet source in the wasm interpreter.
#[derive(Serialize, Debug)]
pub struct RunResult {
    /// Schema tag for consumers.
    pub schema: &'static str,
    pub exit_class: ExitClass,
    /// Real program output captured from `print`/`println` — never synthesized.
    pub stdout: String,
    /// Human-readable diagnostic when `exit_class` is not `Ok`.
    pub diagnostic: Option<String>,
}

const SCHEMA: &str = "garnet.wasm.run/1";

/// Run Garnet source: load under `main`'s `@caps` entry frame, call `main`,
/// capture output. The result carries whatever really happened.
pub fn run_source(src: &str) -> RunResult {
    garnet_interp::output::capture_start();
    let mut interp = garnet_interp::Interpreter::new();
    let outcome = match interp.load_source_with_entry_caps(src, "main") {
        Err(load_err) => RunResult {
            schema: SCHEMA,
            exit_class: ExitClass::LoadError,
            stdout: String::new(),
            diagnostic: Some(format!("{load_err:?}")),
        },
        Ok(()) => match interp.call_entry("main", vec![]) {
            Ok(_) => RunResult {
                schema: SCHEMA,
                exit_class: ExitClass::Ok,
                stdout: String::new(),
                diagnostic: None,
            },
            Err(run_err) => RunResult {
                schema: SCHEMA,
                exit_class: ExitClass::RuntimeError,
                stdout: String::new(),
                diagnostic: Some(format!("{run_err:?}")),
            },
        },
    };
    let stdout = garnet_interp::output::capture_take().unwrap_or_default();
    RunResult { stdout, ..outcome }
}

/// Serialize a `RunResult` as the stable JSON surface the playground consumes.
pub fn run_source_json(src: &str) -> String {
    let result = run_source(src);
    serde_json::to_string(&result).unwrap_or_else(|e| {
        format!(
            "{{\"schema\":\"{SCHEMA}\",\"exit_class\":\"runtime_error\",\
             \"stdout\":\"\",\"diagnostic\":\"serialization failed: {e}\"}}"
        )
    })
}

#[cfg(target_arch = "wasm32")]
mod wasm_bindings {
    use wasm_bindgen::prelude::wasm_bindgen;

    /// JS-facing entry point: Garnet source in, `garnet.wasm.run/1` JSON out.
    #[wasm_bindgen]
    pub fn run_source(src: &str) -> String {
        super::run_source_json(src)
    }
}
