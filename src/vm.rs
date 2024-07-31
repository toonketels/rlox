use crate::chunk::Chunk;
use crate::heap::pointer::PointerHeap as Heap;
use crate::opcode::Value::{Bool, Number};
use crate::opcode::{Byte, Obj, OpCode, Value};
use crate::tokenizer::TokenKind;
use crate::vm::InterpretError::{RuntimeError, RuntimeErrorWithReason};
use stack::Stack;
use std::fmt::{Display, Formatter};

mod stack;

/// Virtual machine that executes our program

pub struct Vm<'a> {
    chunk: &'a Chunk,
    stack: Stack,
    heap: Heap,
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
            heap: Heap::new(),
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

    fn read_string(&mut self) -> Result<Value, InterpretError> {
        let it = self.chunk.read_string(self.advance());
        let str = it.ok_or(RuntimeError)?;
        let obj = self.heap.alloc(Obj::String {
            str: str.to_string(),
        });
        Ok(Value::Object(obj))
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
        macro_rules! binary_op_number {
            ($op:tt) => {
                {

                    let is_number = self.peek_stack(0).is_some_and(|it| it.is_number()) &&  self.peek_stack(1).is_some_and(|it| it.is_number());
                    if !is_number {
                        Err(RuntimeErrorWithReason("Operands must be numbers"))?;
                    }
                    let rhs = self.pop_stack()?.as_number();
                    let lhs = self.pop_stack()?.as_number();
                    self.push_stack(Number(lhs $op rhs))
                }
            };
        }

        macro_rules! binary_op_bool {
            ($op:tt) => {
                {

                    let is_number = self.peek_stack(0).is_some_and(|it| it.is_number()) &&  self.peek_stack(1).is_some_and(|it| it.is_number());
                    if !is_number {
                        Err(RuntimeErrorWithReason("Operands must be numbers"))?;
                    }
                    let rhs = self.pop_stack()?.as_number();
                    let lhs = self.pop_stack()?.as_number();
                    self.push_stack(Bool(lhs $op rhs))
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
                    let it = self.pop_stack()?.is_truthy();
                    self.push_stack(Bool(!it));
                }

                // Literals
                False => self.push_stack(Bool(false)),
                True => self.push_stack(Bool(true)),
                Nil => self.push_stack(Value::Nil),
                String => {
                    let x = self.read_string()?;
                    // @todo turn into string Value
                    self.push_stack(x)
                }

                // Comparison
                Equal => {
                    let rhs = self.pop_stack()?;
                    let lhs = self.pop_stack()?;
                    self.push_stack(Value::Bool(lhs == rhs));
                } // @TODO more then just numbers can be compared
                Greater => binary_op_bool!(>),
                Less => binary_op_bool!(<),

                // Arithmetic
                Add => binary_op_number!(+),
                Subtract => binary_op_number!(-),
                Multiply => binary_op_number!(*),
                Divide => binary_op_number!(/),
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
    fn interpret_math_expression_with_precedence() {
        let chunk = Parser::parse(Tokenizer::new("10 + 30 * 2")).unwrap();
        let result = interpret(&chunk).unwrap();

        assert_eq!(result, Value::Number(70.0));
    }

    #[test]
    fn interpret_booleans() {
        let cases = vec![("true", true), ("false", false)];

        interpret_result_eq_bool(cases)
    }

    #[test]
    fn interpret_nil() {
        let chunk = Parser::parse(Tokenizer::new("nil")).unwrap();
        let result = interpret(&chunk).unwrap();

        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn interpret_not() {
        let cases = vec![
            ("!false", true),
            ("!true", false),
            ("!!true", true),
            ("!!false", false),
            ("!(5 == 5)", false),
            ("!nil", true),
            ("!0", true),
            ("!1", false),
            ("!-1", false),
        ];

        interpret_result_eq_bool(cases)
    }

    #[test]
    fn interpret_equal() {
        let cases = vec![
            ("100 == 100", true),
            ("100 == 10", false),
            ("true == true", true),
            ("true == false", false),
            ("nil == nil", true),
            ("true == 10", false),
            ("100 == nil", false),
            ("false == nil", false),
            ("true == 1", false),
        ];

        interpret_result_eq_bool(cases)
    }

    #[test]
    fn interpret_not_equal() {
        let cases = vec![
            ("100 != 100", false),
            ("100 != 10", true),
            ("true != true", false),
            ("true != false", true),
            ("nil != nil", false),
            ("true != 10", true),
            ("100 != nil", true),
            ("false != nil", true),
            ("true != 1", true),
        ];

        interpret_result_eq_bool(cases);
    }

    #[test]
    fn interpret_greater() {
        let cases = vec![
            ("100 > 100", false),
            ("100 > 10", true),
            ("10 > 100", false),
        ];

        interpret_result_eq_bool(cases)
    }

    #[test]
    fn interpret_greater_equal() {
        let cases = vec![
            ("100 >= 100", true),
            ("100 >= 10", true),
            ("10 >= 100", false),
        ];

        interpret_result_eq_bool(cases)
    }

    #[test]
    fn interpret_less() {
        let cases = vec![
            ("100 < 100", false),
            ("100 < 10", false),
            ("10 < 100", true),
        ];

        interpret_result_eq_bool(cases)
    }
    #[test]
    fn interpret_less_equal() {
        let cases = vec![
            ("100 <= 100", true),
            ("100 <= 10", false),
            ("10 <= 100", true),
        ];

        interpret_result_eq_bool(cases)
    }
    #[test]
    fn interpret_expression() {
        let cases = vec![("!(5 - 4 > 3 * 2 == !nil)", true)];

        interpret_result_eq_bool(cases)
    }

    #[test]
    fn interpret_strings() {
        let chunk = Parser::parse(Tokenizer::new("\"hello world\"")).unwrap();
        let result = interpret(&chunk).unwrap();

        if let Value::Object(it) = result {
            assert_eq!(
                *it.as_ref(),
                Obj::String {
                    str: "hello world".to_string()
                }
            );
        } else {
            panic!("not an Object")
        }
    }

    fn interpret_result_eq_bool(cases: Vec<(&str, bool)>) {
        for (source, expected) in cases {
            let chunk = Parser::parse(Tokenizer::new(source)).unwrap();
            let result = interpret(&chunk).unwrap();

            assert_eq!(result, Value::Bool(expected));
        }
    }

    fn interpret_result_eq_number(cases: Vec<(&str, f64)>) {
        for (source, expected) in cases {
            let chunk = Parser::parse(Tokenizer::new(source)).unwrap();
            let result = interpret(&chunk).unwrap();

            assert_eq!(result, Value::Number(expected));
        }
    }
}
