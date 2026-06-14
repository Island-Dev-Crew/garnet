//! Garnet bytecode VM scaffold.
//!
//! S2 deliberately implements a narrow managed-mode subset and reports the
//! fallback boundary. It is not a production VM and does not replace the
//! tree-walk interpreter for unsupported language forms.

pub mod bytecode;
pub mod caps_recheck;
pub mod codec;
pub mod compiler;
pub mod vm;

pub use bytecode::{
    BinaryOpcode, BytecodeFunction, BytecodeProgram, Constant, Instruction, UnaryOpcode,
};
pub use caps_recheck::{compile_source_rechecked, recheck_artifact, recheck_caps, CapsLaundering};
pub use codec::{deserialize_program, serialize_program};
pub use compiler::{compile_source, CompileSummary, VmArtifact};
pub use vm::{
    run_function_with_options, run_source_with_options, ExecutionSummary, PreparedVm, RunOptions,
    VmRunResult,
};

use std::fmt;

#[derive(Debug, Clone)]
pub enum VmError {
    Parse(String),
    Compile(String),
    Codec(String),
    Runtime(String),
}

impl fmt::Display for VmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VmError::Parse(msg) => write!(f, "parse error: {msg}"),
            VmError::Compile(msg) => write!(f, "compile error: {msg}"),
            VmError::Codec(msg) => write!(f, "codec error: {msg}"),
            VmError::Runtime(msg) => write!(f, "runtime error: {msg}"),
        }
    }
}

impl std::error::Error for VmError {}

impl From<garnet_interp::RuntimeError> for VmError {
    fn from(value: garnet_interp::RuntimeError) -> Self {
        VmError::Runtime(value.to_string())
    }
}
