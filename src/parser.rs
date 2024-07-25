use crate::chunk::Chunk;
use crate::opcode::OpCode;
use crate::tokenizer::{Token, TokenKind, Tokenizer};
use crate::vm::CompilationErrorReason::{
    ExpectedBinaryOperator, ExpectedDifferentToken, ExpectedPrefix, ExpectedRightParen,
    NotEnoughTokens, ParseFloatError,
};
use crate::vm::InterpretError;
use crate::vm::InterpretError::CompileError;

#[derive(Debug)]
pub struct Parser<'a> {
    tokenizer: Tokenizer<'a>,
    chunk: Chunk,
    current: Option<Token<'a>>,
    line: usize, // cache latest line
}

impl<'a> Parser<'a> {
    pub fn new(tokenizer: Tokenizer<'a>) -> Self {
        Self {
            tokenizer,
            chunk: Chunk::new(),
            current: None,
            line: 0,
        }
    }

    pub fn parse(tokenizer: Tokenizer) -> Result<Chunk, InterpretError> {
        let mut it = Parser::new(tokenizer);
        it.advance(); // Loads the first token in current
        it.parse_expression(0)?;
        it.expect_done();
        it.end();
        Ok(it.chunk)
    }

    fn current(&self) -> Result<&Token<'a>, InterpretError> {
        self.current.as_ref().ok_or(CompileError(NotEnoughTokens))
    }

    fn expect_done(&self) -> bool {
        self.current.is_none()
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

    fn parse_expression(&mut self, precedence: i32) -> Result<(), InterpretError> {
        // prefix / nud position
        match self.current()?.kind {
            TokenKind::Number => self.parse_number(),
            TokenKind::LeftParen => self.parse_grouping(),
            TokenKind::Minus => self.parse_unary(),
            _ => todo!(),
        }?;

        while let Some(op) = self.current.as_ref() {
            if self.precedence(op.kind) > precedence {
                self.parse_binary();
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

    fn end(&mut self) {
        self.emit_return(self.line);
    }

    fn parse_number(&mut self) -> Result<(), InterpretError> {
        let it = self
            .current()?
            .source
            .parse::<f64>()
            .map_err(|it| CompileError(ParseFloatError))?;
        let line = self.line;
        self.advance();
        self.emit_constant(it, line);
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
        self.advance();

        match kind {
            TokenKind::Minus => {
                self.parse_expression(self.precedence(kind));
                self.emit_op_code(OpCode::Negate, line)?
            }
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
            _ => Err(CompileError(ExpectedBinaryOperator))?,
        };

        Ok(())
    }

    fn emit_op_code(&mut self, code: OpCode, line: usize) -> Result<(), InterpretError> {
        // @TODO revisit as it might need to be configurable which chunk to write too
        self.chunk.write_code(code, line);
        Ok(())
    }

    fn emit_bytes(&mut self, code1: OpCode, code2: OpCode, line: usize) {
        self.emit_op_code(code1, line);
        self.emit_op_code(code2, line);
    }

    fn emit_constant(&mut self, constant: f64, line: usize) -> Result<(), InterpretError> {
        // @TODO error handling out of range
        self.chunk.write_constant(constant, line);
        Ok(())
    }

    fn emit_return(&mut self, line: usize) {
        self.emit_op_code(OpCode::Return, line);
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
}
