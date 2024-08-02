mod codes;
mod constants;
mod disassemble;
mod lines;

use crate::opcode::OpCode::Constant;
use crate::opcode::{Byte, OpCode, Value};
use crate::vm::InterpretError;
use codes::Codes;
use constants::Constants;
use lines::Lines;

// static strings part of the binary
#[derive(Debug)]
pub struct Strings(Vec<String>);

impl Default for Strings {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Default)]
pub struct Jump {
    pub how_far: u16,
}

// How for in the code block jump
impl Jump {
    pub fn calculate(from: usize, to: usize) -> Result<Self, InterpretError> {
        // to is 13, from is 2
        let how_far = to - from;

        match how_far > u16::MAX as usize {
            true => Err(InterpretError::JumpTooFar),
            false => Ok(Jump {
                how_far: how_far as u16,
            }),
        }
    }

    pub fn to_bytes(&self) -> (Byte, Byte) {
        let lower = self.how_far as u8;
        let higher = (self.how_far >> 8) as u8;
        (higher, lower)
    }

    pub fn from_bytes(higher: Byte, lower: Byte) -> Self {
        let how_far = (higher as u16) << 8 | (lower as u16);

        Self { how_far }
    }
}

impl Strings {
    pub fn new() -> Self {
        Strings(Vec::new())
    }

    pub fn add(&mut self, string: String) -> usize {
        self.0.push(string);
        self.0.len() - 1
    }

    pub fn get(&self, index: usize) -> Option<&String> {
        self.0.get(index)
    }
}

#[derive(Debug)]
pub struct Chunk {
    pub(crate) code: Codes,
    pub(crate) constants: Constants,
    pub(crate) strings: Strings,
    // Tracks the src line the corresponding opcode refers to for error reporting
    pub(crate) lines: Lines,
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            code: Codes::new(),
            constants: Constants::new(),
            strings: Strings::new(),
            lines: Lines::new(),
        }
    }

    fn write_byte(&mut self, byte: Byte, line: usize) {
        let at = self.code.add(byte);
        // Keeps track which src line this belongs to
        self.lines.insert(at, line);
    }

    fn add_constant(&mut self, value: Value) -> usize {
        self.constants.add(value)
    }

    pub fn write_code(&mut self, op_code: OpCode, line: usize) {
        self.write_byte(op_code as Byte, line)
    }

    // Returns the address to patch
    pub fn write_jump(&mut self, op_code: OpCode, line: usize) -> Result<usize, InterpretError> {
        let (higher, lower) = Jump::default().to_bytes();
        self.write_byte(op_code as Byte, line);

        self.write_byte(higher, line);
        self.write_byte(lower, line);

        Ok(self.code.len() - 2)
    }

    pub fn patch_jump(&mut self, at: usize) -> Result<(), InterpretError> {
        let (higher, lower) = Jump::calculate(at, self.code.len())?.to_bytes();
        self.code.patch(at, higher);
        self.code.patch(at + 1, lower);
        Ok(())
    }

    pub fn write_constant(&mut self, value: Value, line: usize) {
        let index = self.add_constant(value);

        let at = Byte::try_from(index).expect("Constant added at index out of range for byte");

        self.write_code(Constant, line);
        self.write_byte(at as Byte, line);
    }

    pub fn write_define_global_var(&mut self, str: String, line: usize) {
        let index = self.strings.add(str);

        let at = Byte::try_from(index)
            .expect("Global variable name added at index out of range for byte");

        self.write_code(OpCode::DefineGlobal, line);
        self.write_byte(at as Byte, line);
    }

    pub fn write_set_global_var(&mut self, str: String, line: usize) {
        let index = self.strings.add(str);

        let at = Byte::try_from(index)
            .expect("Global variable name added at index out of range for byte");

        self.write_code(OpCode::SetGlobal, line);
        self.write_byte(at as Byte, line);
    }

    pub fn write_get_global_var(&mut self, str: String, line: usize) {
        let index = self.strings.add(str);

        let at = Byte::try_from(index)
            .expect("Global variable name added at index out of range for byte");

        self.write_code(OpCode::GetGlobal, line);
        self.write_byte(at as Byte, line);
    }

    pub fn write_set_local_var(&mut self, locals_index: usize, line: usize) {
        let at = Byte::try_from(locals_index)
            .expect("Local variable name added at index out of range for byte");

        self.write_code(OpCode::SetLocal, line);
        self.write_byte(at as Byte, line);
    }

    pub fn write_get_local_var(&mut self, locals_index: usize, line: usize) {
        let at = Byte::try_from(locals_index)
            .expect("Local variable name added at index out of range for byte");

        self.write_code(OpCode::GetLocal, line);
        self.write_byte(at as Byte, line);
    }

    pub fn write_string(&mut self, str: String, line: usize) {
        let index = self.strings.add(str);

        let at = Byte::try_from(index).expect("String added at index out of range for byte");

        self.write_code(OpCode::String, line);
        self.write_byte(at as Byte, line);
    }

    pub fn read_byte(&self, index: usize) -> Option<Byte> {
        self.code.get(index)
    }

    pub fn read_jump(&self, index: usize) -> Option<Jump> {
        let higher = self.read_byte(index)?;
        let lower = self.read_byte(index + 1)?;
        Some(Jump::from_bytes(higher, lower))
    }

    pub fn read_constant(&self, index: usize) -> Option<Value> {
        let i = self.read_byte(index)?;
        let index = i as usize;

        self.constants.get(index)
    }

    pub fn read_string(&self, index: usize) -> Option<&str> {
        let i = self.read_byte(index)?;
        let index = i as usize;

        let it = self.strings.get(index);
        it.map(|it| it.as_str())
    }
}
