//! RB-4b.3 — per-pass capability re-check on the AST→bytecode lowering
//! (Directive 7, the GHC-Core pattern).
//!
//! The VM compiler lowers the AST to bytecode and, in doing so, drops the
//! source's `@caps` annotations entirely (the compiler has zero capability
//! awareness). This module re-establishes the invariant the checker proves
//! at the source level, but on the LOWERED artifact: **no native function's
//! bytecode may require more host authority than the checker's per-function
//! transitive verdict allows.** A lowering (or future optimization) pass
//! that launders authority — emitting a `Call` to a capability-bearing
//! primitive into a function whose source never declared it — is caught
//! HERE, at the pass that introduced it, rather than as a downstream
//! surprise. "The seal attests what the core proves" stays mechanically
//! true across this lowering.
//!
//! ## Honest scope (no overclaim)
//!
//! - This is a **static cross-IR caps-containment check (lowered ⊆ declared)
//!   with a deterministic trap** ([`recheck_caps`] rejects a planted
//!   laundering instruction — see the tests), NOT new runtime enforcement.
//!   It is one-directional: a function may *declare* more than its bytecode
//!   uses; only bytecode that requires *more* than the declared surface is a
//!   laundering. Runtime caps enforcement is the interpreter's S90
//!   `require_capability` / VM entry-frame (S92) job; this check does not
//!   replace or extend it.
//! - **Fallback (non-native) functions are skipped**: they execute under the
//!   interpreter's S90 guards, so a re-check there would be vacuous (the
//!   interpreter already enforces the source caps at run time).
//! - On every real program the check is SATISFIED by construction (the
//!   compiler lowers calls faithfully, so a native function's bytecode caps
//!   are a subset of its source's transitive caps). Its value is the
//!   **guard against a future pass that widens authority** — proven real,
//!   not aspirational, by the planted-laundering trap test.
//! - The lowered surface is computed from each native function's direct
//!   `Call` instructions (qualified paths fall back to tree-walk, so native
//!   bytecode calls are bare-name). Resolution mirrors the checker
//!   (`caps_graph::resolve_callee`): a `Call` whose bare name is a **user
//!   function declared in this module** resolves to that user fn — even one
//!   named like a primitive (`get`, `read_file`, …) — and contributes no
//!   direct caps (they flow through the checker's transitive verdict); so does
//!   a caps-free builtin (`print`/`len`). Only an **unshadowed** registry
//!   primitive contributes its caps.
//! - **Seal embedding is out of scope** (RFC-gated, Jon): this slice lands
//!   the mechanism + trap; wiring the verdict into the seal predicate is a
//!   later capability-model change.

use crate::bytecode::{BytecodeFunction, BytecodeProgram, Constant, Instruction};
use garnet_check::caps_graph::check_caps_coverage;
use garnet_check::CapSet;
use garnet_parser::ast::Module;
use std::collections::BTreeSet;

/// A lowering pass laundered authority: a native function's bytecode requires
/// a capability the checker's source-level verdict does not grant it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapsLaundering {
    /// The native function whose lowered bytecode over-reaches.
    pub function: String,
    /// The capabilities present in the lowered bytecode but absent from the
    /// checker's transitive verdict for `function` (lexicographic).
    pub widened: Vec<String>,
}

impl std::fmt::Display for CapsLaundering {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "caps laundering: lowered bytecode for `{}` requires [{}], not granted by its \
             source @caps surface",
            self.function,
            self.widened.join(", ")
        )
    }
}

/// Compile `src` to a VM artifact AND re-check the lowering did not launder
/// authority (the on-path RB-4b.3 instance). A caller that wants the
/// "lowering preserves the caps surface" guarantee uses this instead of the
/// behavior-identical [`crate::compile_source`]; a laundering becomes a
/// `VmError::Compile`. Normal compilation is unchanged.
pub fn compile_source_rechecked(src: &str) -> Result<crate::VmArtifact, crate::VmError> {
    let artifact = crate::compile_source(src)?;
    if let Err(laundering) = recheck_artifact(&artifact) {
        return Err(crate::VmError::Compile(laundering.to_string()));
    }
    Ok(artifact)
}

/// Re-check a compiled [`crate::VmArtifact`] against the source it was
/// lowered from (the wired RB-4b.3 instance on the VM compile path). Re-parses
/// the artifact's source — which compiled successfully, so it parses — and
/// runs [`recheck_caps`]. A parse failure (unreachable for a real artifact)
/// is treated as "nothing to re-check".
pub fn recheck_artifact(artifact: &crate::VmArtifact) -> Result<(), CapsLaundering> {
    match garnet_parser::parse_source(&artifact.source) {
        Ok(module) => recheck_caps(&artifact.program, &module),
        Err(_) => Ok(()),
    }
}

/// Re-check the lowered `program` against the checker's source-level caps
/// verdict for `source` (the AST the program was compiled from). Returns the
/// first laundering found, or `Ok(())` if every native function's lowered
/// capability surface is within its declared transitive surface.
pub fn recheck_caps(program: &BytecodeProgram, source: &Module) -> Result<(), CapsLaundering> {
    let report = check_caps_coverage(source);
    // The module's declared user functions. `check_caps_coverage` records a
    // transitive entry for every declared fn, so its keys ARE the user-fn set.
    // A bare bytecode `Call` naming one of these resolves to the user function
    // (NOT a same-named primitive), mirroring `caps_graph::resolve_callee`'s
    // shadow order — its caps already flow through the checker's transitive
    // verdict, so it adds no DIRECT caps here.
    let user_fns: BTreeSet<&str> = report.transitive.keys().map(String::as_str).collect();
    for func in &program.functions {
        // Fallback functions run under the interpreter's S90 guards; a
        // re-check here is vacuous (disclosed in the module docs).
        if !func.native {
            continue;
        }
        let declared = report
            .transitive
            .get(&func.name)
            .copied()
            .unwrap_or(CapSet::EMPTY);
        let lowered = lowered_caps(func, &program.constants, &user_fns);
        let widened = lowered.difference(declared);
        if !widened.is_empty() {
            return Err(CapsLaundering {
                function: func.name.clone(),
                widened: widened.names().into_iter().map(String::from).collect(),
            });
        }
    }
    Ok(())
}

/// The capability surface a native function's bytecode directly requires:
/// the union of registry caps of every *primitive* it `Call`s. A callee that
/// names a user function in this module (`user_fns`) is shadowed exactly as
/// `caps_graph::resolve_callee` shadows it — even when that user function is
/// named like a primitive — and contributes nothing here; its caps already
/// flow through the checker's transitive verdict. `print`/`len` and other
/// non-registry names also contribute nothing.
fn lowered_caps(
    func: &BytecodeFunction,
    constants: &[Constant],
    user_fns: &BTreeSet<&str>,
) -> CapSet {
    let mut caps = CapSet::EMPTY;
    for instr in &func.instructions {
        if let Instruction::Call { name, .. } = instr {
            if let Some(Constant::Str(callee)) = constants.get(*name as usize) {
                // Mirror `caps_graph::resolve_callee`: a call whose (bare) name
                // is a declared user function resolves to that user fn FIRST,
                // shadowing any registry primitive of the same name.
                let bare = callee.rsplit("::").next().unwrap_or(callee);
                if user_fns.contains(bare) {
                    continue;
                }
                caps |= prim_caps_for(callee);
            }
        }
    }
    caps
}

/// Registry caps for an *unshadowed* primitive callee, resolved both as a
/// qualified key (`fs::read_file`) and a bare last segment (`read_file`). When
/// several prims share a bare name (`array::contains` vs `str::contains`) the
/// caps are unioned — matching the checker's bare-name index, which unions on
/// collision (`caps_graph.rs`). The caller ([`lowered_caps`]) applies the
/// user-fn shadow first, so the two IRs agree on what a call requires.
/// Unknown names contribute no caps.
fn prim_caps_for(name: &str) -> CapSet {
    let registry = garnet_stdlib::registry::all_prims();
    let bare = name.rsplit("::").next().unwrap_or(name);
    registry
        .iter()
        .filter(|(key, _)| *key == name || key.rsplit("::").next() == Some(bare))
        .flat_map(|(_, meta)| meta.required_caps.0.iter())
        .fold(CapSet::EMPTY, |acc, cap| {
            acc | CapSet::from_name_or_other(cap)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile_source;

    fn program(src: &str) -> (BytecodeProgram, Module) {
        let artifact = compile_source(src).expect("source compiles");
        let module = garnet_parser::parse_source(src).expect("source parses");
        (artifact.program, module)
    }

    #[test]
    fn faithful_lowering_passes_the_recheck() {
        // A function that declares + uses fs is consistent across IRs.
        let (program, source) = program(
            "@caps(fs)\ndef readit() {\n  read_file(\"x\")\n}\n@caps()\ndef main() {\n  1\n}\n",
        );
        assert_eq!(recheck_caps(&program, &source), Ok(()));
    }

    #[test]
    fn pure_program_passes_the_recheck() {
        let (program, source) = program("@caps()\ndef main() {\n  1 + 2\n}\n");
        assert_eq!(recheck_caps(&program, &source), Ok(()));
    }

    #[test]
    fn planted_laundering_call_is_trapped() {
        // The deterministic trap: take a function whose SOURCE declares no fs
        // authority, then inject a `Call` to the fs-requiring `read_file`
        // primitive into its NATIVE bytecode — simulating a lowering pass
        // that laundered authority. The re-check must catch it.
        let (mut program, source) = program("@caps()\ndef main() {\n  1\n}\n");
        let main = program
            .functions
            .iter_mut()
            .find(|f| f.name == "main")
            .expect("main compiled");
        assert!(main.native, "main is native bytecode");
        let name = program.constants.len() as u32;
        program
            .constants
            .push(Constant::Str("read_file".to_string()));
        // re-borrow after the constants push
        let main = program
            .functions
            .iter_mut()
            .find(|f| f.name == "main")
            .unwrap();
        main.instructions
            .insert(0, Instruction::Call { name, argc: 1 });

        let err = recheck_caps(&program, &source).expect_err("laundering must be trapped");
        assert_eq!(err.function, "main");
        assert_eq!(err.widened, vec!["fs"]);
    }

    #[test]
    fn user_function_shadowing_a_primitive_name_is_not_laundering() {
        // RB-4b.3 review finding (HIGH): bare `Call` names carry no
        // user-vs-primitive tag through lowering, so the re-check must consult
        // the module's declared functions and mirror
        // `caps_graph::resolve_callee` — a user function named like a
        // cap-bearing primitive (here `read_file`, which carries `fs`)
        // resolves to the USER fn and must NOT be flagged as laundering. Before
        // the shadow fix, `main`'s call to the user `read_file` was mis-read as
        // the fs primitive and `compile_source_rechecked` rejected valid code.
        let (program, source) = program(
            "@caps()\ndef read_file(x) {\n  x\n}\n@caps()\ndef main() {\n  read_file(1)\n}\n",
        );
        // Non-vacuous: both the shadowing fn and its caller must be native
        // bytecode (a fallback would be skipped and hide the bug).
        assert!(
            program
                .functions
                .iter()
                .any(|f| f.name == "read_file" && f.native),
            "the shadowing user fn must compile native for this test to bite"
        );
        assert!(program
            .functions
            .iter()
            .any(|f| f.name == "main" && f.native));
        assert_eq!(recheck_caps(&program, &source), Ok(()));
    }

    #[test]
    fn fallback_functions_are_skipped() {
        // A function that falls back to tree-walk (e.g. a closure) is not
        // native; the re-check skips it (interp S90 guards cover it). Even a
        // planted Call on a fallback function is ignored here.
        let (mut program, source) = program("@caps()\ndef main() {\n  1\n}\n");
        let name = program.constants.len() as u32;
        program
            .constants
            .push(Constant::Str("read_file".to_string()));
        let main = program
            .functions
            .iter_mut()
            .find(|f| f.name == "main")
            .unwrap();
        main.native = false;
        main.instructions
            .insert(0, Instruction::Call { name, argc: 1 });
        assert_eq!(recheck_caps(&program, &source), Ok(()));
    }
}
