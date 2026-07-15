//! Browser-facing Garnet interpreter/checker surface (W-PLAY).
//!
//! `run_source` loads under `main`'s `@caps` frame and returns REAL captured
//! output. `check_source` returns the parser/checker's authoritative diagnostics;
//! `diff_caps_source` returns its declared-capability surfaces and diff.
//!
//! Honest scope: the diff is declared-surface-only, and this adapter alone does
//! not prove browser-page execution. Anything deferred stays deferred.

#![deny(clippy::unwrap_used, clippy::expect_used)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

use serde::Serialize;

#[derive(Serialize, Debug, PartialEq, Eq, Clone)]
pub struct WasmDiagnostic {
    pub code: &'static str,
    pub severity: &'static str,
    pub message: String,
}
#[derive(Serialize, Debug, PartialEq, Eq, Clone)]
pub struct CheckResult {
    pub schema: &'static str,
    pub ok: bool,
    pub diagnostics: Vec<WasmDiagnostic>,
}
#[derive(Serialize, Debug, PartialEq, Eq, Clone)]
pub struct FunctionCaps {
    pub name: String,
    pub caps: Vec<String>,
}
#[derive(Serialize, Debug, PartialEq, Eq, Clone)]
pub struct WasmCapabilitySurface {
    pub aggregate: Vec<String>,
    pub per_function: Vec<FunctionCaps>,
    pub has_wildcard: bool,
}
impl From<garnet_check::CapabilitySurface> for WasmCapabilitySurface {
    fn from(surface: garnet_check::CapabilitySurface) -> Self {
        Self {
            aggregate: surface.aggregate,
            per_function: surface
                .per_function
                .into_iter()
                .map(|(name, caps)| FunctionCaps { name, caps })
                .collect(),
            has_wildcard: surface.has_wildcard,
        }
    }
}
#[derive(Serialize, Debug, PartialEq, Eq, Clone)]
pub struct FunctionCapsExpansion {
    pub name: String,
    pub gained: Vec<String>,
}
#[derive(Serialize, Debug, PartialEq, Eq, Clone)]
pub struct SideParseError {
    pub side: &'static str,
    pub diagnostic: WasmDiagnostic,
}
/// Declared-capability diff result. `ok` means both inputs parsed; authority
/// expansion remains a valid verdict and is not an adapter failure.
#[derive(Serialize, Debug, PartialEq, Eq, Clone)]
pub struct DiffCapsResult {
    pub schema: &'static str,
    pub ok: bool,
    pub authority_expanded: Option<bool>,
    pub old_surface: Option<WasmCapabilitySurface>,
    pub new_surface: Option<WasmCapabilitySurface>,
    pub aggregate_added: Vec<String>,
    pub aggregate_removed: Vec<String>,
    pub functions_added: Vec<String>,
    pub functions_removed: Vec<String>,
    pub functions_caps_expanded: Vec<FunctionCapsExpansion>,
    pub wildcard_introduced: Option<bool>,
    pub scope: &'static str,
    pub parse_error: Option<SideParseError>,
}

const CHECK_SCHEMA: &str = "garnet.wasm.check/1";
const DIFF_CAPS_SCHEMA: &str = "garnet.wasm.diff-caps/1";
const DIFF_CAPS_SCOPE: &str = "declared-surface-only; does not prove absence of undeclared authority; bound annotations are not part of this surface";

fn parse_diagnostic(error: &garnet_parser::error::ParseError) -> WasmDiagnostic {
    use garnet_parser::error::ParseError;

    let code = match error {
        ParseError::UnexpectedChar { .. } => "parse.unexpected_char",
        ParseError::UnterminatedString { .. } => "parse.unterminated_string",
        ParseError::InvalidInt { .. } => "parse.invalid_int",
        ParseError::InvalidFloat { .. } => "parse.invalid_float",
        ParseError::UnexpectedToken { .. } => "parse.unexpected_token",
        ParseError::UnexpectedEof { .. } => "parse.unexpected_eof",
        ParseError::BudgetExceeded { .. } => "parse.budget_exceeded",
        ParseError::ReservedWord { .. } => "parse.reserved_word",
    };
    WasmDiagnostic {
        code,
        severity: "error",
        message: error.to_string(),
    }
}

fn checker_severity(severity: garnet_check::Severity) -> &'static str {
    match severity {
        garnet_check::Severity::Error => "error",
        garnet_check::Severity::Warning => "warning",
        garnet_check::Severity::Info => "info",
    }
}

/// Parse and run the real safe-mode checker over one source buffer.
pub fn check_source(src: &str) -> CheckResult {
    match garnet_parser::parse_source(src) {
        Err(error) => CheckResult {
            schema: CHECK_SCHEMA,
            ok: false,
            diagnostics: vec![parse_diagnostic(&error)],
        },
        Ok(module) => {
            let report = garnet_check::check_module(&module);
            CheckResult {
                schema: CHECK_SCHEMA,
                ok: report.ok(),
                diagnostics: report
                    .errors
                    .iter()
                    .map(|error| WasmDiagnostic {
                        code: error.code(),
                        severity: checker_severity(error.severity()),
                        message: error.to_string(),
                    })
                    .collect(),
            }
        }
    }
}

/// Serialize the stable `garnet.wasm.check/1` surface.
pub fn check_source_json(src: &str) -> String {
    adapter_json(&check_source(src), CHECK_SCHEMA)
}

fn diff_parse_failure(
    side: &'static str,
    error: &garnet_parser::error::ParseError,
    old_surface: Option<WasmCapabilitySurface>,
) -> DiffCapsResult {
    DiffCapsResult {
        schema: DIFF_CAPS_SCHEMA,
        ok: false,
        authority_expanded: None,
        old_surface,
        new_surface: None,
        aggregate_added: Vec::new(),
        aggregate_removed: Vec::new(),
        functions_added: Vec::new(),
        functions_removed: Vec::new(),
        functions_caps_expanded: Vec::new(),
        wildcard_introduced: None,
        scope: DIFF_CAPS_SCOPE,
        parse_error: Some(SideParseError {
            side,
            diagnostic: parse_diagnostic(error),
        }),
    }
}

/// Diff the parser/checker's declared-capability surfaces for two source buffers.
pub fn diff_caps_source(old_src: &str, new_src: &str) -> DiffCapsResult {
    let old_module = match garnet_parser::parse_source(old_src) {
        Ok(module) => module,
        Err(error) => return diff_parse_failure("old", &error, None),
    };
    let old_surface = garnet_check::capability_surface(&old_module);
    let new_module = match garnet_parser::parse_source(new_src) {
        Ok(module) => module,
        Err(error) => return diff_parse_failure("new", &error, Some(old_surface.clone().into())),
    };
    let new_surface = garnet_check::capability_surface(&new_module);
    let diff = garnet_check::diff_caps(&old_surface, &new_surface);

    DiffCapsResult {
        schema: DIFF_CAPS_SCHEMA,
        ok: true,
        authority_expanded: Some(diff.authority_expanded()),
        old_surface: Some(old_surface.into()),
        new_surface: Some(new_surface.into()),
        aggregate_added: diff.aggregate_added,
        aggregate_removed: diff.aggregate_removed,
        functions_added: diff.functions_added,
        functions_removed: diff.functions_removed,
        functions_caps_expanded: diff
            .functions_caps_expanded
            .into_iter()
            .map(|(name, gained)| FunctionCapsExpansion { name, gained })
            .collect(),
        wildcard_introduced: Some(diff.wildcard_introduced),
        scope: DIFF_CAPS_SCOPE,
        parse_error: None,
    }
}

/// Serialize the stable `garnet.wasm.diff-caps/1` surface.
pub fn diff_caps_json(old_src: &str, new_src: &str) -> String {
    adapter_json(&diff_caps_source(old_src, new_src), DIFF_CAPS_SCHEMA)
}

fn adapter_json<T: Serialize>(value: &T, schema: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| {
        format!("{{\"schema\":\"{schema}\",\"ok\":false,\"serialization_error\":true}}")
    })
}

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

    /// JS-facing checker: Garnet source in, `garnet.wasm.check/1` JSON out.
    #[wasm_bindgen]
    pub fn check_source(src: &str) -> String {
        super::check_source_json(src)
    }

    /// JS-facing capability diff: two sources in, versioned JSON out.
    #[wasm_bindgen]
    pub fn diff_caps_source(old_src: &str, new_src: &str) -> String {
        super::diff_caps_json(old_src, new_src)
    }
}
