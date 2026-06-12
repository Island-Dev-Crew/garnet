use crate::bytecode::{
    BinaryOpcode, BytecodeFunction, BytecodeProgram, Constant, Instruction, UnaryOpcode,
};
use crate::compiler::{compile_source, VmArtifact};
use crate::VmError;
use garnet_interp::{Interpreter, Value};
use std::cell::RefCell;
use std::collections::BTreeMap;

thread_local! {
    /// Per-function active recursion depth for `@max_depth` enforcement on the
    /// VM's native path (S99). VM-local — deliberately NOT shared with the
    /// interpreter's counter (a native frame that itself falls back would
    /// otherwise be double-counted). The counter unwinds via `VmDepthGuard` on
    /// every frame drop / error path, so it starts clean each run.
    static VM_MAX_DEPTH_DEPTHS: RefCell<BTreeMap<String, u64>> = const { RefCell::new(BTreeMap::new()) };
}

/// RAII guard mirroring the interpreter's `MaxDepthGuard` (`eval.rs`): increments
/// the per-function counter on `enter`, decrements on drop. Because the VM uses an
/// explicit heap frame stack rather than host recursion, the guard is stored on
/// the owning `Frame` so the `Vec<Frame>` drop unwinds every live counter on any
/// return or early-error (trap) path.
struct VmDepthGuard {
    name: String,
    depth: u64,
}

impl VmDepthGuard {
    fn enter(name: String) -> Self {
        let depth = VM_MAX_DEPTH_DEPTHS.with(|m| {
            let mut m = m.borrow_mut();
            let c = m.entry(name.clone()).or_insert(0);
            *c += 1;
            *c
        });
        VmDepthGuard { name, depth }
    }
}

impl Drop for VmDepthGuard {
    fn drop(&mut self) {
        VM_MAX_DEPTH_DEPTHS.with(|m| {
            if let Some(c) = m.borrow_mut().get_mut(&self.name) {
                *c = c.saturating_sub(1);
            }
        });
    }
}

/// Enter the `@max_depth(N)` guard for `function` if it declares a ceiling,
/// trapping with the exact interpreter message when the recursion depth exceeds
/// `N` (S99 trap-parity). Returns `Ok(None)` for uncapped functions. On a trap the
/// freshly-entered guard drops here (rolling back its own increment); the caller's
/// already-pushed frames unwind their guards as the `Vec<Frame>` drops.
fn enter_depth_guard(function: &BytecodeFunction) -> Result<Option<VmDepthGuard>, VmError> {
    match function.max_depth_ceiling {
        Some(n) => {
            let guard = VmDepthGuard::enter(function.name.clone());
            if guard.depth > n.max(0) as u64 {
                return Err(VmError::Runtime(format!(
                    "bounded: @max_depth({n}) exceeded for `{}` (recursion depth {})",
                    function.name, guard.depth
                )));
            }
            Ok(Some(guard))
        }
        None => Ok(None),
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RunOptions {
    pub emit_stdout: bool,
}

impl Default for RunOptions {
    fn default() -> Self {
        Self { emit_stdout: true }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ExecutionSummary {
    pub native_function_calls: usize,
    pub fallback_function_calls: usize,
    pub native_instruction_count: usize,
    pub fallback_reasons: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct VmRunResult {
    pub value: Value,
    pub called_entry: bool,
    pub summary: ExecutionSummary,
}

pub struct PreparedVm<'a> {
    engine: VmEngine<'a>,
}

impl<'a> PreparedVm<'a> {
    pub fn new(artifact: &'a VmArtifact, options: RunOptions) -> Result<Self, VmError> {
        Ok(Self {
            engine: VmEngine::new(artifact, options)?,
        })
    }

    pub fn call_function(&mut self, name: &str, args: Vec<Value>) -> Result<Value, VmError> {
        self.engine.call_function(name, args)
    }

    pub fn summary(&self) -> &ExecutionSummary {
        &self.engine.summary
    }
}

pub fn run_source_with_options(src: &str, options: RunOptions) -> Result<VmRunResult, VmError> {
    let artifact = compile_source(src)?;
    if artifact.program.function("main").is_none() {
        return Ok(VmRunResult {
            value: Value::Nil,
            called_entry: false,
            summary: ExecutionSummary::default(),
        });
    }
    run_function_artifact(&artifact, "main", Vec::new(), options).map(|mut result| {
        result.called_entry = true;
        result
    })
}

pub fn run_function_with_options(
    src: &str,
    name: &str,
    args: Vec<Value>,
    options: RunOptions,
) -> Result<VmRunResult, VmError> {
    let artifact = compile_source(src)?;
    run_function_artifact(&artifact, name, args, options)
}

fn run_function_artifact(
    artifact: &VmArtifact,
    name: &str,
    args: Vec<Value>,
    options: RunOptions,
) -> Result<VmRunResult, VmError> {
    let mut engine = VmEngine::new(artifact, options)?;
    let value = engine.call_function(name, args)?;
    Ok(VmRunResult {
        value,
        called_entry: true,
        summary: engine.summary,
    })
}

struct VmEngine<'a> {
    program: &'a BytecodeProgram,
    fallback: Interpreter,
    options: RunOptions,
    summary: ExecutionSummary,
}

#[derive(Clone)]
enum Slot {
    Empty,
    Runtime(Value),
    Iterator(VmIterator),
}

#[derive(Clone)]
struct VmIterator {
    items: Vec<Value>,
    index: usize,
}

/// One activation record on the explicit call-frame stack. Holds the index
/// of the executing function (not a borrow, to keep `&'a BytecodeProgram`
/// decoupled from `&mut self`), its local slots, its operand stack, and the
/// instruction pointer to resume at.
struct Frame {
    function_idx: usize,
    locals: Vec<Slot>,
    stack: Vec<Slot>,
    ip: usize,
    /// The `@max_depth` recursion guard owned by this activation (S99). Its `Drop`
    /// decrements the per-function depth counter when the frame is popped or the
    /// whole frame stack unwinds on a trap. `None` for uncapped functions.
    depth_guard: Option<VmDepthGuard>,
}

impl Frame {
    fn new(
        function_idx: usize,
        function: &BytecodeFunction,
        args: Vec<Value>,
    ) -> Result<Self, VmError> {
        if args.len() != function.params.len() {
            return Err(VmError::Runtime(format!(
                "{}: arity mismatch (expected {}, got {})",
                function.name,
                function.params.len(),
                args.len()
            )));
        }
        let mut locals = vec![Slot::Empty; function.locals.len()];
        for (index, arg) in args.into_iter().enumerate() {
            locals[index] = Slot::Runtime(arg);
        }
        Ok(Self {
            function_idx,
            locals,
            stack: Vec::new(),
            ip: 0,
            depth_guard: None,
        })
    }
}

impl<'a> VmEngine<'a> {
    fn new(artifact: &'a VmArtifact, options: RunOptions) -> Result<Self, VmError> {
        let mut fallback = Interpreter::new();
        fallback.load_source(&artifact.source)?;
        Ok(Self {
            program: &artifact.program,
            fallback,
            options,
            summary: ExecutionSummary::default(),
        })
    }

    fn call_function(&mut self, name: &str, args: Vec<Value>) -> Result<Value, VmError> {
        // S100: install the program-entry `@caps` frame for the whole run, mirroring
        // `--interp`'s `call_entry`. Without it the VM's fallback path runs with no
        // entry frame, so the S92 program-entry capability gate is bypassed — an
        // undeclared subprocess capability laundered through a helper that declares
        // `@caps(proc)` would trap under `--interp` but be allowed under `--vm`. The
        // scope is owned (it registers in the interpreter's thread-local caps
        // context, shared because the VM and its fallback run on the same thread)
        // and unwinds the entry frame when this call returns or traps.
        let _entry_caps = self.fallback.enter_entry_caps_frame(name);
        let Some(idx) = self.program.functions.iter().position(|f| f.name == name) else {
            return self.call_fallback(name, args, "function not present in VM program");
        };
        let function = &self.program.functions[idx];
        if !function.native {
            let reason = function
                .fallback_reason
                .clone()
                .unwrap_or_else(|| "function marked fallback".to_string());
            return self.call_fallback(name, args, &reason);
        }
        self.summary.native_function_calls += 1;
        self.run_frames(idx, args)
    }

    fn call_fallback(
        &mut self,
        name: &str,
        args: Vec<Value>,
        reason: &str,
    ) -> Result<Value, VmError> {
        self.summary.fallback_function_calls += 1;
        self.summary
            .fallback_reasons
            .push(format!("{name}: {reason}"));
        self.fallback.call(name, args).map_err(VmError::from)
    }

    /// Execute a native function with an explicit, heap-allocated call-frame
    /// stack. Native callees push a new `Frame` instead of recursing in the
    /// host (Rust) language, so deep Garnet recursion runs on the heap and
    /// does not overflow the Rust stack. Builtins and fallback callees execute
    /// inline; the fallback path delegates to the tree-walk interpreter (its
    /// own recursion domain — out of scope to flatten here).
    fn run_frames(&mut self, entry_idx: usize, args: Vec<Value>) -> Result<Value, VmError> {
        // Copy the shared program reference into a local so indexing it does
        // not entangle the `&mut self` borrows used for summary/fallback.
        let program = self.program;
        // S99: guard the entry activation when the entry function itself declares
        // `@max_depth` (e.g. the annotated function is run directly as the entry).
        let entry_function = &program.functions[entry_idx];
        let entry_guard = enter_depth_guard(entry_function)?;
        let mut entry_frame = Frame::new(entry_idx, entry_function, args)?;
        entry_frame.depth_guard = entry_guard;
        let mut frames: Vec<Frame> = vec![entry_frame];

        loop {
            let top = frames.len() - 1;
            let func = &program.functions[frames[top].function_idx];

            if frames[top].ip >= func.instructions.len() {
                // Fell off the end of the body: implicit Nil return.
                frames.pop();
                match frames.last_mut() {
                    Some(caller) => caller.stack.push(Slot::Runtime(Value::Nil)),
                    None => return Ok(Value::Nil),
                }
                continue;
            }

            // Clone the current instruction to release the `program` borrow
            // before any `&mut self` dispatch below.
            let instruction = func.instructions[frames[top].ip].clone();
            self.summary.native_instruction_count += 1;

            match instruction {
                Instruction::Call { name, argc } => {
                    let callee = self.constant_string(name)?.to_string();
                    let call_args = pop_args(&mut frames[top].stack, argc)?;
                    // Advance the caller past the call so it resumes correctly
                    // once the callee returns.
                    frames[top].ip += 1;

                    if let Some(value) = self.try_builtin(&callee, &call_args)? {
                        frames[top].stack.push(Slot::Runtime(value));
                        continue;
                    }

                    if let Some(callee_idx) = program
                        .functions
                        .iter()
                        .position(|f| f.name == callee && f.native)
                    {
                        // S99: enforce the callee's `@max_depth` ceiling before
                        // pushing its frame. A trap returns `Err` here (the callee
                        // frame is never pushed), and the in-flight `Vec<Frame>`
                        // unwinds every live depth guard as it drops — exactly as
                        // the interpreter traps at recursion depth `N + 1`.
                        let callee_function = &program.functions[callee_idx];
                        let guard = enter_depth_guard(callee_function)?;
                        self.summary.native_function_calls += 1;
                        let mut frame = Frame::new(callee_idx, callee_function, call_args)?;
                        frame.depth_guard = guard;
                        frames.push(frame);
                        continue;
                    }

                    let value =
                        self.call_fallback(&callee, call_args, "callee falls back to tree-walk")?;
                    frames[top].stack.push(Slot::Runtime(value));
                }
                Instruction::Return => {
                    let value = match frames[top].stack.pop() {
                        Some(Slot::Runtime(value)) => value,
                        Some(Slot::Empty) | Some(Slot::Iterator(_)) | None => Value::Nil,
                    };
                    frames.pop();
                    match frames.last_mut() {
                        Some(caller) => caller.stack.push(Slot::Runtime(value)),
                        None => return Ok(value),
                    }
                }
                other => self.step(&mut frames[top], &other)?,
            }
        }
    }

    /// Execute one non-call, non-return instruction against a single frame.
    fn step(&mut self, frame: &mut Frame, instruction: &Instruction) -> Result<(), VmError> {
        match instruction {
            Instruction::Const(index) => {
                frame.stack.push(Slot::Runtime(self.constant(*index)?));
                frame.ip += 1;
            }
            Instruction::LoadGlobal(index) => {
                let name = self.constant_string(*index)?.to_string();
                let value = self
                    .fallback
                    .global
                    .get(&name)
                    .ok_or_else(|| VmError::Runtime(format!("undefined global: {name}")))?;
                frame.stack.push(Slot::Runtime(value));
                frame.ip += 1;
            }
            Instruction::LoadLocal(slot) => {
                frame
                    .stack
                    .push(Slot::Runtime(runtime_slot(&frame.locals, *slot)?.clone()));
                frame.ip += 1;
            }
            Instruction::StoreLocal(slot) => {
                let value = frame
                    .stack
                    .pop()
                    .ok_or_else(|| VmError::Runtime("stack underflow".to_string()))?;
                store_slot(&mut frame.locals, *slot, value)?;
                frame.ip += 1;
            }
            Instruction::Pop => {
                frame.stack.pop();
                frame.ip += 1;
            }
            Instruction::Binary(op) => {
                let rhs = pop_runtime(&mut frame.stack)?;
                let lhs = pop_runtime(&mut frame.stack)?;
                frame
                    .stack
                    .push(Slot::Runtime(apply_binary(*op, lhs, rhs)?));
                frame.ip += 1;
            }
            Instruction::Unary(op) => {
                let value = pop_runtime(&mut frame.stack)?;
                frame.stack.push(Slot::Runtime(apply_unary(*op, value)?));
                frame.ip += 1;
            }
            Instruction::Jump(target) => {
                frame.ip = *target;
            }
            Instruction::JumpIfFalse(target) => {
                let value = pop_runtime(&mut frame.stack)?;
                if !value.truthy() {
                    frame.ip = *target;
                } else {
                    frame.ip += 1;
                }
            }
            Instruction::MakeArray(count) => {
                let mut values = Vec::with_capacity(*count as usize);
                for _ in 0..*count {
                    values.push(pop_runtime(&mut frame.stack)?);
                }
                values.reverse();
                frame.stack.push(Slot::Runtime(Value::array(values)));
                frame.ip += 1;
            }
            Instruction::IterInit => {
                let value = pop_runtime(&mut frame.stack)?;
                frame.stack.push(Slot::Iterator(VmIterator {
                    items: materialize_iter(&value)?,
                    index: 0,
                }));
                frame.ip += 1;
            }
            Instruction::IterNext {
                iterator_slot,
                item_slot,
                jump_to,
            } => {
                let iterator = iterator_slot_mut(&mut frame.locals, *iterator_slot)?;
                if iterator.index >= iterator.items.len() {
                    frame.ip = *jump_to;
                } else {
                    let item = iterator.items[iterator.index].clone();
                    iterator.index += 1;
                    store_slot(&mut frame.locals, *item_slot, Slot::Runtime(item))?;
                    frame.ip += 1;
                }
            }
            Instruction::CallMethod { name, argc } => {
                let name = self.constant_string(*name)?.to_string();
                let args = pop_args(&mut frame.stack, *argc)?;
                let receiver = pop_runtime(&mut frame.stack)?;
                let value = call_method(&receiver, &name, args)?;
                frame.stack.push(Slot::Runtime(value));
                frame.ip += 1;
            }
            Instruction::Call { .. } | Instruction::Return => {
                unreachable!("Call/Return are handled by run_frames, not step")
            }
        }
        Ok(())
    }

    /// Resolve a builtin call. Returns `Some(value)` for a recognized builtin
    /// (`println`/`print`/`len`), `None` for anything else (a user function,
    /// resolved by `run_frames` as a native frame or a tree-walk fallback).
    fn try_builtin(&mut self, name: &str, args: &[Value]) -> Result<Option<Value>, VmError> {
        match name {
            "println" => {
                if self.options.emit_stdout {
                    let rendered = args.iter().map(Value::display).collect::<Vec<_>>();
                    println!("{}", rendered.join(" "));
                }
                Ok(Some(Value::Nil))
            }
            "print" => {
                if self.options.emit_stdout {
                    let rendered = args.iter().map(Value::display).collect::<Vec<_>>();
                    print!("{}", rendered.join(" "));
                }
                Ok(Some(Value::Nil))
            }
            "len" => {
                let value = args
                    .first()
                    .ok_or_else(|| VmError::Runtime("len: missing arg".to_string()))?;
                Ok(Some(len_value(value)?))
            }
            _ => Ok(None),
        }
    }

    fn constant(&self, index: u32) -> Result<Value, VmError> {
        let constant = self
            .program
            .constants
            .get(index as usize)
            .ok_or_else(|| VmError::Runtime(format!("constant index out of range: {index}")))?;
        Ok(match constant {
            Constant::Nil => Value::Nil,
            Constant::Bool(value) => Value::Bool(*value),
            Constant::Int(value) => Value::Int(*value),
            Constant::Float(value) => Value::Float(*value),
            Constant::Str(value) => Value::str(value.clone()),
            Constant::Symbol(value) => Value::sym(value.clone()),
        })
    }

    fn constant_string(&self, index: u32) -> Result<&str, VmError> {
        match self.program.constants.get(index as usize) {
            Some(Constant::Str(value)) => Ok(value),
            Some(_) => Err(VmError::Runtime(format!(
                "constant {index} is not a string"
            ))),
            None => Err(VmError::Runtime(format!(
                "constant index out of range: {index}"
            ))),
        }
    }
}

fn store_slot(locals: &mut [Slot], slot: u16, value: Slot) -> Result<(), VmError> {
    let target = locals
        .get_mut(slot as usize)
        .ok_or_else(|| VmError::Runtime(format!("local slot out of range: {slot}")))?;
    *target = value;
    Ok(())
}

fn runtime_slot(locals: &[Slot], slot: u16) -> Result<&Value, VmError> {
    match locals.get(slot as usize) {
        Some(Slot::Runtime(value)) => Ok(value),
        Some(Slot::Empty) => Err(VmError::Runtime(format!("local slot {slot} is empty"))),
        Some(Slot::Iterator(_)) => Err(VmError::Runtime(format!(
            "local slot {slot} holds an iterator"
        ))),
        None => Err(VmError::Runtime(format!("local slot out of range: {slot}"))),
    }
}

fn iterator_slot_mut(locals: &mut [Slot], slot: u16) -> Result<&mut VmIterator, VmError> {
    match locals.get_mut(slot as usize) {
        Some(Slot::Iterator(iterator)) => Ok(iterator),
        Some(_) => Err(VmError::Runtime(format!(
            "local slot {slot} does not hold an iterator"
        ))),
        None => Err(VmError::Runtime(format!("local slot out of range: {slot}"))),
    }
}

fn pop_runtime(stack: &mut Vec<Slot>) -> Result<Value, VmError> {
    match stack.pop() {
        Some(Slot::Runtime(value)) => Ok(value),
        Some(Slot::Empty) | None => Err(VmError::Runtime("stack underflow".to_string())),
        Some(Slot::Iterator(_)) => Err(VmError::Runtime(
            "iterator leaked onto runtime stack".to_string(),
        )),
    }
}

fn pop_args(stack: &mut Vec<Slot>, argc: u16) -> Result<Vec<Value>, VmError> {
    let mut args = Vec::with_capacity(argc as usize);
    for _ in 0..argc {
        args.push(pop_runtime(stack)?);
    }
    args.reverse();
    Ok(args)
}

fn apply_binary(op: BinaryOpcode, lhs: Value, rhs: Value) -> Result<Value, VmError> {
    use BinaryOpcode::*;
    use Value::*;
    match op {
        Add => match (&lhs, &rhs) {
            (Int(a), Int(b)) => Ok(Int(a + b)),
            (Float(a), Float(b)) => Ok(Float(a + b)),
            (Int(a), Float(b)) => Ok(Float(*a as f64 + b)),
            (Float(a), Int(b)) => Ok(Float(a + *b as f64)),
            (Str(a), Str(b)) => Ok(Value::str(format!("{a}{b}"))),
            _ => Err(VmError::Runtime(
                "Add expects numeric or string pair".to_string(),
            )),
        },
        Sub => match (&lhs, &rhs) {
            (Int(a), Int(b)) => Ok(Int(a - b)),
            (Float(a), Float(b)) => Ok(Float(a - b)),
            (Int(a), Float(b)) => Ok(Float(*a as f64 - b)),
            (Float(a), Int(b)) => Ok(Float(a - *b as f64)),
            _ => Err(VmError::Runtime("Sub expects numeric pair".to_string())),
        },
        Mul => match (&lhs, &rhs) {
            (Int(a), Int(b)) => Ok(Int(a * b)),
            (Float(a), Float(b)) => Ok(Float(a * b)),
            (Int(a), Float(b)) => Ok(Float(*a as f64 * b)),
            (Float(a), Int(b)) => Ok(Float(a * *b as f64)),
            _ => Err(VmError::Runtime("Mul expects numeric pair".to_string())),
        },
        Div => {
            let div_by_zero = matches!(&rhs, Int(0)) || matches!(&rhs, Float(f) if *f == 0.0);
            if div_by_zero {
                return Err(VmError::Runtime("division by zero".to_string()));
            }
            match (&lhs, &rhs) {
                // checked_div: zero pre-guarded above; None is the
                // i64::MIN / -1 overflow. Same message as the interpreter so
                // the two backends stay diagnostically identical (RB-2).
                (Int(a), Int(b)) => a
                    .checked_div(*b)
                    .map(Int)
                    .ok_or_else(|| VmError::Runtime(format!("integer overflow: {a} / {b}"))),
                (Float(a), Float(b)) => Ok(Float(a / b)),
                (Int(a), Float(b)) => Ok(Float(*a as f64 / b)),
                (Float(a), Int(b)) => Ok(Float(a / *b as f64)),
                _ => Err(VmError::Runtime("Div expects numeric pair".to_string())),
            }
        }
        Mod => match (&lhs, &rhs) {
            // None = i64::MIN % -1 overflow — same message as the interpreter (RB-2).
            (Int(a), Int(b)) if *b != 0 => a
                .checked_rem(*b)
                .map(Int)
                .ok_or_else(|| VmError::Runtime(format!("integer overflow: {a} % {b}"))),
            (Int(_), Int(0)) => Err(VmError::Runtime("division by zero".to_string())),
            (Float(a), Float(b)) => Ok(Float(a % b)),
            _ => Err(VmError::Runtime("Mod expects numeric pair".to_string())),
        },
        Eq => Ok(Bool(lhs.eq_deep(&rhs))),
        NotEq => Ok(Bool(!lhs.eq_deep(&rhs))),
        Lt | Gt | LtEq | GtEq => {
            let cmp = lhs.partial_compare(&rhs).ok_or_else(|| {
                VmError::Runtime("comparison expects comparable pair".to_string())
            })?;
            Ok(Bool(match op {
                Lt => cmp.is_lt(),
                Gt => cmp.is_gt(),
                LtEq => cmp.is_le(),
                GtEq => cmp.is_ge(),
                _ => unreachable!(),
            }))
        }
    }
}

fn apply_unary(op: UnaryOpcode, value: Value) -> Result<Value, VmError> {
    match op {
        UnaryOpcode::Neg => match value {
            Value::Int(value) => Ok(Value::Int(-value)),
            Value::Float(value) => Ok(Value::Float(-value)),
            _ => Err(VmError::Runtime("Neg expects numeric value".to_string())),
        },
        UnaryOpcode::Not => Ok(Value::Bool(!value.truthy())),
    }
}

fn call_method(receiver: &Value, method: &str, args: Vec<Value>) -> Result<Value, VmError> {
    match method {
        "len" | "length" | "size" | "count" if args.is_empty() => len_value(receiver),
        _ => Err(VmError::Runtime(format!(
            "method `{method}` falls back to tree-walk outside S2 native subset"
        ))),
    }
}

fn len_value(value: &Value) -> Result<Value, VmError> {
    match value {
        Value::Str(value) => Ok(Value::Int(value.chars().count() as i64)),
        Value::Array(value) => Ok(Value::Int(value.borrow().len() as i64)),
        Value::Map(value) => Ok(Value::Int(value.borrow().len() as i64)),
        _ => Err(VmError::Runtime("len expects String/Array/Map".to_string())),
    }
}

fn materialize_iter(value: &Value) -> Result<Vec<Value>, VmError> {
    match value {
        Value::Array(values) => Ok(values.borrow().clone()),
        Value::Range {
            start,
            end,
            inclusive,
        } => {
            let stop = if *inclusive { *end + 1 } else { *end };
            Ok((*start..stop).map(Value::Int).collect())
        }
        Value::Str(value) => Ok(value
            .chars()
            .map(|ch| Value::str(ch.to_string()))
            .collect::<Vec<_>>()),
        Value::Map(value) => Ok(value
            .borrow()
            .iter()
            .map(|(key, value)| Value::tuple(vec![Value::str(key.clone()), value.clone()]))
            .collect::<Vec<_>>()),
        _ => Err(VmError::Runtime("value is not iterable".to_string())),
    }
}
