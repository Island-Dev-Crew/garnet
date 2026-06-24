use crate::bytecode::{
    BinaryOpcode, BytecodeFunction, BytecodeProgram, Constant, Instruction, UnaryOpcode,
};
use crate::VmError;
use garnet_parser::ast::{
    Annotation, AssignOp, BinOp, Block, ClosureBody, Expr, FnDef, Item, MatchArm, Module, Stmt,
    StringLit, UnOp,
};
use garnet_parser::token::StrPart;
use std::collections::{BTreeMap, HashSet};

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
        // S99: capture the `@max_depth(N)` ceiling before consuming `function`,
        // so both the native and fallback bytecode carry it (the VM enforces the
        // same ceiling the interpreter does; mirrors interp `eval::max_depth_ceiling`).
        let ceiling = max_depth_ceiling(&function.annotations);
        // RB-4 (VM⇄interp scope parity): the VM has no lexical scope frames yet
        // (one slot per NAME), so a nested `let`/`var`/`const`/`for`-var that
        // re-binds an ENCLOSING block's name overwrites the outer slot and cannot
        // restore it on block exit — `--vm` would diverge from `--interp` (the
        // reference). Until the VM grows scope frames (RB-5), force any function
        // whose body contains block-local/loop-local shadowing onto the tree-walk
        // fallback, which always agrees with the interpreter. Conservative by
        // design: over-fallback is sound; under-detection is not.
        let compile_result = if function_body_has_block_local_shadowing(&function) {
            Err(SHADOWING_FALLBACK_REASON.to_string())
        } else {
            let mut compiler = FunctionCompiler::new(&mut self.constants, &function);
            compiler
                .compile_body(&function.body)
                .map(|()| compiler.finish_native(function.name.clone(), ceiling))
        };
        match compile_result {
            Ok(native) => native,
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
                max_depth_ceiling: ceiling,
            },
        }
    }
}

/// The `@max_depth(N)` ceiling declared on a function, if any — mirrors the
/// interpreter's `eval::max_depth_ceiling` so the VM and interpreter agree on the
/// declared ceiling (S99 trap-parity).
fn max_depth_ceiling(annotations: &[Annotation]) -> Option<i64> {
    annotations.iter().find_map(|a| match a {
        Annotation::MaxDepth(n, _) => Some(*n),
        _ => None,
    })
}

/// Fallback reason recorded when a function is forced to tree-walk because its
/// body shadows an enclosing-scope binding (RB-4). The VM grows real lexical
/// scope frames in RB-5; until then this fallback is the only thing keeping
/// `--vm` in lockstep with `--interp` on this shape.
const SHADOWING_FALLBACK_REASON: &str =
    "block-local variable shadowing — falls back to tree-walk until the VM has lexical scope frames (RB-5)";

/// Returns `true` when `body` binds a name in some block that is ALREADY bound
/// in an ENCLOSING block scope (lexical shadowing). The VM's flat one-slot-per-
/// name model cannot represent this, so such a function must run on the tree-walk
/// interpreter for `--vm`⇄`--interp` parity (RB-4).
///
/// Detection is conservative: it walks every block-bearing and sub-expression
/// position so no nested block is missed. Same-scope rebinding (two `let x` in
/// the SAME block) is NOT shadowing and does not trigger — that compiles to VM
/// bytecode unchanged. Only ENCLOSING-scope shadowing triggers the fallback.
///
/// A function's parameters form the outermost scope frame: a `let` in the body
/// that re-binds a parameter name is enclosing-scope shadowing too, matching the
/// interpreter's nested-environment semantics.
pub(crate) fn body_has_block_local_shadowing(body: &Block) -> bool {
    let mut scopes: Vec<HashSet<String>> = vec![HashSet::new()];
    block_shadows(body, &mut scopes)
}

/// Like [`body_has_block_local_shadowing`] but seeds the outermost scope frame
/// with the function parameters, so a body `let` that re-binds a parameter name
/// is correctly detected as enclosing-scope shadowing (the param frame encloses
/// the body block). With no parameters this is exactly
/// [`body_has_block_local_shadowing`]. This is the entry `compile_function` uses.
fn function_body_has_block_local_shadowing(function: &FnDef) -> bool {
    if function.params.is_empty() {
        return body_has_block_local_shadowing(&function.body);
    }
    let mut scopes: Vec<HashSet<String>> = vec![HashSet::new()];
    for param in &function.params {
        scopes[0].insert(param.name.clone());
    }
    block_shadows(&function.body, &mut scopes)
}

/// Record a binder `name` introduced in the CURRENT (top) scope frame. Returns
/// `true` if that name already exists in any ENCLOSING (lower) frame — i.e. this
/// binder shadows an enclosing scope. The name is inserted into the current frame
/// regardless, so later same-scope rebinds compare against it without firing.
fn bind_name(name: &str, scopes: &mut [HashSet<String>]) -> bool {
    let shadows_enclosing = scopes
        .split_last()
        .map(|(_current, enclosing)| enclosing.iter().any(|frame| frame.contains(name)))
        .unwrap_or(false);
    if let Some(current) = scopes.last_mut() {
        current.insert(name.to_string());
    }
    shadows_enclosing
}

/// Walk a `Block`: push a fresh scope frame, walk its statements and tail
/// expression, then pop. Returns `true` if any binder inside shadows an
/// enclosing frame.
fn block_shadows(block: &Block, scopes: &mut Vec<HashSet<String>>) -> bool {
    scopes.push(HashSet::new());
    let mut found = false;
    for stmt in &block.stmts {
        if stmt_shadows(stmt, scopes) {
            found = true;
        }
    }
    if let Some(tail) = &block.tail_expr {
        if expr_shadows(tail, scopes) {
            found = true;
        }
    }
    scopes.pop();
    found
}

/// Walk a statement. Binders (`let`/`var`/`const`/`for`-var) add to the current
/// scope frame; every block-bearing and value position is recursed into.
fn stmt_shadows(stmt: &Stmt, scopes: &mut Vec<HashSet<String>>) -> bool {
    match stmt {
        Stmt::Let(decl) => {
            // The initializer is evaluated in the scope BEFORE the new binding,
            // so check it first, then record the binder.
            let value = expr_shadows(&decl.value, scopes);
            bind_name(&decl.name, scopes) || value
        }
        Stmt::Var(decl) => {
            let value = expr_shadows(&decl.value, scopes);
            bind_name(&decl.name, scopes) || value
        }
        Stmt::Const(decl) => {
            let value = expr_shadows(&decl.value, scopes);
            bind_name(&decl.name, scopes) || value
        }
        Stmt::Assign { target, value, .. } => {
            expr_shadows(target, scopes) || expr_shadows(value, scopes)
        }
        Stmt::While {
            condition, body, ..
        } => expr_shadows(condition, scopes) || block_shadows(body, scopes),
        Stmt::For {
            var, iter, body, ..
        } => {
            // `for` binds `var` in the loop body's scope frame. Evaluate the
            // iterator first (outer scope), then push the body frame and seed it
            // with `var` so a `var`-shadowed enclosing name is caught and so a
            // body re-bind of `var` is same-scope (not shadowing).
            if expr_shadows(iter, scopes) {
                return true;
            }
            scopes.push(HashSet::new());
            let mut found = bind_name(var, scopes);
            for stmt in &body.stmts {
                if stmt_shadows(stmt, scopes) {
                    found = true;
                }
            }
            if let Some(tail) = &body.tail_expr {
                if expr_shadows(tail, scopes) {
                    found = true;
                }
            }
            scopes.pop();
            found
        }
        Stmt::Loop { body, .. } => block_shadows(body, scopes),
        Stmt::Break { value, .. }
        | Stmt::Return { value, .. }
        | Stmt::Yield { value, .. }
        | Stmt::Next { value, .. } => value
            .as_ref()
            .map(|v| expr_shadows(v, scopes))
            .unwrap_or(false),
        Stmt::Raise { value, .. } => expr_shadows(value, scopes),
        Stmt::Continue { .. } => false,
        Stmt::Expr(expr) => expr_shadows(expr, scopes),
    }
}

/// Walk an expression, recursing into EVERY sub-expression and every nested
/// block-bearing form (`if`, `match`, `try`/`ensure`, closures, etc.) so no
/// nested block escapes the scope-stack walk.
fn expr_shadows(expr: &Expr, scopes: &mut Vec<HashSet<String>>) -> bool {
    match expr {
        Expr::Int(..)
        | Expr::Float(..)
        | Expr::Bool(..)
        | Expr::Nil(_)
        | Expr::Str(..)
        | Expr::Symbol(..)
        | Expr::Ident(..)
        | Expr::Path(..) => false,
        Expr::Binary { lhs, rhs, .. } => expr_shadows(lhs, scopes) || expr_shadows(rhs, scopes),
        Expr::Unary { expr, .. } => expr_shadows(expr, scopes),
        Expr::Call { callee, args, .. } => {
            expr_shadows(callee, scopes) || any_expr_shadows(args, scopes)
        }
        Expr::Method { receiver, args, .. } => {
            expr_shadows(receiver, scopes) || any_expr_shadows(args, scopes)
        }
        Expr::Field { receiver, .. } => expr_shadows(receiver, scopes),
        Expr::Index {
            receiver, index, ..
        } => expr_shadows(receiver, scopes) || expr_shadows(index, scopes),
        Expr::Cast { expr, .. } => expr_shadows(expr, scopes),
        Expr::If {
            condition,
            then_block,
            elsif_clauses,
            else_block,
            ..
        } => {
            if expr_shadows(condition, scopes) || block_shadows(then_block, scopes) {
                return true;
            }
            for (elsif_cond, elsif_block) in elsif_clauses {
                if expr_shadows(elsif_cond, scopes) || block_shadows(elsif_block, scopes) {
                    return true;
                }
            }
            else_block
                .as_ref()
                .map(|b| block_shadows(b, scopes))
                .unwrap_or(false)
        }
        Expr::Match { subject, arms, .. } => {
            if expr_shadows(subject, scopes) {
                return true;
            }
            arms.iter().any(|arm| arm_shadows(arm, scopes))
        }
        Expr::Try {
            body,
            rescues,
            ensure,
            ..
        } => {
            if block_shadows(body, scopes) {
                return true;
            }
            for rescue in rescues {
                if block_shadows(&rescue.body, scopes) {
                    return true;
                }
            }
            ensure
                .as_ref()
                .map(|b| block_shadows(b, scopes))
                .unwrap_or(false)
        }
        Expr::Closure { body, .. } => match body.as_ref() {
            ClosureBody::Block(block) => block_shadows(block, scopes),
            ClosureBody::Expr(expr) => expr_shadows(expr, scopes),
        },
        Expr::Spawn { expr, .. } => expr_shadows(expr, scopes),
        Expr::Array { elements, .. } => any_expr_shadows(elements, scopes),
        Expr::Map { entries, .. } => entries
            .iter()
            .any(|(k, v)| expr_shadows(k, scopes) || expr_shadows(v, scopes)),
    }
}

/// Walk a match arm. The arm's pattern-bound names and its body live in the same
/// nested scope frame; the guard is evaluated in that frame too. (Garnet match
/// patterns can bind names via `Pattern::Ident`/`Enum`/`Tuple`; those are arm-
/// local and do not shadow enclosing block bindings in a way the VM compiles —
/// the VM falls back on `Match` already — but we still recurse the bodies to
/// catch nested `let` shadowing inside an arm body.)
fn arm_shadows(arm: &MatchArm, scopes: &mut Vec<HashSet<String>>) -> bool {
    if let Some(guard) = &arm.guard {
        if expr_shadows(guard, scopes) {
            return true;
        }
    }
    block_shadows(&arm.body, scopes)
}

fn any_expr_shadows(exprs: &[Expr], scopes: &mut Vec<HashSet<String>>) -> bool {
    exprs.iter().any(|e| expr_shadows(e, scopes))
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

    fn finish_native(self, name: String, max_depth_ceiling: Option<i64>) -> BytecodeFunction {
        BytecodeFunction {
            name,
            params: self.params,
            locals: self.locals,
            instructions: self.instructions,
            native: true,
            fallback_reason: None,
            max_depth_ceiling,
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Parse `src` and return the body `Block` of its first `def`/`fn`.
    fn first_fn_body(src: &str) -> Block {
        let module = garnet_parser::parse_source(src).expect("parse");
        for item in module.items {
            if let Item::Fn(function) = item {
                return function.body;
            }
        }
        panic!("source had no function: {src}");
    }

    /// Run the shadowing detector over the first function's body.
    fn detects(src: &str) -> bool {
        body_has_block_local_shadowing(&first_fn_body(src))
    }

    /// Compile `src` and return whether its (single) function lowered to native
    /// bytecode (`true`) or fell back to the tree-walk interpreter (`false`).
    fn first_fn_is_native(src: &str) -> bool {
        let artifact = compile_source(src).expect("compile");
        artifact.program.functions[0].native
    }

    #[test]
    fn inner_let_rebinding_enclosing_name_is_shadowing() {
        // The canonical probe: outer `x`, inner block re-binds `x`.
        let src = "@caps()\ndef main() -> int {\n  let x = 1\n  if true { let x = 2  x }\n  x\n}\n";
        assert!(detects(src));
        assert!(!first_fn_is_native(src), "must fall back to tree-walk");
    }

    #[test]
    fn same_scope_rebinding_is_not_shadowing() {
        // Two `let x` in the SAME block is a rebind, not enclosing-scope shadowing.
        let src = "@caps()\ndef main() -> int {\n  let x = 1\n  let x = 2\n  x\n}\n";
        assert!(!detects(src));
        assert!(first_fn_is_native(src), "same-scope rebind stays native");
    }

    #[test]
    fn distinct_names_are_not_shadowing() {
        let src = "@caps()\ndef main() -> int {\n  let a = 1\n  let b = 2\n  a + b\n}\n";
        assert!(!detects(src));
        assert!(first_fn_is_native(src));
    }

    #[test]
    fn sibling_blocks_reusing_a_name_are_not_shadowing() {
        // Two sibling blocks each bind `y`, but neither encloses the other, and
        // `y` is not bound in the enclosing function-body scope → not shadowing.
        let src = "@caps()\ndef main() -> int {\n  if true { let y = 1  y }\n  if true { let y = 2  y }\n  0\n}\n";
        assert!(!detects(src));
    }

    #[test]
    fn nested_block_shadowing_via_loop_var_triggers() {
        // `n` is a parameter (outer frame); a `for n in ...` re-binds it.
        let src = "@caps()\ndef main(n) -> int {\n  for n in [1, 2, 3] { n }\n  n\n}\n";
        assert!(detects_with_params(src));
    }

    #[test]
    fn shadowing_buried_in_subexpression_is_detected() {
        // The shadowing block sits inside a call argument / binary position, not
        // as a top-level statement — the walk must still reach it.
        let src = "@caps()\ndef main() -> int {\n  let x = 1\n  let y = (if true { let x = 9  x } else { 0 }) + 1\n  x + y\n}\n";
        assert!(detects(src));
    }

    /// Parameter-aware detection (mirrors `compile_function`).
    fn detects_with_params(src: &str) -> bool {
        let module = garnet_parser::parse_source(src).expect("parse");
        for item in module.items {
            if let Item::Fn(function) = item {
                return function_body_has_block_local_shadowing(&function);
            }
        }
        panic!("no function");
    }

    #[test]
    fn body_let_rebinding_a_parameter_is_shadowing() {
        let src = "@caps()\ndef main(x) -> int {\n  let x = x + 1\n  x\n}\n";
        // Body block encloses nothing here, but the param frame DOES enclose the
        // body block, so a body `let x` shadows the parameter `x`.
        assert!(detects_with_params(src));
    }
}
