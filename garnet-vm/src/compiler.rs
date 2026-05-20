use crate::bytecode::{
    BinaryOpcode, BytecodeFunction, BytecodeProgram, Constant, Instruction, UnaryOpcode,
};
use crate::VmError;
use garnet_parser::ast::{
    AssignOp, BinOp, Block, ClosureBody, Expr, FnDef, Item, Module, Stmt, StringLit, UnOp,
};
use garnet_parser::token::StrPart;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct CompileSummary {
    pub native_functions: usize,
    pub fallback_functions: usize,
    pub native_opcode_families: Vec<String>,
    pub native_instruction_count: usize,
    pub fallback_reasons: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct VmArtifact {
    pub source: String,
    pub program: BytecodeProgram,
    pub summary: CompileSummary,
}

pub fn compile_source(src: &str) -> Result<VmArtifact, VmError> {
    let module =
        garnet_parser::parse_source(src).map_err(|error| VmError::Parse(format!("{error:?}")))?;
    let mut compiler = ModuleCompiler::default();
    let program = compiler.compile_module(module);
    let summary = summarize(&program);
    Ok(VmArtifact {
        source: src.to_string(),
        program,
        summary,
    })
}

fn summarize(program: &BytecodeProgram) -> CompileSummary {
    let native_functions = program
        .functions
        .iter()
        .filter(|function| function.native)
        .count();
    let fallback_reasons = program
        .functions
        .iter()
        .filter_map(|function| {
            function
                .fallback_reason
                .as_ref()
                .map(|reason| format!("{}: {reason}", function.name))
        })
        .collect::<Vec<_>>();
    let mut native_opcode_families = program
        .native_opcode_families()
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
    native_opcode_families.sort();
    CompileSummary {
        native_functions,
        fallback_functions: program.functions.len().saturating_sub(native_functions),
        native_opcode_families,
        native_instruction_count: program.native_instruction_count(),
        fallback_reasons,
    }
}

#[derive(Default)]
struct ModuleCompiler {
    constants: Vec<Constant>,
}

impl ModuleCompiler {
    fn compile_module(&mut self, module: Module) -> BytecodeProgram {
        let mut functions = Vec::new();
        for item in module.items {
            if let Item::Fn(function) = item {
                functions.push(self.compile_function(function));
            }
        }
        BytecodeProgram {
            constants: self.constants.clone(),
            functions,
        }
    }

    fn compile_function(&mut self, function: FnDef) -> BytecodeFunction {
        let mut compiler = FunctionCompiler::new(&mut self.constants, &function);
        match compiler.compile_body(&function.body) {
            Ok(()) => compiler.finish_native(function.name),
            Err(reason) => BytecodeFunction {
                name: function.name,
                params: function
                    .params
                    .into_iter()
                    .map(|param| param.name)
                    .collect(),
                locals: Vec::new(),
                instructions: Vec::new(),
                native: false,
                fallback_reason: Some(reason),
            },
        }
    }
}

struct FunctionCompiler<'a> {
    constants: &'a mut Vec<Constant>,
    params: Vec<String>,
    locals: Vec<String>,
    local_slots: BTreeMap<String, u16>,
    instructions: Vec<Instruction>,
    hidden_counter: usize,
}

impl<'a> FunctionCompiler<'a> {
    fn new(constants: &'a mut Vec<Constant>, function: &FnDef) -> Self {
        let mut local_slots = BTreeMap::new();
        let mut locals = Vec::new();
        let mut params = Vec::new();
        for param in &function.params {
            let slot = locals.len() as u16;
            local_slots.insert(param.name.clone(), slot);
            locals.push(param.name.clone());
            params.push(param.name.clone());
        }
        Self {
            constants,
            params,
            locals,
            local_slots,
            instructions: Vec::new(),
            hidden_counter: 0,
        }
    }

    fn compile_body(&mut self, body: &Block) -> Result<(), String> {
        self.compile_block_value(body)?;
        self.emit(Instruction::Return);
        Ok(())
    }

    fn finish_native(self, name: String) -> BytecodeFunction {
        BytecodeFunction {
            name,
            params: self.params,
            locals: self.locals,
            instructions: self.instructions,
            native: true,
            fallback_reason: None,
        }
    }

    fn compile_block_value(&mut self, block: &Block) -> Result<(), String> {
        for stmt in &block.stmts {
            self.compile_stmt(stmt)?;
        }
        if let Some(tail) = &block.tail_expr {
            self.compile_expr(tail)
        } else {
            self.emit_const(Constant::Nil);
            Ok(())
        }
    }

    fn compile_block_stmt(&mut self, block: &Block) -> Result<(), String> {
        for stmt in &block.stmts {
            self.compile_stmt(stmt)?;
        }
        if let Some(tail) = &block.tail_expr {
            self.compile_expr(tail)?;
            self.emit(Instruction::Pop);
        }
        Ok(())
    }

    fn compile_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Let(decl) => {
                self.compile_expr(&decl.value)?;
                let slot = self.define_local(&decl.name);
                self.emit(Instruction::StoreLocal(slot));
                Ok(())
            }
            Stmt::Var(decl) => {
                self.compile_expr(&decl.value)?;
                let slot = self.define_local(&decl.name);
                self.emit(Instruction::StoreLocal(slot));
                Ok(())
            }
            Stmt::Const(decl) => {
                self.compile_expr(&decl.value)?;
                let slot = self.define_local(&decl.name);
                self.emit(Instruction::StoreLocal(slot));
                Ok(())
            }
            Stmt::Assign {
                target, op, value, ..
            } => self.compile_assign(target, *op, value),
            Stmt::While {
                condition, body, ..
            } => self.compile_while(condition, body),
            Stmt::For {
                var, iter, body, ..
            } => self.compile_for(var, iter, body),
            Stmt::Return { value, .. } => {
                if let Some(value) = value {
                    self.compile_expr(value)?;
                } else {
                    self.emit_const(Constant::Nil);
                }
                self.emit(Instruction::Return);
                Ok(())
            }
            Stmt::Expr(expr) => {
                self.compile_expr(expr)?;
                self.emit(Instruction::Pop);
                Ok(())
            }
            Stmt::Loop { .. }
            | Stmt::Break { .. }
            | Stmt::Continue { .. }
            | Stmt::Yield { .. }
            | Stmt::Next { .. }
            | Stmt::Raise { .. } => Err("statement form falls back to tree-walk".to_string()),
        }
    }

    fn compile_assign(&mut self, target: &Expr, op: AssignOp, value: &Expr) -> Result<(), String> {
        let Expr::Ident(name, _) = target else {
            return Err("non-identifier assignment falls back to tree-walk".to_string());
        };
        let slot = self
            .lookup_local(name)
            .ok_or_else(|| format!("assignment target `{name}` is not a VM local"))?;
        if op == AssignOp::Eq {
            self.compile_expr(value)?;
            self.emit(Instruction::StoreLocal(slot));
            return Ok(());
        }
        self.emit(Instruction::LoadLocal(slot));
        self.compile_expr(value)?;
        let binary = match op {
            AssignOp::PlusEq => BinaryOpcode::Add,
            AssignOp::MinusEq => BinaryOpcode::Sub,
            AssignOp::StarEq => BinaryOpcode::Mul,
            AssignOp::SlashEq => BinaryOpcode::Div,
            AssignOp::PercentEq => BinaryOpcode::Mod,
            AssignOp::Eq => unreachable!(),
        };
        self.emit(Instruction::Binary(binary));
        self.emit(Instruction::StoreLocal(slot));
        Ok(())
    }

    fn compile_while(&mut self, condition: &Expr, body: &Block) -> Result<(), String> {
        let loop_start = self.instructions.len();
        self.compile_expr(condition)?;
        let exit_jump = self.emit(Instruction::JumpIfFalse(usize::MAX));
        self.compile_block_stmt(body)?;
        self.emit(Instruction::Jump(loop_start));
        let exit = self.instructions.len();
        self.patch_jump(exit_jump, exit);
        Ok(())
    }

    fn compile_for(&mut self, var: &str, iter: &Expr, body: &Block) -> Result<(), String> {
        self.compile_expr(iter)?;
        self.emit(Instruction::IterInit);
        let iterator_name = self.next_hidden("iter");
        let iterator_slot = self.define_local(&iterator_name);
        self.emit(Instruction::StoreLocal(iterator_slot));
        let item_slot = self.define_local(var);
        let loop_start = self.instructions.len();
        let next = self.emit(Instruction::IterNext {
            iterator_slot,
            item_slot,
            jump_to: usize::MAX,
        });
        self.compile_block_stmt(body)?;
        self.emit(Instruction::Jump(loop_start));
        let exit = self.instructions.len();
        self.patch_iter_next(next, exit);
        Ok(())
    }

    fn compile_expr(&mut self, expr: &Expr) -> Result<(), String> {
        match expr {
            Expr::Int(value, _) => {
                self.emit_const(Constant::Int(*value));
                Ok(())
            }
            Expr::Float(value, _) => {
                self.emit_const(Constant::Float(*value));
                Ok(())
            }
            Expr::Bool(value, _) => {
                self.emit_const(Constant::Bool(*value));
                Ok(())
            }
            Expr::Nil(_) => {
                self.emit_const(Constant::Nil);
                Ok(())
            }
            Expr::Symbol(value, _) => {
                self.emit_const(Constant::Symbol(value.clone()));
                Ok(())
            }
            Expr::Str(lit, _) => self.compile_string(lit),
            Expr::Ident(name, _) => {
                if let Some(slot) = self.lookup_local(name) {
                    self.emit(Instruction::LoadLocal(slot));
                } else {
                    let constant = self.intern(Constant::Str(name.clone()));
                    self.emit(Instruction::LoadGlobal(constant));
                }
                Ok(())
            }
            Expr::Binary { op, lhs, rhs, .. } => {
                let op = map_binary(*op)?;
                self.compile_expr(lhs)?;
                self.compile_expr(rhs)?;
                self.emit(Instruction::Binary(op));
                Ok(())
            }
            Expr::Unary { op, expr, .. } => {
                let op = map_unary(*op)?;
                self.compile_expr(expr)?;
                self.emit(Instruction::Unary(op));
                Ok(())
            }
            Expr::Call { callee, args, .. } => {
                let Expr::Ident(name, _) = callee.as_ref() else {
                    return Err("non-identifier call falls back to tree-walk".to_string());
                };
                for arg in args {
                    self.compile_expr(arg)?;
                }
                let name = self.intern(Constant::Str(name.clone()));
                self.emit(Instruction::Call {
                    name,
                    argc: args.len() as u16,
                });
                Ok(())
            }
            Expr::Method {
                receiver,
                method,
                args,
                ..
            } => {
                if !matches!(method.as_str(), "len" | "length" | "size" | "count")
                    || !args.is_empty()
                {
                    return Err("method call falls back to tree-walk".to_string());
                }
                self.compile_expr(receiver)?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                let method = self.intern(Constant::Str(method.clone()));
                self.emit(Instruction::CallMethod {
                    name: method,
                    argc: args.len() as u16,
                });
                Ok(())
            }
            Expr::If {
                condition,
                then_block,
                elsif_clauses,
                else_block,
                ..
            } => self.compile_if_chain(condition, then_block, elsif_clauses, else_block.as_ref()),
            Expr::Array { elements, .. } => {
                for element in elements {
                    self.compile_expr(element)?;
                }
                self.emit(Instruction::MakeArray(elements.len() as u16));
                Ok(())
            }
            Expr::Closure { body, .. } => match body.as_ref() {
                ClosureBody::Block(_) | ClosureBody::Expr(_) => {
                    Err("closures fall back to tree-walk".to_string())
                }
            },
            Expr::Path(_, _)
            | Expr::Field { .. }
            | Expr::Index { .. }
            | Expr::Cast { .. }
            | Expr::Match { .. }
            | Expr::Try { .. }
            | Expr::Spawn { .. }
            | Expr::Map { .. } => Err("expression form falls back to tree-walk".to_string()),
        }
    }

    fn compile_if_chain(
        &mut self,
        condition: &Expr,
        then_block: &Block,
        elsif_clauses: &[(Expr, Block)],
        else_block: Option<&Block>,
    ) -> Result<(), String> {
        self.compile_expr(condition)?;
        let else_jump = self.emit(Instruction::JumpIfFalse(usize::MAX));
        self.compile_block_value(then_block)?;
        let end_jump = self.emit(Instruction::Jump(usize::MAX));
        let else_start = self.instructions.len();
        self.patch_jump(else_jump, else_start);
        if let Some((elsif_condition, elsif_block)) = elsif_clauses.first() {
            self.compile_if_chain(
                elsif_condition,
                elsif_block,
                &elsif_clauses[1..],
                else_block,
            )?;
        } else if let Some(else_block) = else_block {
            self.compile_block_value(else_block)?;
        } else {
            self.emit_const(Constant::Nil);
        }
        let end = self.instructions.len();
        self.patch_jump(end_jump, end);
        Ok(())
    }

    fn compile_string(&mut self, lit: &StringLit) -> Result<(), String> {
        let mut out = String::new();
        for part in &lit.parts {
            match part {
                StrPart::Lit(value) => out.push_str(value),
                StrPart::Interp(_) => {
                    return Err("interpolated strings fall back to tree-walk".to_string())
                }
            }
        }
        self.emit_const(Constant::Str(out));
        Ok(())
    }

    fn emit_const(&mut self, constant: Constant) {
        let index = self.intern(constant);
        self.emit(Instruction::Const(index));
    }

    fn intern(&mut self, constant: Constant) -> u32 {
        if let Some(index) = self
            .constants
            .iter()
            .position(|existing| existing == &constant)
        {
            return index as u32;
        }
        self.constants.push(constant);
        (self.constants.len() - 1) as u32
    }

    fn emit(&mut self, instruction: Instruction) -> usize {
        let index = self.instructions.len();
        self.instructions.push(instruction);
        index
    }

    fn patch_jump(&mut self, index: usize, target: usize) {
        match &mut self.instructions[index] {
            Instruction::Jump(value) | Instruction::JumpIfFalse(value) => *value = target,
            other => panic!("cannot patch non-jump instruction: {other:?}"),
        }
    }

    fn patch_iter_next(&mut self, index: usize, target: usize) {
        match &mut self.instructions[index] {
            Instruction::IterNext { jump_to, .. } => *jump_to = target,
            other => panic!("cannot patch non-iterator instruction: {other:?}"),
        }
    }

    fn define_local(&mut self, name: &str) -> u16 {
        if let Some(slot) = self.local_slots.get(name) {
            return *slot;
        }
        let slot = self.locals.len() as u16;
        self.locals.push(name.to_string());
        self.local_slots.insert(name.to_string(), slot);
        slot
    }

    fn lookup_local(&self, name: &str) -> Option<u16> {
        self.local_slots.get(name).copied()
    }

    fn next_hidden(&mut self, prefix: &str) -> String {
        let value = format!("#{prefix}_{}", self.hidden_counter);
        self.hidden_counter += 1;
        value
    }
}

fn map_binary(op: BinOp) -> Result<BinaryOpcode, String> {
    match op {
        BinOp::Add => Ok(BinaryOpcode::Add),
        BinOp::Sub => Ok(BinaryOpcode::Sub),
        BinOp::Mul => Ok(BinaryOpcode::Mul),
        BinOp::Div => Ok(BinaryOpcode::Div),
        BinOp::Mod => Ok(BinaryOpcode::Mod),
        BinOp::Eq => Ok(BinaryOpcode::Eq),
        BinOp::NotEq => Ok(BinaryOpcode::NotEq),
        BinOp::Lt => Ok(BinaryOpcode::Lt),
        BinOp::Gt => Ok(BinaryOpcode::Gt),
        BinOp::LtEq => Ok(BinaryOpcode::LtEq),
        BinOp::GtEq => Ok(BinaryOpcode::GtEq),
        BinOp::And | BinOp::Or | BinOp::Pipeline | BinOp::Range | BinOp::RangeInclusive => {
            Err("binary operator falls back to tree-walk".to_string())
        }
    }
}

fn map_unary(op: UnOp) -> Result<UnaryOpcode, String> {
    match op {
        UnOp::Neg => Ok(UnaryOpcode::Neg),
        UnOp::Not => Ok(UnaryOpcode::Not),
        UnOp::Question => Err("postfix question operator falls back to tree-walk".to_string()),
    }
}
