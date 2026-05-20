use crate::bytecode::{
    BinaryOpcode, BytecodeFunction, BytecodeProgram, Constant, Instruction, UnaryOpcode,
};
use crate::compiler::{compile_source, VmArtifact};
use crate::VmError;
use garnet_interp::{Interpreter, Value};

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
        let Some(function) = self.program.function(name) else {
            return self.call_fallback(name, args, "function not present in VM program");
        };
        if !function.native {
            return self.call_fallback(
                name,
                args,
                function
                    .fallback_reason
                    .as_deref()
                    .unwrap_or("function marked fallback"),
            );
        }
        self.summary.native_function_calls += 1;
        self.execute(function, args)
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

    fn execute(&mut self, function: &BytecodeFunction, args: Vec<Value>) -> Result<Value, VmError> {
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
        let mut stack: Vec<Slot> = Vec::new();
        let mut ip = 0usize;
        while ip < function.instructions.len() {
            let instruction = &function.instructions[ip];
            self.summary.native_instruction_count += 1;
            match instruction {
                Instruction::Const(index) => {
                    stack.push(Slot::Runtime(self.constant(*index)?));
                    ip += 1;
                }
                Instruction::LoadGlobal(index) => {
                    let name = self.constant_string(*index)?;
                    let value = self
                        .fallback
                        .global
                        .get(name)
                        .ok_or_else(|| VmError::Runtime(format!("undefined global: {name}")))?;
                    stack.push(Slot::Runtime(value));
                    ip += 1;
                }
                Instruction::LoadLocal(slot) => {
                    stack.push(Slot::Runtime(runtime_slot(&locals, *slot)?.clone()));
                    ip += 1;
                }
                Instruction::StoreLocal(slot) => {
                    let value = stack
                        .pop()
                        .ok_or_else(|| VmError::Runtime("stack underflow".to_string()))?;
                    store_slot(&mut locals, *slot, value)?;
                    ip += 1;
                }
                Instruction::Pop => {
                    stack.pop();
                    ip += 1;
                }
                Instruction::Binary(op) => {
                    let rhs = pop_runtime(&mut stack)?;
                    let lhs = pop_runtime(&mut stack)?;
                    stack.push(Slot::Runtime(apply_binary(*op, lhs, rhs)?));
                    ip += 1;
                }
                Instruction::Unary(op) => {
                    let value = pop_runtime(&mut stack)?;
                    stack.push(Slot::Runtime(apply_unary(*op, value)?));
                    ip += 1;
                }
                Instruction::Jump(target) => {
                    ip = *target;
                }
                Instruction::JumpIfFalse(target) => {
                    let value = pop_runtime(&mut stack)?;
                    if !value.truthy() {
                        ip = *target;
                    } else {
                        ip += 1;
                    }
                }
                Instruction::MakeArray(count) => {
                    let mut values = Vec::with_capacity(*count as usize);
                    for _ in 0..*count {
                        values.push(pop_runtime(&mut stack)?);
                    }
                    values.reverse();
                    stack.push(Slot::Runtime(Value::array(values)));
                    ip += 1;
                }
                Instruction::IterInit => {
                    let value = pop_runtime(&mut stack)?;
                    stack.push(Slot::Iterator(VmIterator {
                        items: materialize_iter(&value)?,
                        index: 0,
                    }));
                    ip += 1;
                }
                Instruction::IterNext {
                    iterator_slot,
                    item_slot,
                    jump_to,
                } => {
                    let iterator = iterator_slot_mut(&mut locals, *iterator_slot)?;
                    if iterator.index >= iterator.items.len() {
                        ip = *jump_to;
                    } else {
                        let item = iterator.items[iterator.index].clone();
                        iterator.index += 1;
                        store_slot(&mut locals, *item_slot, Slot::Runtime(item))?;
                        ip += 1;
                    }
                }
                Instruction::Call { name, argc } => {
                    let name = self.constant_string(*name)?.to_string();
                    let args = pop_args(&mut stack, *argc)?;
                    let value = self.call_named(&name, args)?;
                    stack.push(Slot::Runtime(value));
                    ip += 1;
                }
                Instruction::CallMethod { name, argc } => {
                    let name = self.constant_string(*name)?.to_string();
                    let args = pop_args(&mut stack, *argc)?;
                    let receiver = pop_runtime(&mut stack)?;
                    let value = call_method(&receiver, &name, args)?;
                    stack.push(Slot::Runtime(value));
                    ip += 1;
                }
                Instruction::Return => {
                    return Ok(match stack.pop() {
                        Some(Slot::Runtime(value)) => value,
                        Some(Slot::Empty) | Some(Slot::Iterator(_)) | None => Value::Nil,
                    });
                }
            }
        }
        Ok(Value::Nil)
    }

    fn call_named(&mut self, name: &str, args: Vec<Value>) -> Result<Value, VmError> {
        match name {
            "println" => {
                if self.options.emit_stdout {
                    let rendered = args.iter().map(Value::display).collect::<Vec<_>>();
                    println!("{}", rendered.join(" "));
                }
                Ok(Value::Nil)
            }
            "print" => {
                if self.options.emit_stdout {
                    let rendered = args.iter().map(Value::display).collect::<Vec<_>>();
                    print!("{}", rendered.join(" "));
                }
                Ok(Value::Nil)
            }
            "len" => {
                let value = args
                    .first()
                    .ok_or_else(|| VmError::Runtime("len: missing arg".to_string()))?;
                len_value(value)
            }
            other => self.call_function(other, args),
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
                (Int(a), Int(b)) => Ok(Int(a / b)),
                (Float(a), Float(b)) => Ok(Float(a / b)),
                (Int(a), Float(b)) => Ok(Float(*a as f64 / b)),
                (Float(a), Int(b)) => Ok(Float(a / *b as f64)),
                _ => Err(VmError::Runtime("Div expects numeric pair".to_string())),
            }
        }
        Mod => match (&lhs, &rhs) {
            (Int(a), Int(b)) if *b != 0 => Ok(Int(a % b)),
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
