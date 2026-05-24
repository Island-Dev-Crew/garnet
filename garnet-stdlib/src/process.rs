//! `std::process` — child-process spawning (Layer 1, cap: `proc`).
//!
//! Host helpers over `std::process`. A Garnet function calling these must
//! declare `@caps(proc)`. The command line is split on ASCII whitespace
//! (no shell quoting — a documented v0.7 limitation; richer argv handling is
//! v0.8). `@stability(experimental)`.

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
}
