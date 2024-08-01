use crate::chunk::Chunk;
use crate::heap::rc::RcHeap as Heap;
use crate::opcode::Value::{Bool, Number, Object};
use crate::opcode::{Byte, Obj, OpCode, Returned, Value};
use crate::tokenizer::TokenKind;
use crate::vm::InterpretError::{RuntimeError, RuntimeErrorWithReason, StackUnderflowError};
use stack::Stack;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

mod stack;

/// Virtual machine that executes our program

pub struct Vm<'a> {
    chunk: &'a Chunk,
    stack: Stack,
    heap: Heap,
    globals: HashMap<String, Value>,
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
    StackUnderflowError,
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
            InterpretError::StackUnderflowError => write!(f, "stack underflow error"),
            InterpretError::RuntimeErrorWithReason(reason) => {
                write!(f, "runtime error: {}", reason)
            }
            InterpretError::LoadError => write!(f, "load error"),
            InterpretError::Io(io) => write!(f, "Io error {}", io),
        }
    }
}

pub fn interpret(chunk: &Chunk) -> Result<Returned, InterpretError> {
    let mut vm = Vm::new(chunk);
    let result = vm.run();
    // Not strictly necessary to call free_all as it would be dropped by just going out of scope too
    vm.heap.free_all();

    println!("Globals: {:?}", vm.globals);

    result.map(Returned::from)
}

impl<'a> Vm<'a> {
    pub fn new(chunk: &'a Chunk) -> Self {
        Vm {
            chunk,
            stack: Stack::new(),
            heap: Heap::new(),
            globals: HashMap::new(),
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

    fn read_global_name(&mut self) -> Result<String, InterpretError> {
        let it = self.chunk.read_string(self.advance());
        let str = it.ok_or(RuntimeError)?;
        Ok(str.to_string())
    }

    fn push_stack(&mut self, value: Value) {
        self.stack.push(value)
    }

    fn pop_stack(&mut self) -> Result<Value, InterpretError> {
        self.stack.pop().ok_or(StackUnderflowError)
    }

    fn peek_stack(&mut self, offset: usize) -> Option<&Value> {
        self.stack.peek(offset)
    }

    fn peek_stack_expected(&mut self, offset: usize) -> Result<&Value, InterpretError> {
        self.peek_stack(offset).ok_or(StackUnderflowError)
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
                Add => {
                    let is_string = self.peek_stack(0).is_some_and(|it| it.is_string())
                        && self.peek_stack(1).is_some_and(|it| it.is_string());
                    if is_string {
                        self.string_concatenate()?;
                    } else {
                        binary_op_number!(+)
                    }
                }
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

                // bindings
                DefineGlobal => {
                    let name = self.read_global_name()?;
                    let value = self.pop_stack()?;
                    self.globals.insert(name, value);
                }

                GetGlobal => {
                    let name = self.read_global_name()?;
                    let value = self.globals.get(&name).unwrap_or(&Value::Nil);
                    self.push_stack(value.clone())
                }

                SetGlobal => {
                    let name = self.read_global_name()?;
                    let value = self.peek_stack_expected(0)?.clone(); // we dont pop from the stack
                    if let std::collections::hash_map::Entry::Occupied(mut e) =
                        self.globals.entry(name)
                    {
                        e.insert(value);
                    } else {
                        Err(RuntimeErrorWithReason("Global is not defined"))?
                    }
                }

                // statements
                Print => {
                    self.print();
                }
                Pop => {
                    self.pop_stack();
                }
            }
        }
    }

    fn string_concatenate(&mut self) -> Result<(), InterpretError> {
        let rhs = self.pop_stack()?;
        let lhs = self.pop_stack()?;
        let it = self.heap.alloc(Obj::String {
            str: lhs.as_string().to_string() + rhs.as_string(),
        });
        self.push_stack(Object(it));
        Ok(())
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

    fn print(&mut self) -> Result<(), InterpretError> {
        let it = self.pop_stack()?;
        println!("PRINTED: {:?}", &it);
        // Push the value back onto the stack so we can still return it
        self.push_stack(it);
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::parser::Parser;
    use crate::tokenizer::Tokenizer;

    #[test]
    fn interpret_math_expression_with_precedence() {
        interpret_result(vec![("10 + 30 * 2", 70.0)]);
    }

    #[test]
    fn interpret_booleans() {
        interpret_result(vec![("true", true), ("false", false)])
    }

    #[test]
    fn interpret_nil() {
        interpret_result(vec![("nil", Returned::Nil)])
    }

    #[test]
    fn interpret_not() {
        interpret_result(vec![
            ("!false", true),
            ("!true", false),
            ("!!true", true),
            ("!!false", false),
            ("!(5 == 5)", false),
            ("!nil", true),
            ("!0", true),
            ("!1", false),
            ("!-1", false),
        ])
    }

    #[test]
    fn interpret_equal() {
        interpret_result(vec![
            ("100 == 100", true),
            ("100 == 10", false),
            ("true == true", true),
            ("true == false", false),
            ("nil == nil", true),
            ("true == 10", false),
            ("100 == nil", false),
            ("false == nil", false),
            ("true == 1", false),
        ])
    }

    #[test]
    fn interpret_not_equal() {
        interpret_result(vec![
            ("100 != 100", false),
            ("100 != 10", true),
            ("true != true", false),
            ("true != false", true),
            ("nil != nil", false),
            ("true != 10", true),
            ("100 != nil", true),
            ("false != nil", true),
            ("true != 1", true),
        ]);
    }

    #[test]
    fn interpret_greater() {
        interpret_result(vec![
            ("100 > 100", false),
            ("100 > 10", true),
            ("10 > 100", false),
        ])
    }

    #[test]
    fn interpret_greater_equal() {
        interpret_result(vec![
            ("100 >= 100", true),
            ("100 >= 10", true),
            ("10 >= 100", false),
        ])
    }

    #[test]
    fn interpret_less() {
        interpret_result(vec![
            ("100 < 100", false),
            ("100 < 10", false),
            ("10 < 100", true),
        ])
    }
    #[test]
    fn interpret_less_equal() {
        interpret_result(vec![
            ("100 <= 100", true),
            ("100 <= 10", false),
            ("10 <= 100", true),
        ])
    }
    #[test]
    fn interpret_expression() {
        interpret_result(vec![("!(5 - 4 > 3 * 2 == !nil)", true)])
    }

    #[test]
    fn interpret_strings() {
        interpret_result(vec![("\"hello world\"", "hello world")]);
    }

    #[test]
    fn interpret_string_equality() {
        interpret_result(vec![
            ("\"ok\" == \"ok\"", true),
            ("\"ok\" == \"nok\"", false),
            ("\"ok\" != \"nok\"", true),
            ("\"ok\" != \"ok\"", false),
        ])
    }

    #[test]
    fn interpret_string_concatenation() {
        interpret_result(vec![
            ("\"hello \" + \"world\"", "hello world"),
            ("\"hello\" + \" \"  + \"world\"", "hello world"),
        ])
    }

    #[test]
    fn interpret_print_statement() {
        interpret_result(vec![("print 5 + 2;", 7.0)]);

        interpret_result(vec![
            ("print 5 > 2;", true),
            ("print 5 >= 5;", true),
            ("print 5 <= 7;", true),
            ("print 5 != 7;", true),
        ]);

        interpret_result(vec![
            ("print \"hello \" + \"world\";", "hello world"),
            ("\"hello\" + \" \"  + \"world\"", "hello world"),
        ])
    }

    #[test]
    fn interpret_var_statements() {
        interpret_result(vec![("var summed = 5 + 2; print summed *2;", 14.0)]);
    }

    #[test]
    fn interpret_unknown_globals_are_nil() {
        // @TODO treat as runtime error instead
        interpret_result(vec![("print unknown;", Value::Nil)]);
    }

    #[test]
    fn interpret_set_global() {
        interpret_result(vec![("var it; it = 3 + 5; print it;", 8.0)]);
    }

    #[test]
    #[should_panic]
    fn interpret_set_global_illegal_grammar() {
        // This should throw because `a * b = 3 + 8;` mixes variable assignment
        // in an expression which is nonsense
        // Proper way for writing is:
        // var b = 3 + 8;
        //  1 * b;
        // print b;
        interpret_result(vec![("var b; 1 * b = 3 + 8; print b;", 11.0)]);
    }

    #[test]
    #[should_panic]
    fn interpret_set_global_undefined() {
        // throws error global not defined
        interpret_result(vec![("var it; unknown = 3 + 5; print unknown;", 8.0)]);
    }

    fn interpret_result<T>(cases: Vec<(&str, T)>)
    where
        Returned: From<T>,
    {
        for (source, expected) in cases {
            let chunk = Parser::parse(Tokenizer::new(source)).unwrap();
            let result = interpret(&chunk).unwrap();

            assert_eq!(result, Returned::from(expected));
        }
    }
}
