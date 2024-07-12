use crate::chunk::Chunk;
use crate::codes::Byte;
use crate::constants::Value;
use crate::opcode::OpCode;
use crate::stack::Stack;
use crate::vm::InterpretError::RuntimeError;
use std::fmt::{Display, Formatter};

/// Virtual machine that executes our program

pub struct Vm<'a> {
    chunk: &'a Chunk,
    stack: Stack,
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
        Vm {
            chunk,
            stack: Stack::new(),
            ip: 0,
        }
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

    fn read_constant(&mut self) -> Result<Value, InterpretError> {
        self.chunk.read_constant(self.advance()).ok_or(RuntimeError)
    }

    fn push_stack(&mut self, value: Value) {
        self.stack.push(value)
    }

    fn pop_stack(&mut self) -> Result<Value, InterpretError> {
        self.stack.pop().ok_or(RuntimeError)
    }

    pub fn run(&mut self) -> Result<(), InterpretError> {
        macro_rules! binary_op {
            ($op:tt) => {
                {
                    let x = self.pop_stack()?;
                    let y = self.pop_stack()?;
                    self.push_stack(x $op y)
                }
            };
        }

        use OpCode::*;
        loop {
            match self.read_decode()? {
                // We are done
                Return => {
                    println!("Return: {:?}", self.pop_stack()?);
                    break Ok(());
                }

                // Arithmetic
                Add => binary_op!(+),
                Subtract => binary_op!(-),
                Multiply => binary_op!(*),
                Divide => binary_op!(/),
                Negate => {
                    let x = self.pop_stack()?;
                    self.push_stack(-x)
                }

                Constant => {
                    let x = self.read_constant()?;
                    self.push_stack(x)
                }
            }
        }
    }

    fn read_decode(&mut self) -> Result<OpCode, InterpretError> {
        // No more codes to fetch... runtime error
        let byte = self.read_byte().ok_or(RuntimeError)?;
        // Byte is not an opcode... runtime error
        let code = OpCode::try_from(byte).map_err(|_| RuntimeError)?;

        // This is ugly, because read_byte advances the ip, we need to put it back
        // for the disassemble instruction
        self.chunk.disassemble_instruction(byte, self.ip - 1);

        Ok(code)
    }
}
