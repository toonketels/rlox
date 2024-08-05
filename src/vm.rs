use crate::chunk::{Chunk, Jump};
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
    ScopeUnderflow,
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
    JumpTooFar,
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
            InterpretError::JumpTooFar => write!(f, "jump too far"),
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

    fn read_jump(&mut self) -> Option<Jump> {
        let at = self.advance(); // start of jump code
        self.advance(); // advance once more because a jump is 2 bytes long
        self.chunk.read_jump(at)
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

    fn peek_stack(&self, offset: usize) -> Option<&Value> {
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
                    // there should be just one value on the stack which will be popped before we exit

                    let it = self.pop_stack()?;

                    if !self.stack.is_empty() {
                        // Currently, we can do an early return and still have some items on the stack
                        println!("stack not empty: {:?}", self.stack);
                        // Err(RuntimeErrorWithReason(
                        //     "Program terminating but stack is not empty",
                        // ))?;
                    }
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
                    // we dont pop from the stack according to the book
                    // that seems odd so we dont
                    // => We dont because this is an expression statement which will auto pop the stack
                    let value = self.peek_stack(0).ok_or(StackUnderflowError)?.clone();
                    if let std::collections::hash_map::Entry::Occupied(mut e) =
                        self.globals.entry(name)
                    {
                        e.insert(value);
                    } else {
                        Err(RuntimeErrorWithReason("Global is not defined"))?
                    }
                }

                GetLocal => {
                    // next byte contains the local_var_offset
                    let at = self.read_byte().ok_or(RuntimeError)?;
                    let value = self.stack.get(at as usize).ok_or(RuntimeErrorWithReason(
                        "Local variable value could not be found",
                    ))?;
                    self.push_stack(value.clone());
                }

                SetLocal => {
                    // next byte contains the local_var_offset
                    let at = self.read_byte().ok_or(RuntimeError)?;
                    // According to the book, we should just peek the stack to not modify if but
                    // then our stack just keeps growing so better pop it.
                    // => We dont because this is an expression statement which will auto pop the stack
                    let value = self.peek_stack(0).ok_or(StackUnderflowError)?;
                    self.stack.set(at as usize, value.clone());
                }

                // statements
                Print => {
                    self.print()?;
                }
                Pop => {
                    self.pop_stack()?;
                }
                // control flow
                JumpIfFalse => {
                    // Always read the jump as it will update the ip past the Jump bytes
                    // which we need if we dont jump so the next instruction to fetch
                    // on true if the on true block
                    let distance = self.read_jump().ok_or(RuntimeError)?;
                    if !self.peek_stack(0).ok_or(StackUnderflowError)?.is_truthy() {
                        self.jump_forward(distance)
                    }
                }
                JumpIfTrue => {
                    // Always read the jump as it will update the ip past the Jump bytes
                    // which we need if we dont jump so the next instruction to fetch
                    // on false if the on false block
                    let distance = self.read_jump().ok_or(RuntimeError)?;
                    if self.peek_stack(0).ok_or(StackUnderflowError)?.is_truthy() {
                        self.jump_forward(distance)
                    }
                }

                Jump => {
                    let distance = self.read_jump().ok_or(RuntimeError)?;
                    self.jump_forward(distance)
                }

                Loop => {
                    let distance = self.read_jump().ok_or(RuntimeError)?;
                    self.jump_backward(distance)
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
        // According to the book: `Print is a statement so does not modify the stack`
        // But that means our stack just keeps growing?
        // Lets just pop the argument to print so that after print our
        // stack is back where it was so
        // our program exists correctly with an empty stack
        let it = self.pop_stack()?;
        println!("PRINTED: {:?}", &it);
        Ok(())
    }

    fn jump_forward(&mut self, jump: Jump) {
        self.ip += jump.distance as usize;
    }

    fn jump_backward(&mut self, jump: Jump) {
        self.ip -= jump.distance as usize;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::opcode::Value::Nil;
    use crate::parser::Parser;
    use crate::tokenizer::Tokenizer;

    #[test]
    fn interpret_math_expression_with_precedence() {
        interpret_result(vec![("return 10 + 30 * 2;", 70.0)]);
    }

    #[test]
    fn interpret_booleans() {
        interpret_result(vec![("return true;", true), ("return false;", false)])
    }

    #[test]
    fn interpret_nil() {
        interpret_result(vec![
            ("return nil;", Returned::Nil),
            ("return;", Returned::Nil),
        ])
    }

    #[test]
    fn interpret_not() {
        interpret_result(vec![
            ("return !false;", true),
            ("return !true;", false),
            ("return !!true;", true),
            ("return !!false;", false),
            ("return !(5 == 5);", false),
            ("return !nil;", true),
            ("return !0;", true),
            ("return !1;", false),
            ("return !-1;", false),
        ])
    }

    #[test]
    fn interpret_equal() {
        interpret_result(vec![
            ("return 100 == 100;", true),
            ("return 100 == 10;", false),
            ("return true == true;", true),
            ("return true == false;", false),
            ("return nil == nil;", true),
            ("return true == 10;", false),
            ("return 100 == nil;", false),
            ("return false == nil;", false),
            ("return true == 1;", false),
        ])
    }

    #[test]
    fn interpret_not_equal() {
        interpret_result(vec![
            ("return 100 != 100;", false),
            ("return 100 != 10;", true),
            ("return true != true;", false),
            ("return true != false;", true),
            ("return nil != nil;", false),
            ("return true != 10;", true),
            ("return 100 != nil;", true),
            ("return false != nil;", true),
            ("return true != 1;", true),
        ]);
    }

    #[test]
    fn interpret_greater() {
        interpret_result(vec![
            ("return 100 > 100;", false),
            ("return 100 > 10;", true),
            ("return 10 > 100;", false),
        ])
    }

    #[test]
    fn interpret_greater_equal() {
        interpret_result(vec![
            ("return 100 >= 100;", true),
            ("return 100 >= 10;", true),
            ("return 10 >= 100;", false),
        ])
    }

    #[test]
    fn interpret_less() {
        interpret_result(vec![
            ("return 100 < 100;", false),
            ("return 100 < 10;", false),
            ("return 10 < 100;", true),
        ])
    }
    #[test]
    fn interpret_less_equal() {
        interpret_result(vec![
            ("return 100 <= 100;", true),
            ("return 100 <= 10;", false),
            ("return 10 <= 100;", true),
        ])
    }

    #[test]
    fn interpret_and_expression() {
        interpret_result(vec![
            ("return true and false;", false),
            ("return true and true;", true),
            ("return false and true;", false),
            ("return false and false;", false),
        ])
    }

    #[test]
    fn interpret_or_expression() {
        interpret_result(vec![
            ("return true or false;", true),
            ("return true or true;", true),
            ("return false or true;", true),
            ("return false or false;", false),
        ])
    }

    #[test]
    fn interpret_expression() {
        interpret_result(vec![("return !(5 - 4 > 3 * 2 == !nil);", true)])
    }

    #[test]
    fn interpret_strings() {
        interpret_result(vec![("return \"hello world\";", "hello world")]);
    }

    #[test]
    fn interpret_string_equality() {
        interpret_result(vec![
            ("return \"ok\" == \"ok\";", true),
            ("return \"ok\" == \"nok\";", false),
            ("return \"ok\" != \"nok\";", true),
            ("return \"ok\" != \"ok\";", false),
        ])
    }

    #[test]
    fn interpret_string_concatenation() {
        interpret_result(vec![
            ("return \"hello \" + \"world\";", "hello world"),
            ("return \"hello\" + \" \"  + \"world\";", "hello world"),
        ])
    }

    #[test]
    fn interpret_print_statement() {
        interpret_result(vec![("return 5 + 2;", 7.0)]);

        interpret_result(vec![
            ("return 5 > 2;", true),
            ("return 5 >= 5;", true),
            ("return 5 <= 7;", true),
            ("return 5 != 7;", true),
        ]);

        interpret_result(vec![
            ("return \"hello \" + \"world\";", "hello world"),
            ("return \"hello\" + \" \"  + \"world\";", "hello world"),
        ])
    }

    #[test]
    fn interpret_var_statements() {
        interpret_result(vec![
            (
                "var summed = 5 + 2; print summed *2; return summed * 2;",
                14.0,
            ),
            (
                "var a; var b; var c; a = 5; b = 2; c = a + b; print c *2; return c * 2;",
                14.0,
            ),
        ]);
    }

    #[test]
    fn interpret_unknown_globals_are_nil() {
        // @TODO treat as runtime error instead
        interpret_result(vec![("return unknown;", Value::Nil)]);
    }

    #[test]
    fn interpret_set_global() {
        interpret_result(vec![("var it; it = 3 + 5; return it;", 8.0)]);
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
        interpret_result(vec![("var b; 1 * b = 3 + 8; return b;", 11.0)]);
    }

    #[test]
    #[should_panic]
    fn interpret_set_global_undefined() {
        // throws error global not defined
        interpret_result(vec![("var it; unknown = 3 + 5; return unknown;", 8.0)]);
    }

    #[test]
    fn interpret_block_statements_1() {
        interpret_result(vec![("{ var x = 15; var y; } return;", Nil)]);
    }

    #[test]
    fn interpret_block_statements_2() {
        interpret_result(vec![("{ var x; x = 10; print x; } return;", Nil)]);
    }

    #[test]
    fn interpret_block_statements_3() {
        interpret_result(vec![("{ var x; print x; } return;", Nil)]);
    }

    #[test]
    fn interpret_block_statements_4() {
        interpret_result(vec![(
            "{ var x; var y; x = 10; y = 20; print x; } return;",
            Nil,
        )]);
    }
    #[test]
    fn interpret_block_statements_5() {
        interpret_result(vec![("var x; { x = 10; var y = 20; } return x;", 10.0)]);
    }

    #[test]
    fn interpret_block_statements_6() {
        interpret_result(vec![(
            "var z; { var x; var y; x = 10; y = 20; z = x + y; } return z;",
            30.0,
        )]);
    }

    #[test]
    fn interpret_block_statements_7() {
        interpret_result(vec![(
            "var z; { var x; var y; x = 10; y = 20; z = y; } return;",
            Nil,
        )]);
    }

    #[test]
    fn interpret_if_statement_true() {
        interpret_result(vec![(
            "var z = 2; if (true) { var x = 3; var y = 5; z = x + y; } return z;",
            8.0,
        )]);
    }

    #[test]
    fn interpret_if_statement_false() {
        interpret_result(vec![(
            "var z = 2; if (false) { var x = 3; var y = 5; z = x + y; } return z;",
            2.0,
        )]);
    }

    #[test]
    fn interpret_if_else_statement_true() {
        interpret_result(vec![(
            "var z = 2; if (true) { var x = 3; var y = 5; z = x + y; } else { z = 200; }return z;",
            8.0,
        )]);
    }

    #[test]
    fn interpret_if_else_statement_false() {
        interpret_result(vec![(
            "var z = 2; if (false) { var x = 3; var y = 5; z = x + y; } else { z = 200; }return z;",
            200.0,
        )]);
    }

    #[test]
    fn interpret_if_else_statement_false_2() {
        interpret_result(vec![(
            "if (false){ var x = 3; var y = 5; } else { var y = 100; } var x = 5; return x +2;",
            7.0,
        )]);
    }

    #[test]
    fn interpret_while_loop() {
        interpret_result(vec![(
            "var x = 0; var y = 3; while (y > 0) { y = y - 1; x = x + 1; } return x;",
            3.0,
        )]);

        interpret_result(vec![
            ("var x = true; while (x) { x  = false; } return x;", false),
            ("var x = true; while (x) { x  = !x; } return x;", false),
        ]);

        interpret_result(vec![(
            "var x = \"hi\"; var y = 3; while (y > 0) { y = y - 1; x = x + \"hi\"; } return x;",
            "hihihihi",
        )])
    }

    #[test]
    fn interpret_for_loop() {
        interpret_result(vec![
            (
                "var x = 0; for (var i = 0; i < 10; i = i + 1) { x = x + 1; } return x;",
                10.0,
            ),
            ("var x = 0; for (; x < 10;) { x = x + 1; } return x;", 10.0),
            (
                "var x = 0; for (;;) { x = x + 1; if (x >= 10) return x; } return x;",
                10.0,
            ),
        ])
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
