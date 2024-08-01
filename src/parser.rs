use crate::chunk::Chunk;
use crate::compiler::{Compiler, LocalVarResolution};
use crate::opcode::OpCode::{False, Nil, Return, True};
use crate::opcode::Value::Number;
use crate::opcode::{OpCode, Value};
use crate::tokenizer::{Token, TokenKind, Tokenizer};
use crate::vm::CompilationErrorReason::{
    ExpectedBinaryOperator, ExpectedDifferentToken, ExpectedPrefix, ExpectedRightParen,
    NotEnoughTokens, ParseFloatError, TooMayTokens,
};
use crate::vm::InterpretError;
use crate::vm::InterpretError::{CompileError, RuntimeErrorWithReason};

#[derive(Debug)]
pub struct Parser<'a> {
    tokenizer: Tokenizer<'a>,
    // Its weird that the parser owns the compiler, would seem to be the other way around
    // @TODO fix it
    compiler: Compiler,
    chunk: Chunk,
    current: Option<Token<'a>>,
    line: usize, // cache latest line
}

impl<'a> Parser<'a> {
    pub fn new(tokenizer: Tokenizer<'a>) -> Self {
        Self {
            tokenizer,
            compiler: Compiler::new(),
            chunk: Chunk::new(),
            current: None,
            line: 0,
        }
    }

    pub fn parse(tokenizer: Tokenizer) -> Result<Chunk, InterpretError> {
        let mut it = Parser::new(tokenizer);
        it.advance(); // Loads the first token in current
        while it.current.as_ref().is_some() {
            it.parse_declaration()?;
        }
        it.expect_done()?;
        it.end()?;
        Ok(it.chunk)
    }

    fn current(&self) -> Result<&Token<'a>, InterpretError> {
        self.current.as_ref().ok_or(CompileError(NotEnoughTokens))
    }

    fn expect_done(&self) -> Result<(), InterpretError> {
        if self.current.is_none() {
            Ok(())
        } else {
            Err(CompileError(TooMayTokens))
        }
    }

    fn expect(&self, expected: TokenKind) -> Result<(), InterpretError> {
        match self.current()?.kind {
            it if it == expected => Ok(()),
            received => Err(CompileError(ExpectedDifferentToken { expected, received })),
        }
    }

    fn advance(&mut self) {
        self.current = self.tokenizer.next();
        if let Some(token) = self.current.as_ref() {
            self.line = token.line
        }
    }

    // if the current token is what it expected, consume it
    fn expect_advance(
        &mut self,
        token: TokenKind,
        error: &'static str,
    ) -> Result<(), InterpretError> {
        match self.current()?.kind {
            it if it == token => {
                self.advance();
                Ok(())
            }
            _ => Err(InterpretError::RuntimeErrorWithReason(error)),
        }
    }

    fn parse_expression(&mut self, precedence: i32) -> Result<(), InterpretError> {
        // prefix / nud position
        match self.current()?.kind {
            TokenKind::Number => self.parse_number(),
            TokenKind::String => self.parse_string(),
            TokenKind::False | TokenKind::True | TokenKind::Nil => self.parse_literal(),
            TokenKind::LeftParen => self.parse_grouping(),
            TokenKind::Minus | TokenKind::Bang => self.parse_unary(),
            TokenKind::Identifier => self.parse_named_variable(precedence),
            TokenKind::Return => self.parse_return(),
            it => {
                println!("token not handled: {:?}", it);
                todo!()
            }
        }?;

        while let Some(op) = self.current.as_ref() {
            if self.precedence(op.kind) > precedence {
                self.parse_binary()?;
            } else {
                break;
            }
        }

        Ok(())
    }

    fn precedence(&self, token: TokenKind) -> i32 {
        match token {
            TokenKind::Equal => 10,
            TokenKind::Or => 30,
            TokenKind::And => 40,
            TokenKind::EqualEqual | TokenKind::BangEqual => 50,
            TokenKind::Less
            | TokenKind::Greater
            | TokenKind::LessEqual
            | TokenKind::GreaterEqual => 60,
            TokenKind::Minus | TokenKind::Plus => 70,
            TokenKind::Star | TokenKind::Slash => 80,
            TokenKind::Bang => 90, // missing -
            // UNARY,       // ! -
            // CALL,        // . ()
            // PRIMARY
            _ => 0,
        }
    }

    fn end(&mut self) -> Result<(), InterpretError> {
        self.emit_return(self.line)?;
        Ok(())
    }

    fn parse_number(&mut self) -> Result<(), InterpretError> {
        let it = self
            .current()?
            .source
            .parse::<f64>()
            .map_err(|it| CompileError(ParseFloatError))?;
        let line = self.line;
        self.advance();
        self.emit_constant(Number(it), line);
        Ok(())
    }

    fn parse_string(&mut self) -> Result<(), InterpretError> {
        let it = self
            .current()?
            .source
            .strip_prefix('"')
            .expect("source strings start with \"")
            .strip_suffix('"')
            .expect("source strings start with \"")
            .to_string();
        let line = self.line;
        self.advance();
        self.emit_string(it, line);
        Ok(())
    }

    fn parse_named_variable(&mut self, precedence: i32) -> Result<(), InterpretError> {
        let name = self.parse_var_name()?;
        let line = self.line;
        let is_local_var = self.compiler.resolve_local_variable(name.as_str());
        // Trying to assign while we are in a statement like `2 * b = 3 + 5`
        // b should not be assigned here
        // we know this because the * pushes a higher precedence level then =
        // what is legal is just setting the variable:
        // var x;
        // x = 15; <- this is what we want to allow here
        let can_assign = precedence <= self.precedence(TokenKind::Equal);
        match self.current()?.kind {
            TokenKind::Equal if can_assign => {
                self.advance();
                self.parse_expression(0)?;
                self.expect_advance(
                    TokenKind::Semicolon,
                    "Expected ';' after variable declaration",
                )?;
                match is_local_var {
                    LocalVarResolution::FoundAt(at) => self.emit_set_local_var(at, line)?,
                    LocalVarResolution::NotFound => self.emit_set_global_var(name, line)?,
                }
            }
            // Not allowed to assign
            TokenKind::Equal => Err(RuntimeErrorWithReason("Invalid assignment target"))?,
            _ => match is_local_var {
                LocalVarResolution::FoundAt(at) => self.emit_get_local_var(at, line)?,
                LocalVarResolution::NotFound => self.emit_get_global_var(name, line)?,
            },
        }

        Ok(())
    }

    fn parse_grouping(&mut self) -> Result<(), InterpretError> {
        self.advance(); // consume '('
        self.parse_expression(0);
        match self.current()?.kind {
            TokenKind::RightParen => self.advance(), // consume ')'
            _ => Err(CompileError(ExpectedRightParen))?,
        }
        Ok(())
    }

    fn parse_unary(&mut self) -> Result<(), InterpretError> {
        let kind = self.current()?.kind;
        let line = self.line;

        match kind {
            TokenKind::Minus => {
                self.advance();
                self.parse_expression(self.precedence(kind));
                self.emit_op_code(OpCode::Negate, line)?
            }
            TokenKind::Bang => {
                self.advance();
                self.parse_expression(self.precedence(kind));
                self.emit_op_code(OpCode::Not, line)?
            }
            _ => Err(CompileError(ExpectedPrefix))?,
        }

        Ok(())
    }

    fn parse_literal(&mut self) -> Result<(), InterpretError> {
        let kind = self.current()?.kind;
        macro_rules! emit {
            ($variant:ident) => {{
                let line = self.line;
                self.advance();
                self.emit_op_code($variant, line)?
            }};
        }

        match kind {
            TokenKind::False => emit!(False),
            TokenKind::True => emit!(True),
            TokenKind::Nil => emit!(Nil),
            _ => Err(CompileError(ExpectedPrefix))?,
        }

        Ok(())
    }

    fn parse_binary(&mut self) -> Result<(), InterpretError> {
        let kind = self.current()?.kind;
        let line = self.line;

        match kind {
            TokenKind::Plus => {
                self.advance();
                self.parse_expression(self.precedence(kind))?;
                self.emit_op_code(OpCode::Add, line)
            }
            TokenKind::Minus => {
                self.advance();
                self.parse_expression(self.precedence(kind))?;
                self.emit_op_code(OpCode::Subtract, line)
            }
            TokenKind::Star => {
                self.advance();
                self.parse_expression(self.precedence(kind))?;
                self.emit_op_code(OpCode::Multiply, line)
            }
            TokenKind::Slash => {
                self.advance();
                self.parse_expression(self.precedence(kind))?;
                self.emit_op_code(OpCode::Divide, line)
            }
            TokenKind::EqualEqual => {
                self.advance();
                self.parse_expression(self.precedence(kind))?;
                self.emit_op_code(OpCode::Equal, line)
            }
            TokenKind::BangEqual => {
                self.advance();
                self.parse_expression(self.precedence(kind))?;
                self.emit_op_codes(OpCode::Equal, OpCode::Not, line)
            }
            TokenKind::Greater => {
                self.advance();
                self.parse_expression(self.precedence(kind))?;
                self.emit_op_code(OpCode::Greater, line)
            }
            TokenKind::GreaterEqual => {
                self.advance();
                self.parse_expression(self.precedence(kind))?;
                self.emit_op_codes(OpCode::Less, OpCode::Not, line)
            }
            TokenKind::Less => {
                self.advance();
                self.parse_expression(self.precedence(kind))?;
                self.emit_op_code(OpCode::Less, line)
            }
            TokenKind::LessEqual => {
                self.advance();
                self.parse_expression(self.precedence(kind))?;
                self.emit_op_codes(OpCode::Greater, OpCode::Not, line)
            }
            _ => Err(CompileError(ExpectedBinaryOperator))?,
        };

        Ok(())
    }

    fn emit_op_code(&mut self, code: OpCode, line: usize) -> Result<(), InterpretError> {
        // @TODO revisit as it might need to be configurable which chunk to write too
        self.chunk.write_code(code, line);
        Ok(())
    }

    fn emit_op_codes(
        &mut self,
        code1: OpCode,
        code2: OpCode,
        line: usize,
    ) -> Result<(), InterpretError> {
        self.emit_op_code(code1, line)?;
        self.emit_op_code(code2, line)?;
        Ok(())
    }

    fn emit_constant(&mut self, constant: Value, line: usize) -> Result<(), InterpretError> {
        // @TODO error handling out of range
        self.chunk.write_constant(constant, line);
        Ok(())
    }

    fn emit_string(&mut self, str: std::string::String, line: usize) -> Result<(), InterpretError> {
        // @TODO error handling out of range
        self.chunk.write_string(str, line);
        Ok(())
    }

    fn emit_define_global_var(
        &mut self,
        str: std::string::String,
        line: usize,
    ) -> Result<(), InterpretError> {
        // @TODO error handling out of range
        self.chunk.write_define_global_var(str, line);
        Ok(())
    }

    fn emit_set_global_var(
        &mut self,
        str: std::string::String,
        line: usize,
    ) -> Result<(), InterpretError> {
        // @TODO error handling out of range
        self.chunk.write_set_global_var(str, line);
        Ok(())
    }

    fn emit_set_local_var(&mut self, at: usize, line: usize) -> Result<(), InterpretError> {
        self.chunk.write_set_local_var(at, line);
        Ok(())
    }

    fn emit_get_global_var(
        &mut self,
        str: std::string::String,
        line: usize,
    ) -> Result<(), InterpretError> {
        // @TODO error handling out of range
        self.chunk.write_get_global_var(str, line);
        Ok(())
    }

    fn emit_get_local_var(&mut self, at: usize, line: usize) -> Result<(), InterpretError> {
        self.chunk.write_get_local_var(at, line);
        Ok(())
    }

    fn emit_return(&mut self, line: usize) -> Result<(), InterpretError> {
        self.emit_op_code(OpCode::Return, line)?;
        Ok(())
    }

    // declarations: statements that bind a new name (variable) to a value
    // If nothing find, starts parsing statements
    fn parse_declaration(&mut self) -> Result<(), InterpretError> {
        match self.current()?.kind {
            TokenKind::Var => self.parse_var_declaration(),
            _ => self.parse_statement(),
        }
        // @TODO implement synchronize to recover from errors
    }

    // all other statements
    fn parse_statement(&mut self) -> Result<(), InterpretError> {
        match self.current()?.kind {
            TokenKind::Print => self.parse_print_statement(),
            TokenKind::LeftBrace => self.parse_block(),
            // @TODO replace parse_expression by parse_expression_statement and no longer return value from interpret
            _ => self.parse_expression(0),
            // _ => self.parse_expression_statement(),
        }
    }

    fn parse_print_statement(&mut self) -> Result<(), InterpretError> {
        self.advance();
        self.parse_expression(0)?;
        self.expect_advance(TokenKind::Semicolon, "Expected ';' after value");
        self.emit_op_code(OpCode::Print, self.line)
    }

    // Evaluates the expression and throws away the result
    fn parse_expression_statement(&mut self) -> Result<(), InterpretError> {
        self.parse_expression(0);
        self.expect_advance(TokenKind::Semicolon, "Expected ';' after value");
        self.emit_op_code(OpCode::Pop, self.line)
    }

    fn parse_var_declaration(&mut self) -> Result<(), InterpretError> {
        self.advance();
        let name = self.parse_var_name()?;

        match self.current()?.kind {
            TokenKind::Equal => {
                self.advance();
                self.parse_expression(0)
            }
            // var a; becomes var a = nil;
            _ => self.emit_op_code(OpCode::Nil, self.line),
        }?;

        self.expect_advance(
            TokenKind::Semicolon,
            "Expected ';' after variable declaration",
        )?;

        match self.compiler.in_local_scope() {
            true => self.declare_local_var(name),
            false => self.emit_define_global_var(name, self.line),
        }
    }

    fn parse_var_name(&mut self) -> Result<String, InterpretError> {
        let it = if self.current()?.kind == TokenKind::Identifier {
            Ok(self.current()?.source.to_string())
        } else {
            Err(InterpretError::RuntimeErrorWithReason(
                "Expected variable name",
            ))
        };
        self.advance();
        it
    }

    // parses block statement like `{ var x = 34; }
    fn parse_block(&mut self) -> Result<(), InterpretError> {
        self.advance();
        self.compiler.begin_scope()?;

        while !self.current()?.is_kind(TokenKind::RightBrace)
            && !self.current()?.is_kind(TokenKind::Eof)
        {
            self.parse_declaration()?;
        }

        let mut local_vars_to_pop = self.compiler.end_scope()?;
        // Pop the local vars from the stack as they are out of scope
        // becomes more complicated once we work with real stack frames
        while local_vars_to_pop > 0 {
            self.emit_op_code(OpCode::Pop, self.line)?;
            local_vars_to_pop -= 1;
        }

        self.expect_advance(TokenKind::RightBrace, "Expect '}' after block");

        Ok(())
    }

    fn declare_local_var(&mut self, name: String) -> Result<(), InterpretError> {
        self.compiler.add_local_var(name)?;
        Ok(())
    }

    fn parse_return(&mut self) -> Result<(), InterpretError> {
        self.advance();

        match self.current()?.kind {
            TokenKind::Semicolon => self.emit_op_code(Nil, self.line),
            _ => self.parse_expression(0),
        };

        self.expect_advance(
            TokenKind::Semicolon,
            "Expected ';' after variable declaration",
        )?;
        self.emit_op_code(Return, self.line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handles_new() {
        println!("{:?}", Parser::new(Tokenizer::new("10+10")))
    }

    #[test]
    fn parse_1() {
        let it = Parser::parse(Tokenizer::new("10 + 30"));

        assert!(it.is_ok());

        let output = it.unwrap().disassemble_into_string("parse 1");
        let expected = r#"
== parse 1 ==
       0        0 | Constant 10.0
       2        0 | Constant 30.0
       4        0 | Add
       5        0 | Return
"#;
        assert_eq!(output, expected);
    }

    #[test]
    fn parse_2() {
        let it = Parser::parse(Tokenizer::new("10 + 30 * 40"));

        assert!(it.is_ok());

        let output = it.unwrap().disassemble_into_string("parse 2");
        let expected = r#"
== parse 2 ==
       0        0 | Constant 10.0
       2        0 | Constant 30.0
       4        0 | Constant 40.0
       6        0 | Multiply
       7        0 | Add
       8        0 | Return
"#;
        assert_eq!(output, expected);
    }

    #[test]
    fn parse_3() {
        let it = Parser::parse(Tokenizer::new("(10 + 30) * 40"));

        assert!(it.is_ok());

        let output = it.unwrap().disassemble_into_string("parse 3");
        let expected = r#"
== parse 3 ==
       0        0 | Constant 10.0
       2        0 | Constant 30.0
       4        0 | Add
       5        0 | Constant 40.0
       7        0 | Multiply
       8        0 | Return
"#;
        assert_eq!(output, expected);
    }

    #[test]
    fn parse_4() {
        let it = Parser::parse(Tokenizer::new("(10 + -30) * 40"));

        assert!(it.is_ok());

        let output = it.unwrap().disassemble_into_string("parse 4");
        let expected = r#"
== parse 4 ==
       0        0 | Constant 10.0
       2        0 | Constant 30.0
       4        0 | Negate
       5        0 | Add
       6        0 | Constant 40.0
       8        0 | Multiply
       9        0 | Return
"#;
        assert_eq!(output, expected);
    }

    #[test]
    fn parse_5() {
        let it = Parser::parse(Tokenizer::new("\"hello world\""));

        assert!(it.is_ok());

        let output = it.unwrap().disassemble_into_string("parse 5");
        let expected = r#"
== parse 5 ==
       0        0 | String "hello world"
       2        0 | Return
"#;
        assert_eq!(output, expected);
    }

    #[test]
    fn parse_print_statement() {
        let it = Parser::parse(Tokenizer::new("print \"hello world\";"));

        assert!(it.is_ok());

        let output = it.unwrap().disassemble_into_string("parse print statement");
        let expected = r#"
== parse print statement ==
       0        0 | String "hello world"
       2        0 | Print
       3        0 | Return
"#;
        assert_eq!(output, expected);
    }

    #[test]
    fn parse_var_declaration_1() {
        let it = Parser::parse(Tokenizer::new("var it = 5 + 3;"));

        assert!(it.is_ok());

        let output = it
            .unwrap()
            .disassemble_into_string("parse var declaration 1");
        let expected = r#"
== parse var declaration 1 ==
       0        0 | Constant 5.0
       2        0 | Constant 3.0
       4        0 | Add
       5        0 | Global define "it"
       7        0 | Return
"#;
        assert_eq!(output, expected);
    }

    #[test]
    fn parse_var_declaration_2() {
        let it = Parser::parse(Tokenizer::new("var it = hello;"));

        assert!(it.is_ok());

        let output = it
            .unwrap()
            .disassemble_into_string("parse var declaration 2");
        let expected = r#"
== parse var declaration 2 ==
       0        0 | Global get "hello"
       2        0 | Global define "it"
       4        0 | Return
"#;
        assert_eq!(output, expected);
    }

    #[test]
    fn parse_var_declaration_3() {
        let it = Parser::parse(Tokenizer::new("var it; it = 3 + 5; print it;"));

        assert!(it.is_ok());

        let output = it
            .unwrap()
            .disassemble_into_string("parse var declaration 2");
        let expected = r#"
== parse var declaration 2 ==
       0        0 | Nil
       1        0 | Global define "it"
       3        0 | Constant 3.0
       5        0 | Constant 5.0
       7        0 | Add
       8        0 | Global set "it"
      10        0 | Global get "it"
      12        0 | Print
      13        0 | Return
"#;
        assert_eq!(output, expected);
    }
}
