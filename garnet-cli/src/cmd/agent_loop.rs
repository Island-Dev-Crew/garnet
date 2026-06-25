//! `garnet agent-loop --baseline <old> --proposal <new>` — S102 (Stage U): the
//! real agent-acceptance loop. A (simulated) agent proposes a Garnet change; the
//! loop ACCEPTS it ONLY on ENFORCED evidence, in four gated stages:
//!
//!   1. **check** — the proposal must pass the static language/checker gate before
//!      later gates can run or seal it.
//!   2. **diff-caps** (S37) — the declared capability surface must NOT widen. A
//!      widening proposal is a true gate **failure** (Rule 2): it is REFUSED and
//!      never reaches the kernel or the seal.
//!   3. **the enforced kernel** (S99 `@max_depth` + S100 `@caps` traps) — the
//!      proposal must run without tripping an enforced ceiling.
//!   4. **seal** (S38) — an accepted proposal is attested, recording the autonomous
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
//! (`check`, `diff-caps`, `run`, `seal`) as subprocesses of the running binary — it
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
    /// S103: if set, write the full trust dossier (the 4 trust artifacts on accept,
    /// or the refusal record on reject) + an honest `decision.md` into this dir.
    record_dir: Option<PathBuf>,
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
    let mut record_dir: Option<PathBuf> = None;

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
            "--record-dir" => {
                record_dir = Some(PathBuf::from(need(args, i, "--record-dir")?));
                i += 2;
            }
            "--help" | "-h" => {
                println!(
                    "usage: garnet agent-loop --baseline <old.garnet> --proposal <new.garnet> \
                     [--backend interp|vm] [--seal-out <path>] [--authored-by <prov>] \
                     [--attest <k>=<v>]... [--gate-version <id>] [--record-dir <dir>]"
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
        record_dir,
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

/// The loop's verdict on a proposal, used to write the S103 dossier.
enum Outcome<'a> {
    Accepted {
        value: &'a str,
        run_stdout: &'a [u8],
    },
    RejectedCheck {
        check_stdout: &'a [u8],
        check_stderr: &'a [u8],
    },
    RejectedDiffCaps,
    RejectedRun {
        run_stderr: &'a [u8],
    },
}

/// S103: write the trust dossier into `--record-dir`. On ACCEPT this is the **4
/// trust artifacts** — capability_manifest.json (S36), diff_caps.txt (S37),
/// seal.json (S38), transparency_log.jsonl (S68) — plus an honest `decision.md`.
/// On REJECT it records the refusal (the negative proof): no seal is ever written.
fn write_record(a: &Args, exe: &std::path::Path, diff_stdout: &[u8], outcome: &Outcome) {
    let Some(dir) = a.record_dir.as_deref() else {
        return;
    };
    if let Err(e) = std::fs::create_dir_all(dir) {
        eprintln!(
            "garnet agent-loop: cannot create --record-dir `{}`: {e}",
            dir.display()
        );
        return;
    }
    // Artifact 2 — the diff-caps capability-surface decision (always captured).
    let _ = std::fs::write(dir.join("diff_caps.txt"), diff_stdout);

    let decision = match outcome {
        Outcome::Accepted { value, .. } => format!(
            "# Agent-loop decision: ACCEPTED\n\n\
             Proposal `{}` (vs baseline `{}`) was ACCEPTED on capability+depth evidence.\n\n\
             - diff-caps: no authority expansion — the declared capability surface did not widen.\n\
             - enforced kernel ({}): ran without tripping an enforced ceiling ({value}).\n\
             - sealed: attested in `seal.json` with autonomous-acceptance provenance.\n\n\
             The 4 trust artifacts: `capability_manifest.json` (S36), `diff_caps.txt` (S37), \
             `seal.json` (S38), `transparency_log.jsonl` (S68).\n\n\
             Honest scope: accepted on capability + depth evidence ONLY — `@caps` and \
             `@max_depth` are enforced. `@bounded`/memory/time/`@mailbox`/OS-sandbox remain \
             declared-not-enforced; this is NOT a claim of full boundedness or safety.\n",
            a.proposal.display(),
            a.baseline.display(),
            a.backend,
        ),
        Outcome::RejectedCheck { .. } => format!(
            "# Agent-loop decision: REJECTED (proposal failed check)\n\n\
             Proposal `{}` (vs baseline `{}`) was REFUSED at the check gate before diff-caps, \
             run, or seal. See `check.txt` for the checker diagnostics. It was not sealed.\n",
            a.proposal.display(),
            a.baseline.display(),
        ),
        Outcome::RejectedDiffCaps => format!(
            "# Agent-loop decision: REJECTED (capability widening)\n\n\
             Proposal `{}` (vs baseline `{}`) was REFUSED at the diff-caps gate: it WIDENED the \
             declared capability surface (see `diff_caps.txt`). It never ran and was never sealed \
             — the negative proof. A widening is a true gate FAILURE (Rule 2), not a warning.\n",
            a.proposal.display(),
            a.baseline.display(),
        ),
        Outcome::RejectedRun { .. } => format!(
            "# Agent-loop decision: REJECTED (enforced-ceiling trap)\n\n\
             Proposal `{}` (vs baseline `{}`) passed diff-caps (no widening) but the enforced \
             kernel ({}) TRAPPED it (see `run_trap.txt`) — an `@max_depth` or `@caps` ceiling was \
             exceeded. It was not sealed. Acceptance rests on the enforced run, not only the static \
             capability gate.\n",
            a.proposal.display(),
            a.baseline.display(),
            a.backend,
        ),
    };
    let _ = std::fs::write(dir.join("decision.md"), decision);

    match outcome {
        Outcome::Accepted { run_stdout, .. } => {
            // Artifact 1 — the capability manifest (S36).
            if let Ok(o) = Command::new(exe).arg("caps").arg(&a.proposal).output() {
                let _ = std::fs::write(dir.join("capability_manifest.json"), o.stdout);
            }
            // Artifact 3 — the in-toto seal (S38), copied from `--seal-out`.
            let _ = std::fs::copy(&a.seal_out, dir.join("seal.json"));
            // Artifact 4 — the transparency-log entry (S68), appended + chain-verifiable.
            let _ = Command::new(exe)
                .arg("caps-log")
                .arg(&a.proposal)
                .arg("--log")
                .arg(dir.join("transparency_log.jsonl"))
                .output();
            let _ = std::fs::write(dir.join("run_output.txt"), run_stdout);
        }
        Outcome::RejectedCheck {
            check_stdout,
            check_stderr,
        } => {
            let mut check = Vec::with_capacity(check_stdout.len() + check_stderr.len());
            check.extend_from_slice(check_stdout);
            check.extend_from_slice(check_stderr);
            let _ = std::fs::write(dir.join("check.txt"), check);
        }
        Outcome::RejectedRun { run_stderr } => {
            let _ = std::fs::write(dir.join("run_trap.txt"), run_stderr);
        }
        Outcome::RejectedDiffCaps => {}
    }
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

    // STAGE 1 — check: malformed or checker-rejected proposals fail closed before
    // diff-caps, run, or seal. This keeps invalid annotations from being treated
    // as oversized runtime ceilings by the later acceptance stages.
    let check = match Command::new(&exe).arg("check").arg(&a.proposal).output() {
        Ok(o) => o,
        Err(e) => {
            eprintln!("garnet agent-loop: check failed to launch: {e}");
            return ExitCode::from(2);
        }
    };
    if !check.status.success() {
        println!("agent-loop: stage check -> REJECT (proposal failed garnet check)");
        echo("  | ", &check.stdout);
        echo("  | ", &check.stderr);
        println!("agent-loop: REJECTED at stage check (the proposal is not run and is not sealed)");
        write_record(
            &a,
            &exe,
            &[],
            &Outcome::RejectedCheck {
                check_stdout: &check.stdout,
                check_stderr: &check.stderr,
            },
        );
        return ExitCode::from(1);
    }
    println!("agent-loop: stage check -> PASS");

    // STAGE 2 — diff-caps (Rule 2 hard gate): a capability widening is REFUSED and
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
        write_record(&a, &exe, &diff.stdout, &Outcome::RejectedDiffCaps);
        return ExitCode::from(1);
    }
    println!("agent-loop: stage diff-caps -> PASS (no authority expansion, band 5/5)");

    // STAGE 3 — the ENFORCED kernel (S99 @max_depth + S100 @caps). A proposal that
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
        write_record(
            &a,
            &exe,
            &diff.stdout,
            &Outcome::RejectedRun {
                run_stderr: &kernel.stderr,
            },
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

    // STAGE 4 — seal (S38): attest the accepted proposal, recording the autonomous
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
    write_record(
        &a,
        &exe,
        &diff.stdout,
        &Outcome::Accepted {
            value: &value,
            run_stdout: &kernel.stdout,
        },
    );
    println!("agent-loop: ACCEPTED on capability+depth evidence");
    ExitCode::SUCCESS
}
