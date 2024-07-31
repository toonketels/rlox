mod codes;
mod constants;
mod disassemble;
mod lines;

use crate::opcode::OpCode::Constant;
use crate::opcode::{Byte, OpCode, Value};
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

impl Strings {
    pub fn new() -> Self {
        Strings(Vec::new())
    }

    pub fn add(&mut self, string: String) -> usize {
        self.0.push(string);
        self.0.len() - 1
    }

    pub fn get(&self, index: usize) -> Option<&String> {
        let it = self.0.get(index);
        it
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

    pub fn write_constant(&mut self, value: Value, line: usize) {
        let index = self.add_constant(value);

        let at = Byte::try_from(index).expect("Constant added at index out of range for byte");

        self.write_code(Constant, line);
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
