use crate::bytecode::{
    BinaryOpcode, BytecodeFunction, BytecodeProgram, Constant, Instruction, UnaryOpcode,
};
use crate::VmError;

// ABI v0.2 (S14): the magic is version-bumped from `GARNVM01` and each
// function now carries an explicit `arity` field ahead of its parameter
// vector so a reader can validate arity without trusting the vector length.
// This is a tightened, more self-describing schema — still NOT a stable
// cross-version external ABI promise.
const MAGIC: &[u8; 8] = b"GARNVM02";

pub fn serialize_program(program: &BytecodeProgram) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(MAGIC);
    write_u32(&mut out, program.constants.len() as u32);
    for constant in &program.constants {
        write_constant(&mut out, constant);
    }
    write_u32(&mut out, program.functions.len() as u32);
    for function in &program.functions {
        write_string(&mut out, &function.name);
        // ABI v0.2: explicit arity ahead of the params vector.
        write_u32(&mut out, function.params.len() as u32);
        write_u32(&mut out, function.params.len() as u32);
        for param in &function.params {
            write_string(&mut out, param);
        }
        write_u32(&mut out, function.locals.len() as u32);
        for local in &function.locals {
            write_string(&mut out, local);
        }
        out.push(u8::from(function.native));
        match &function.fallback_reason {
            Some(reason) => {
                out.push(1);
                write_string(&mut out, reason);
            }
            None => out.push(0),
        }
        write_u32(&mut out, function.instructions.len() as u32);
        for instruction in &function.instructions {
            write_instruction(&mut out, instruction);
        }
    }
    out
}

pub fn deserialize_program(bytes: &[u8]) -> Result<BytecodeProgram, VmError> {
    let mut reader = Reader { bytes, cursor: 0 };
    let magic = reader.read_exact(MAGIC.len())?;
    if magic != MAGIC {
        return Err(VmError::Codec("invalid bytecode magic".to_string()));
    }
    let constant_count = reader.read_u32()? as usize;
    let mut constants = Vec::with_capacity(constant_count);
    for _ in 0..constant_count {
        constants.push(reader.read_constant()?);
    }
    let function_count = reader.read_u32()? as usize;
    let mut functions = Vec::with_capacity(function_count);
    for _ in 0..function_count {
        let name = reader.read_string()?;
        // ABI v0.2: explicit arity, cross-checked against the params vector.
        let arity = reader.read_u32()? as usize;
        let params = reader.read_string_vec()?;
        if arity != params.len() {
            return Err(VmError::Codec(format!(
                "function `{name}`: declared arity {arity} != params length {}",
                params.len()
            )));
        }
        let locals = reader.read_string_vec()?;
        let native = reader.read_u8()? != 0;
        let fallback_reason = match reader.read_u8()? {
            0 => None,
            1 => Some(reader.read_string()?),
            other => {
                return Err(VmError::Codec(format!(
                    "invalid fallback-reason marker {other}"
                )))
            }
        };
        let instruction_count = reader.read_u32()? as usize;
        let mut instructions = Vec::with_capacity(instruction_count);
        for _ in 0..instruction_count {
            instructions.push(reader.read_instruction()?);
        }
        functions.push(BytecodeFunction {
            name,
            params,
            locals,
            instructions,
            native,
            fallback_reason,
        });
    }
    Ok(BytecodeProgram {
        constants,
        functions,
    })
}

fn write_constant(out: &mut Vec<u8>, constant: &Constant) {
    match constant {
        Constant::Nil => out.push(0),
        Constant::Bool(value) => {
            out.push(1);
            out.push(u8::from(*value));
        }
        Constant::Int(value) => {
            out.push(2);
            out.extend_from_slice(&value.to_le_bytes());
        }
        Constant::Float(value) => {
            out.push(3);
            out.extend_from_slice(&value.to_bits().to_le_bytes());
        }
        Constant::Str(value) => {
            out.push(4);
            write_string(out, value);
        }
        Constant::Symbol(value) => {
            out.push(5);
            write_string(out, value);
        }
    }
}

fn write_instruction(out: &mut Vec<u8>, instruction: &Instruction) {
    match instruction {
        Instruction::Const(index) => {
            out.push(0);
            write_u32(out, *index);
        }
        Instruction::LoadGlobal(index) => {
            out.push(1);
            write_u32(out, *index);
        }
        Instruction::LoadLocal(slot) => {
            out.push(2);
            write_u16(out, *slot);
        }
        Instruction::StoreLocal(slot) => {
            out.push(3);
            write_u16(out, *slot);
        }
        Instruction::Pop => out.push(4),
        Instruction::Binary(op) => {
            out.push(5);
            out.push(binary_tag(*op));
        }
        Instruction::Unary(op) => {
            out.push(6);
            out.push(unary_tag(*op));
        }
        Instruction::Jump(target) => {
            out.push(7);
            write_u32(out, *target as u32);
        }
        Instruction::JumpIfFalse(target) => {
            out.push(8);
            write_u32(out, *target as u32);
        }
        Instruction::MakeArray(count) => {
            out.push(9);
            write_u16(out, *count);
        }
        Instruction::IterInit => out.push(10),
        Instruction::IterNext {
            iterator_slot,
            item_slot,
            jump_to,
        } => {
            out.push(11);
            write_u16(out, *iterator_slot);
            write_u16(out, *item_slot);
            write_u32(out, *jump_to as u32);
        }
        Instruction::Call { name, argc } => {
            out.push(12);
            write_u32(out, *name);
            write_u16(out, *argc);
        }
        Instruction::CallMethod { name, argc } => {
            out.push(13);
            write_u32(out, *name);
            write_u16(out, *argc);
        }
        Instruction::Return => out.push(14),
    }
}

fn write_string(out: &mut Vec<u8>, value: &str) {
    write_u32(out, value.len() as u32);
    out.extend_from_slice(value.as_bytes());
}

fn write_u16(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn write_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn binary_tag(op: BinaryOpcode) -> u8 {
    match op {
        BinaryOpcode::Add => 0,
        BinaryOpcode::Sub => 1,
        BinaryOpcode::Mul => 2,
        BinaryOpcode::Div => 3,
        BinaryOpcode::Mod => 4,
        BinaryOpcode::Eq => 5,
        BinaryOpcode::NotEq => 6,
        BinaryOpcode::Lt => 7,
        BinaryOpcode::Gt => 8,
        BinaryOpcode::LtEq => 9,
        BinaryOpcode::GtEq => 10,
    }
}

fn unary_tag(op: UnaryOpcode) -> u8 {
    match op {
        UnaryOpcode::Neg => 0,
        UnaryOpcode::Not => 1,
    }
}

struct Reader<'a> {
    bytes: &'a [u8],
    cursor: usize,
}

impl<'a> Reader<'a> {
    fn read_exact(&mut self, len: usize) -> Result<&'a [u8], VmError> {
        let end = self.cursor.saturating_add(len);
        if end > self.bytes.len() {
            return Err(VmError::Codec("unexpected end of bytecode".to_string()));
        }
        let slice = &self.bytes[self.cursor..end];
        self.cursor = end;
        Ok(slice)
    }

    fn read_u8(&mut self) -> Result<u8, VmError> {
        Ok(self.read_exact(1)?[0])
    }

    fn read_u16(&mut self) -> Result<u16, VmError> {
        let bytes = self.read_exact(2)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    fn read_u32(&mut self) -> Result<u32, VmError> {
        let bytes = self.read_exact(4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn read_string(&mut self) -> Result<String, VmError> {
        let len = self.read_u32()? as usize;
        let bytes = self.read_exact(len)?;
        String::from_utf8(bytes.to_vec())
            .map_err(|error| VmError::Codec(format!("invalid utf-8 string: {error}")))
    }

    fn read_string_vec(&mut self) -> Result<Vec<String>, VmError> {
        let count = self.read_u32()? as usize;
        let mut values = Vec::with_capacity(count);
        for _ in 0..count {
            values.push(self.read_string()?);
        }
        Ok(values)
    }

    fn read_constant(&mut self) -> Result<Constant, VmError> {
        match self.read_u8()? {
            0 => Ok(Constant::Nil),
            1 => Ok(Constant::Bool(self.read_u8()? != 0)),
            2 => {
                let bytes = self.read_exact(8)?;
                Ok(Constant::Int(i64::from_le_bytes(
                    bytes.try_into().expect("8-byte i64"),
                )))
            }
            3 => {
                let bytes = self.read_exact(8)?;
                Ok(Constant::Float(f64::from_bits(u64::from_le_bytes(
                    bytes.try_into().expect("8-byte f64 bits"),
                ))))
            }
            4 => Ok(Constant::Str(self.read_string()?)),
            5 => Ok(Constant::Symbol(self.read_string()?)),
            other => Err(VmError::Codec(format!("unknown constant tag {other}"))),
        }
    }

    fn read_instruction(&mut self) -> Result<Instruction, VmError> {
        match self.read_u8()? {
            0 => Ok(Instruction::Const(self.read_u32()?)),
            1 => Ok(Instruction::LoadGlobal(self.read_u32()?)),
            2 => Ok(Instruction::LoadLocal(self.read_u16()?)),
            3 => Ok(Instruction::StoreLocal(self.read_u16()?)),
            4 => Ok(Instruction::Pop),
            5 => Ok(Instruction::Binary(read_binary(self.read_u8()?)?)),
            6 => Ok(Instruction::Unary(read_unary(self.read_u8()?)?)),
            7 => Ok(Instruction::Jump(self.read_u32()? as usize)),
            8 => Ok(Instruction::JumpIfFalse(self.read_u32()? as usize)),
            9 => Ok(Instruction::MakeArray(self.read_u16()?)),
            10 => Ok(Instruction::IterInit),
            11 => Ok(Instruction::IterNext {
                iterator_slot: self.read_u16()?,
                item_slot: self.read_u16()?,
                jump_to: self.read_u32()? as usize,
            }),
            12 => Ok(Instruction::Call {
                name: self.read_u32()?,
                argc: self.read_u16()?,
            }),
            13 => Ok(Instruction::CallMethod {
                name: self.read_u32()?,
                argc: self.read_u16()?,
            }),
            14 => Ok(Instruction::Return),
            other => Err(VmError::Codec(format!("unknown instruction tag {other}"))),
        }
    }
}

fn read_binary(tag: u8) -> Result<BinaryOpcode, VmError> {
    match tag {
        0 => Ok(BinaryOpcode::Add),
        1 => Ok(BinaryOpcode::Sub),
        2 => Ok(BinaryOpcode::Mul),
        3 => Ok(BinaryOpcode::Div),
        4 => Ok(BinaryOpcode::Mod),
        5 => Ok(BinaryOpcode::Eq),
        6 => Ok(BinaryOpcode::NotEq),
        7 => Ok(BinaryOpcode::Lt),
        8 => Ok(BinaryOpcode::Gt),
        9 => Ok(BinaryOpcode::LtEq),
        10 => Ok(BinaryOpcode::GtEq),
        other => Err(VmError::Codec(format!("unknown binary opcode tag {other}"))),
    }
}

fn read_unary(tag: u8) -> Result<UnaryOpcode, VmError> {
    match tag {
        0 => Ok(UnaryOpcode::Neg),
        1 => Ok(UnaryOpcode::Not),
        other => Err(VmError::Codec(format!("unknown unary opcode tag {other}"))),
    }
}
