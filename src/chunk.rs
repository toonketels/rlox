use crate::chunk::OpCode::{Constant, Return};
use crate::codes::{Byte, Codes};
use crate::constants::{Constants, Value};
use crate::lines::Lines;

#[derive(Debug)]
#[repr(u8)]
pub enum OpCode {
    Constant,
    Return,
}

impl TryFrom<Byte> for OpCode {
    type Error = ();

    fn try_from(value: Byte) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Constant),
            1 => Ok(Return),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub struct Chunk {
    code: Codes,
    constants: Constants,
    // Tracks the src line the corresponding opcode refers to for error reporting
    lines: Lines,
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

    pub fn disassemble_chunk(&self, name: &str) {
        println!("== {} ==", name);

        let mut n = 0;
        loop {
            let Some(code) = self.code.get(n) else {
                break;
            };
            n = self.disassemble_instruction(code, n);
        }
    }

    // Returns the next instruction location
    fn disassemble_instruction(&self, byte: Byte, at: usize) -> usize {
        let line = self.lines.at(at);

        match OpCode::try_from(byte) {
            Ok(OpCode::Constant) => {
                let i = self
                    .code
                    .get(at + 1)
                    .unwrap_or_else(|| panic!("Constant at index {:?} should exist", at + 1));

                let index = i as usize;

                let c = self.constants.at(index);

                println!("{:8} {:8} | Constant {:?}", at, line, c);

                at + 2
            }
            Ok(OpCode::Return) => {
                println!("{:8} {:8} | Return", at, line);
                at + 1
            }

            Err(_) => {
                panic!("Not an opcode")
            }
        }
    }
}
