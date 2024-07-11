use crate::chunk::OpCode::{Constant, Return};
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

type Byte = u8;
type Value = f64;

#[derive(Debug)]
pub struct Chunk {
    code: Vec<Byte>,
    constants: Vec<Value>,
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
            code: Vec::new(),
            constants: Vec::new(),
            lines: Lines::new(),
        }
    }

    fn write_byte(&mut self, byte: Byte, line: usize) {
        self.code.push(byte);
        // Keeps track which src line this belongs to
        self.lines.insert(self.code.len() - 1, line);
    }

    fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
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

    // Returns the next instruction
    fn disassemble_instruction(&self, byte: &Byte, at: usize) -> usize {
        let line = self.lines.at(at);

        match OpCode::try_from(*byte) {
            Ok(OpCode::Constant) => {
                let i = self
                    .code
                    .get(at + 1)
                    .unwrap_or_else(|| panic!("Constant at index {:?} should exist", at + 1));

                let index = *i as usize;

                let c = self
                    .constants
                    .get(index)
                    .unwrap_or_else(|| panic!("Constant at index {:?} should exist", index));

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
