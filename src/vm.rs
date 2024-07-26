use crate::chunk::Chunk;
use crate::opcode::Value::{Bool, Number};
use crate::opcode::{Byte, OpCode, Value};
use crate::tokenizer::TokenKind;
use crate::vm::InterpretError::{RuntimeError, RuntimeErrorWithReason};
use stack::Stack;
use std::fmt::{Display, Formatter};

mod stack;

/// Virtual machine that executes our program

pub struct Vm<'a> {
    chunk: &'a Chunk,
    stack: Stack,
    ip: usize,
}

#[derive(Debug)]
pub enum CompilationErrorReason {
    NotEnoughTokens,
    TooMayTokens,
    ParseFloatError,
    ExpectedRightParen,
    ExpectedPrefix,
    ExpectedBinaryOperator,
    ExpectedDifferentToken {
        expected: TokenKind,
        received: TokenKind,
    },
}

#[derive(Debug)]
pub enum InterpretError {
    LoadError,
    CompileError(CompilationErrorReason),
    RuntimeError,
    RuntimeErrorWithReason(&'static str),
    Io(std::io::Error),
}

impl From<std::io::Error> for InterpretError {
    fn from(value: std::io::Error) -> Self {
        InterpretError::Io(value)
    }
}

impl Display for InterpretError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            InterpretError::CompileError(_) => write!(f, "compilation error"),
            InterpretError::RuntimeError => write!(f, "runtime error"),
            InterpretError::RuntimeErrorWithReason(reason) => {
                write!(f, "runtime error: {}", reason)
            }
            InterpretError::LoadError => write!(f, "load error"),
            InterpretError::Io(io) => write!(f, "Io error {}", io),
        }
    }
}

pub fn interpret(chunk: &Chunk) -> Result<Value, InterpretError> {
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

    fn peek_stack(&mut self, offset: usize) -> Option<&Value> {
        self.stack.peek(offset)
    }

    pub fn run(&mut self) -> Result<Value, InterpretError> {
        macro_rules! binary_op {
            ($op:tt) => {
                {

                    let is_number = self.peek_stack(0).is_some_and(|it| it.is_number()) &&  self.peek_stack(1).is_some_and(|it| it.is_number());
                    if !is_number {
                        Err(RuntimeErrorWithReason("Operands must be numbers"))?;
                    }
                    let x = self.pop_stack()?.as_number();
                    let y = self.pop_stack()?.as_number();
                    let z = x $op y;
                    self.push_stack(Number(z))
                }
            };
        }

        use OpCode::*;
        loop {
            match self.read_decode()? {
                // We are done
                Return => {
                    let it = self.pop_stack()?;
                    println!("Return: {:?}", it);
                    break Ok(it);
                }

                // unary
                Not => {
                    let is_bool = self.peek_stack(0).is_some_and(|it| it.is_bool());
                    if !is_bool {
                        Err(RuntimeErrorWithReason("Not works on booleans only"))?;
                    }
                    let it = self.pop_stack()?.as_bool();
                    self.push_stack(Bool(!it));
                }

                // Literals
                False => self.push_stack(Bool(false)),
                True => self.push_stack(Bool(true)),
                Nil => self.push_stack(Value::Nil),

                // Arithmetic
                Add => binary_op!(+),
                Subtract => binary_op!(-),
                Multiply => binary_op!(*),
                Divide => binary_op!(/),
                Negate => {
                    let is_number = self.peek_stack(0).is_some_and(|it| it.is_number());
                    if !is_number {
                        Err(RuntimeErrorWithReason("Negation works on numbers only"))?;
                    }
                    let x = self.pop_stack()?;
                    self.push_stack(Number(-x.as_number()))
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

#[cfg(test)]
mod tests {

    use super::*;
    use crate::parser::Parser;
    use crate::tokenizer::Tokenizer;

    #[test]
    fn interpret_1() {
        let chunk = Parser::parse(Tokenizer::new("10 + 30 * 2")).unwrap();
        let result = interpret(&chunk).unwrap();

        assert_eq!(result, Value::Number(70.0));
    }

    #[test]
    fn interpret_2() {
        let chunk = Parser::parse(Tokenizer::new("!true")).unwrap();
        let result = interpret(&chunk).unwrap();

        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn interpret_3() {
        let chunk = Parser::parse(Tokenizer::new("nil")).unwrap();
        let result = interpret(&chunk).unwrap();

        assert_eq!(result, Value::Nil);
    }

    #[test]
    #[ignore]
    fn interpret_4() {
        // We havent implemented this but the book does
        let chunk = Parser::parse(Tokenizer::new("!nil")).unwrap();
        let result = interpret(&chunk).unwrap();

        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn interpret_5() {
        // We havent implemented this but the book does
        let chunk = Parser::parse(Tokenizer::new("!!false")).unwrap();
        let result = interpret(&chunk).unwrap();

        assert_eq!(result, Value::Bool(false));
    }
}
