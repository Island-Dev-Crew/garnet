//! `std::process` — child-process spawning (Layer 1, cap: `proc`).
//!
//! Host helpers over `std::process`. A Garnet function calling these must
//! declare `@caps(proc)`. `@stability(experimental)`.
//!
//! Two argv contracts coexist:
//!   * [`spawn`] takes a single command line split on ASCII whitespace (no shell
//!     quoting) — the v0.7 contract, kept for backward compatibility.
//!   * [`spawn_args`] and [`output`] (S23) take a `program` plus an explicit argv
//!     array; each element is passed to the OS literally, so an argument that
//!     contains spaces is **not** re-split. [`output`] additionally runs the child
//!     to completion and captures its stdout/stderr/exit-code.

use crate::StdError;
use std::process::{Child, Command};

/// A spawned child process.
#[derive(Debug)]
pub struct Proc(Child);

impl Proc {
    /// The OS process id of the spawned child.
    pub fn pid(&self) -> u32 {
        self.0.id()
    }
}

/// The terminal status of a finished child.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcStatus {
    code: Option<i32>,
}

/// Spawn a child from a whitespace-delimited command line. Errors on an
/// empty command or if the program cannot be launched.
pub fn spawn(cmdline: &str) -> Result<Proc, StdError> {
    let mut parts = cmdline.split_whitespace();
    let program = parts
        .next()
        .ok_or_else(|| StdError::InvalidInput("process spawn: empty command line".into()))?;
    let child = Command::new(program)
        .args(parts)
        .spawn()
        .map_err(|e| StdError::Io(format!("process spawn `{program}`: {e}")))?;
    Ok(Proc(child))
}

/// Wait for a child to exit and capture its terminal status.
pub fn wait(proc: Proc) -> Result<ProcStatus, StdError> {
    let status = proc
        .0
        .wait_with_output()
        .map_err(|e| StdError::Io(format!("process wait: {e}")))?
        .status;
    Ok(ProcStatus {
        code: status.code(),
    })
}

/// The integer exit code, or `None` if the child was terminated by a signal.
pub fn exit_code(status: &ProcStatus) -> Option<i32> {
    status.code
}

/// The captured result of running a child to completion via [`output`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

impl Output {
    /// The integer exit code, or `None` if terminated by a signal.
    pub fn code(&self) -> Option<i32> {
        self.code
    }
    /// Captured standard output (lossy UTF-8).
    pub fn stdout(&self) -> &str {
        &self.stdout
    }
    /// Captured standard error (lossy UTF-8).
    pub fn stderr(&self) -> &str {
        &self.stderr
    }
}

/// Spawn a child from an explicit program + argv array. Unlike [`spawn`], the
/// program and each argument are passed to the OS literally (no whitespace
/// splitting), so an argument containing spaces survives as a single argv
/// element. Errors on an empty `program` or if the program cannot be launched.
pub fn spawn_args(program: &str, args: &[String]) -> Result<Proc, StdError> {
    if program.trim().is_empty() {
        return Err(StdError::InvalidInput(
            "process spawn_args: empty program".into(),
        ));
    }
    let child = Command::new(program)
        .args(args)
        .spawn()
        .map_err(|e| StdError::Io(format!("process spawn_args `{program}`: {e}")))?;
    Ok(Proc(child))
}

/// Run a program with an explicit argv array to completion, capturing its
/// stdout, stderr, and exit code. Same literal-argv contract as [`spawn_args`].
/// Errors on an empty `program` or if the program cannot be launched.
pub fn output(program: &str, args: &[String]) -> Result<Output, StdError> {
    if program.trim().is_empty() {
        return Err(StdError::InvalidInput(
            "process output: empty program".into(),
        ));
    }
    let out = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| StdError::Io(format!("process output `{program}`: {e}")))?;
    Ok(Output {
        code: out.status.code(),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Portable, naive-split-safe commands per platform.
    fn ok_cmd() -> &'static str {
        if cfg!(windows) {
            "cmd /c exit 0"
        } else {
            "true"
        }
    }
    fn err_cmd() -> (&'static str, i32) {
        if cfg!(windows) {
            ("cmd /c exit 7", 7)
        } else {
            ("false", 1)
        }
    }

    #[test]
    fn spawn_wait_zero_exit() {
        let p = spawn(ok_cmd()).expect("spawn ok cmd");
        assert!(p.pid() > 0);
        let status = wait(p).expect("wait");
        assert_eq!(exit_code(&status), Some(0));
    }

    #[test]
    fn spawn_wait_nonzero_exit() {
        let (cmd, expected) = err_cmd();
        let p = spawn(cmd).expect("spawn err cmd");
        let status = wait(p).expect("wait");
        assert_eq!(exit_code(&status), Some(expected));
    }

    #[test]
    fn empty_command_is_error() {
        match spawn("   ") {
            Err(StdError::InvalidInput(_)) => {}
            other => panic!("expected InvalidInput, got {other:?}"),
        }
    }

    #[test]
    fn unknown_program_is_io_error() {
        match spawn("garnet_no_such_program_zzz_999") {
            Err(StdError::Io(_)) => {}
            other => panic!("expected Io error, got {other:?}"),
        }
    }

    // ── S23: structured argv + output capture ──

    /// Portable (program, argv) that echoes `marker` to stdout and exits 0.
    fn echo_cmd(marker: &str) -> (String, Vec<String>) {
        if cfg!(windows) {
            (
                "cmd".into(),
                vec!["/c".into(), "echo".into(), marker.into()],
            )
        } else {
            ("echo".into(), vec![marker.into()])
        }
    }

    #[test]
    fn output_captures_stdout_and_zero_exit() {
        let (prog, args) = echo_cmd("garnet-s23-out");
        let out = output(&prog, &args).expect("run echo");
        assert!(
            out.stdout().contains("garnet-s23-out"),
            "captured stdout should contain the marker: {:?}",
            out.stdout()
        );
        assert_eq!(out.code(), Some(0));
    }

    #[test]
    fn output_reports_nonzero_exit_code() {
        let (prog, args) = if cfg!(windows) {
            (
                "cmd".to_string(),
                vec!["/c".into(), "exit".into(), "7".into()],
            )
        } else {
            ("sh".to_string(), vec!["-c".into(), "exit 7".into()])
        };
        let out = output(&prog, &args).expect("run exit-7");
        assert_eq!(out.code(), Some(7));
    }

    #[test]
    fn spawn_args_spawns_waits_and_reports_exit() {
        let (prog, args) = echo_cmd("ignored");
        let p = spawn_args(&prog, &args).expect("spawn_args echo");
        assert!(p.pid() > 0);
        let status = wait(p).expect("wait");
        assert_eq!(exit_code(&status), Some(0));
    }

    /// On a POSIX host, prove a spaced argument is passed as ONE argv element:
    /// `printf "%s" "a b c"` prints `a b c`, whereas if the arg were re-split
    /// into three args, `printf` would reuse the `%s` format and print `abc`.
    #[cfg(unix)]
    #[test]
    fn spaced_argument_is_not_resplit() {
        let out = output("printf", &["%s".into(), "a b c".into()]).expect("run printf");
        assert_eq!(out.stdout(), "a b c");
    }

    #[test]
    fn empty_program_is_invalid_input() {
        match spawn_args("   ", &[]) {
            Err(StdError::InvalidInput(_)) => {}
            other => panic!("expected InvalidInput from spawn_args, got {other:?}"),
        }
        match output("", &["x".into()]) {
            Err(StdError::InvalidInput(_)) => {}
            other => panic!("expected InvalidInput from output, got {other:?}"),
        }
    }
}
