//! `garnet agent-loop --baseline <old> --proposal <new>` — S102 (Stage U): the
//! real agent-acceptance loop. A (simulated) agent proposes a Garnet change; the
//! loop ACCEPTS it ONLY on ENFORCED evidence, in three gated stages:
//!
//!   1. **diff-caps** (S37) — the declared capability surface must NOT widen. A
//!      widening proposal is a true gate **failure** (Rule 2): it is REFUSED and
//!      never reaches the kernel or the seal.
//!   2. **the enforced kernel** (S99 `@max_depth` + S100 `@caps` traps) — the
//!      proposal must run without tripping an enforced ceiling.
//!   3. **seal** (S38) — an accepted proposal is attested, recording the autonomous
//!      acceptance + agent/model/gate-version provenance (S65/S66).
//!
//! Honest scope: acceptance rests ONLY on the two ENFORCED ceilings — `@caps`
//! host-authority + `@max_depth` recursion. `@bounded` (Wasmtime fuel), memory,
//! time, `@mailbox`, and OS-level sandbox remain **declared-not-enforced**. The
//! verdict is **"ACCEPTED on capability+depth evidence"** — never "fully bounded",
//! "sandboxed", or "safe". diff-caps reads the DECLARED surface; it does not prove
//! the absence of undeclared authority (S46's job). The agent is SIMULATED/SCRIPTED
//! (the proposal is an on-disk file), not a live LLM (that is S94, `[ACCT-GATED]`);
//! the attested `model` should be `simulated`.
//!
//! Wrap, don't rebuild: the loop orchestrates the real `garnet` subcommands
//! (`diff-caps`, `run`, `seal`) as subprocesses of the running binary — it
//! reimplements no gate, so it cannot drift from the gates it accepts under.

use std::path::PathBuf;
use std::process::{Command, ExitCode};

struct Args {
    baseline: PathBuf,
    proposal: PathBuf,
    /// The run backend flag passed to `garnet run`: `--interp` or `--vm`.
    backend: String,
    seal_out: PathBuf,
    authored_by: String,
    /// `key=value` attestation entries threaded into the seal predicate (S66).
    attest: Vec<String>,
}

fn need<'a>(args: &'a [String], i: usize, flag: &str) -> Result<&'a String, ExitCode> {
    args.get(i + 1).ok_or_else(|| {
        eprintln!("garnet agent-loop: {flag} requires a value");
        ExitCode::from(2)
    })
}

fn parse(args: &[String]) -> Result<Args, ExitCode> {
    let mut baseline: Option<PathBuf> = None;
    let mut proposal: Option<PathBuf> = None;
    let mut backend = "--interp".to_string();
    let mut seal_out: Option<PathBuf> = None;
    let mut authored_by: Option<String> = None;
    let mut attest: Vec<String> = Vec::new();
    let mut gate_version: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--baseline" => {
                baseline = Some(PathBuf::from(need(args, i, "--baseline")?));
                i += 2;
            }
            "--proposal" => {
                proposal = Some(PathBuf::from(need(args, i, "--proposal")?));
                i += 2;
            }
            "--backend" => {
                backend = match need(args, i, "--backend")?.as_str() {
                    "interp" | "--interp" => "--interp".to_string(),
                    "vm" | "--vm" => "--vm".to_string(),
                    other => {
                        eprintln!("garnet agent-loop: --backend must be interp|vm, got `{other}`");
                        return Err(ExitCode::from(2));
                    }
                };
                i += 2;
            }
            "--seal-out" => {
                seal_out = Some(PathBuf::from(need(args, i, "--seal-out")?));
                i += 2;
            }
            "--authored-by" => {
                authored_by = Some(need(args, i, "--authored-by")?.clone());
                i += 2;
            }
            "--attest" => {
                let kv = need(args, i, "--attest")?;
                if !kv.contains('=') {
                    eprintln!("garnet agent-loop: --attest expects <key>=<value>, got `{kv}`");
                    return Err(ExitCode::from(2));
                }
                attest.push(kv.clone());
                i += 2;
            }
            "--gate-version" => {
                gate_version = Some(need(args, i, "--gate-version")?.clone());
                i += 2;
            }
            "--help" | "-h" => {
                println!(
                    "usage: garnet agent-loop --baseline <old.garnet> --proposal <new.garnet> \
                     [--backend interp|vm] [--seal-out <path>] [--authored-by <prov>] \
                     [--attest <k>=<v>]... [--gate-version <id>]"
                );
                return Err(ExitCode::SUCCESS);
            }
            other => {
                eprintln!("garnet agent-loop: unexpected argument `{other}`");
                return Err(ExitCode::from(2));
            }
        }
    }

    let (Some(baseline), Some(proposal)) = (baseline, proposal) else {
        eprintln!("usage: garnet agent-loop --baseline <old.garnet> --proposal <new.garnet> [...]");
        return Err(ExitCode::from(2));
    };
    let seal_out = seal_out.unwrap_or_else(|| {
        let mut p = proposal.clone();
        p.set_extension("seal.json");
        p
    });
    // S102: the agent is simulated unless the caller declares otherwise (honest —
    // a real model name here would be a false provenance claim; live LLM is S94).
    let authored_by = authored_by.unwrap_or_else(|| "sim:scripted-agent".to_string());

    // Rule 3: every accepted seal records the autonomous acceptance + the gate the
    // loop accepted under. These are the harness's own truthful provenance; the
    // caller supplies agent/model/prompt_sha256 via --attest.
    let mut full_attest = vec![
        "tool=garnet-agent-loop".to_string(),
        "autonomous=true".to_string(),
        "decision=accepted-on-capability+depth-evidence".to_string(),
    ];
    if let Some(gv) = gate_version {
        full_attest.push(format!("gate_version={gv}"));
    }
    full_attest.extend(attest);

    Ok(Args {
        baseline,
        proposal,
        backend,
        seal_out,
        authored_by,
        attest: full_attest,
    })
}

/// Print the last `n` non-empty lines of captured output, indented, for the
/// transcript (e.g. the diff-caps GAINED line, or a trap).
fn echo(prefix: &str, bytes: &[u8]) {
    for line in String::from_utf8_lossy(bytes).lines() {
        if !line.trim().is_empty() {
            println!("{prefix}{line}");
        }
    }
}

fn garnet_exe() -> Result<PathBuf, ExitCode> {
    std::env::current_exe().map_err(|e| {
        eprintln!("garnet agent-loop: cannot locate the garnet binary: {e}");
        ExitCode::from(2)
    })
}

pub fn run(args: &[String]) -> ExitCode {
    let a = match parse(args) {
        Ok(a) => a,
        Err(code) => return code,
    };
    let exe = match garnet_exe() {
        Ok(p) => p,
        Err(code) => return code,
    };

    println!(
        "agent-loop: proposal `{}` vs baseline `{}` (backend {})",
        a.proposal.display(),
        a.baseline.display(),
        a.backend
    );

    // STAGE 1 — diff-caps (Rule 2 hard gate): a capability widening is REFUSED and
    // never reaches the kernel or the seal.
    let diff = match Command::new(&exe)
        .arg("diff-caps")
        .arg(&a.baseline)
        .arg(&a.proposal)
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            eprintln!("garnet agent-loop: diff-caps failed to launch: {e}");
            return ExitCode::from(2);
        }
    };
    if !diff.status.success() {
        if diff.status.code() == Some(1) {
            println!("agent-loop: stage diff-caps -> REJECT (AUTHORITY EXPANDED, band 2/5)");
            echo("  | ", &diff.stdout);
            println!(
                "agent-loop: REJECTED at stage diff-caps (a capability widening is refused; \
                 the proposal never runs and is never sealed)"
            );
        } else {
            println!("agent-loop: stage diff-caps -> ERROR");
            echo("  | ", &diff.stderr);
            return ExitCode::from(2);
        }
        return ExitCode::from(1);
    }
    println!("agent-loop: stage diff-caps -> PASS (no authority expansion, band 5/5)");

    // STAGE 2 — the ENFORCED kernel (S99 @max_depth + S100 @caps). A proposal that
    // trips an enforced ceiling is REFUSED even though diff-caps passed.
    let kernel = match Command::new(&exe)
        .arg("run")
        .arg(&a.backend)
        .arg(&a.proposal)
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            eprintln!("garnet agent-loop: run failed to launch: {e}");
            return ExitCode::from(2);
        }
    };
    if !kernel.status.success() {
        println!(
            "agent-loop: stage run({}) -> REJECT (the enforced kernel trapped)",
            a.backend
        );
        echo("  | ", &kernel.stderr);
        println!(
            "agent-loop: REJECTED at stage run (an enforced ceiling — @max_depth or @caps — \
             trapped; the proposal is not sealed)"
        );
        return ExitCode::from(1);
    }
    let value = String::from_utf8_lossy(&kernel.stdout)
        .lines()
        .find(|l| l.trim_start().starts_with("=>"))
        .unwrap_or("(no value)")
        .trim()
        .to_string();
    println!("agent-loop: stage run({}) -> PASS ({value})", a.backend);

    // STAGE 3 — seal (S38): attest the accepted proposal, recording the autonomous
    // acceptance + agent/model/gate-version provenance (Rule 3). cosign signs it
    // when present; absent cosign, the predicate is emitted UNSIGNED.
    let mut seal = Command::new(&exe);
    seal.arg("seal")
        .arg(&a.proposal)
        .arg("--out")
        .arg(&a.seal_out)
        .arg("--authored-by")
        .arg(&a.authored_by);
    for kv in &a.attest {
        seal.arg("--attest").arg(kv);
    }
    let sealed = match seal.output() {
        Ok(o) => o,
        Err(e) => {
            eprintln!("garnet agent-loop: seal failed to launch: {e}");
            return ExitCode::from(2);
        }
    };
    if !sealed.status.success() {
        println!("agent-loop: stage seal -> ERROR");
        echo("  | ", &sealed.stderr);
        return ExitCode::from(2);
    }
    println!(
        "agent-loop: stage seal -> SEALED ({}) (unsigned unless cosign present)",
        a.seal_out.display()
    );
    println!("agent-loop: ACCEPTED on capability+depth evidence");
    ExitCode::SUCCESS
}
