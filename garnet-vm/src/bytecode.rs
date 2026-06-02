use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq)]
pub enum Constant {
    Nil,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    Symbol(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOpcode {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOpcode {
    Neg,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    Const(u32),
    LoadGlobal(u32),
    LoadLocal(u16),
    StoreLocal(u16),
    Pop,
    Binary(BinaryOpcode),
    Unary(UnaryOpcode),
    Jump(usize),
    JumpIfFalse(usize),
    MakeArray(u16),
    IterInit,
    IterNext {
        iterator_slot: u16,
        item_slot: u16,
        jump_to: usize,
    },
    Call {
        name: u32,
        argc: u16,
    },
    CallMethod {
        name: u32,
        argc: u16,
    },
    Return,
}

impl Instruction {
    pub fn family(&self) -> &'static str {
        match self {
            Instruction::Const(_) => "Const",
            Instruction::LoadGlobal(_) => "LoadGlobal",
            Instruction::LoadLocal(_) => "LoadLocal",
            Instruction::StoreLocal(_) => "StoreLocal",
            Instruction::Pop => "Pop",
            Instruction::Binary(_) => "Binary",
            Instruction::Unary(_) => "Unary",
            Instruction::Jump(_) => "Jump",
            Instruction::JumpIfFalse(_) => "JumpIfFalse",
            Instruction::MakeArray(_) => "MakeArray",
            Instruction::IterInit => "IterInit",
            Instruction::IterNext { .. } => "IterNext",
            Instruction::Call { .. } => "Call",
            Instruction::CallMethod { .. } => "CallMethod",
            Instruction::Return => "Return",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BytecodeFunction {
    pub name: String,
    pub params: Vec<String>,
    pub locals: Vec<String>,
    pub instructions: Vec<Instruction>,
    pub native: bool,
    pub fallback_reason: Option<String>,
    /// The `@max_depth(N)` recursion ceiling declared on this function, if any
    /// (S99). Carried through compile + codec so the VM enforces the same
    /// ceiling the interpreter does. `None` means uncapped (recurses up to the
    /// VM frame stack); `Some(n)` traps at recursion depth `n + 1`.
    pub max_depth_ceiling: Option<i64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BytecodeProgram {
    pub constants: Vec<Constant>,
    pub functions: Vec<BytecodeFunction>,
}

impl BytecodeProgram {
    pub fn function(&self, name: &str) -> Option<&BytecodeFunction> {
        self.functions.iter().find(|function| function.name == name)
    }

    pub fn native_opcode_families(&self) -> BTreeSet<&'static str> {
        self.functions
            .iter()
            .filter(|function| function.native)
            .flat_map(|function| function.instructions.iter().map(Instruction::family))
            .collect()
    }

    pub fn native_instruction_count(&self) -> usize {
        self.functions
            .iter()
            .filter(|function| function.native)
            .map(|function| function.instructions.len())
            .sum()
    }
}
