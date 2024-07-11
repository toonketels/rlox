use crate::chunk::Chunk;
use crate::codes::Byte;
use crate::constants::Value;
use crate::opcode::OpCode;
use crate::vm::InterpretError::RuntimeError;
use std::fmt::{Display, Formatter};

/// Virtual machine that executes our program

pub struct Vm<'a> {
    chunk: &'a Chunk,
    ip: usize,
}

#[derive(Debug)]
pub enum InterpretError {
    CompileError,
    RuntimeError,
}

impl Display for InterpretError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            InterpretError::CompileError => write!(f, "compilation error"),
            InterpretError::RuntimeError => write!(f, "compilation error"),
        }
    }
}

pub fn interpret(chunk: &Chunk) -> Result<(), InterpretError> {
    let mut vm = Vm::new(chunk);
    vm.run()
}

impl<'a> Vm<'a> {
    pub fn new(chunk: &'a Chunk) -> Self {
        Vm { chunk, ip: 0 }
    }

    /// Returns the next to fetch instruction location and advances the ip
    fn advance(&mut self) -> usize {
        let ip = self.ip;
        self.ip = ip + 1;
        ip
    }

    fn read_byte(&mut self) -> Option<Byte> {
        self.chunk.read_byte(self.advance())
    }

    fn read_constant(&mut self) -> Option<Value> {
        self.chunk.read_constant(self.advance())
    }

    pub fn run(&mut self) -> Result<(), InterpretError> {
        loop {
            // No more codes to fetch... runtime error
            let byte = self.read_byte().ok_or(RuntimeError)?;
            // Byte is not an opcode... runtime error
            let code = OpCode::try_from(byte).map_err(|_| RuntimeError)?;

            // This is ugly, because read_byte advances the ip, we need to put it back
            // for the disassemble instruction
            self.chunk.disassemble_instruction(byte, self.ip - 1);

            match code {
                // We are done
                OpCode::Return => break Ok(()),

                OpCode::Constant => {
                    let constant = self.read_constant().ok_or(RuntimeError)?;
                    println!("constant: {:?}", constant);
                }
            }
        }
    }
}
