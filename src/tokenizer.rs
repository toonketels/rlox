use crate::tokenizer::TokenKind::{Identifier, Number, String};
use std::cmp::PartialEq;

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum TokenKind {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    // Literals.
    Identifier,
    String,
    Number,
    // Keywords.
    And,
    Class,
    Else,
    False,
    For,
    Fun,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Error,
    Eof,
}

// - return errors
//   - maybe Done is a recoverable error
// - peek immediately and see if that simplifies it

trait ByteExtensions {
    fn is_newline(&self) -> bool;
    fn is_alphabetic_or_underscore(&self) -> bool;
}

impl ByteExtensions for u8 {
    fn is_newline(&self) -> bool {
        *self == b'\n'
    }

    fn is_alphabetic_or_underscore(&self) -> bool {
        self.is_ascii_alphabetic() || *self == b'_'
    }
}

#[derive(PartialEq, Debug)]
pub struct Token<'a> {
    pub(crate) kind: TokenKind,
    pub(crate) source: &'a str,
    offset: usize,
    pub(crate) line: usize,
}

impl<'a> Token<'a> {
    pub fn new(kind: TokenKind, source: &'a str, source_offset: usize, line: usize) -> Self {
        Self {
            kind,
            source,
            offset: source_offset,
            line,
        }
    }

    pub fn is_kind(&self, kind: TokenKind) -> bool {
        self.kind == kind
    }
}

#[derive(Debug)]
pub struct Tokenizer<'a> {
    source: &'a str,
    as_bytes: &'a [u8],
    checkpoint: usize, // checkpoint to indicate a start of a token
    current: usize,    // points to the next item to read
    line: usize,
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.token()
    }
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            as_bytes: source.as_bytes(),
            checkpoint: 0,
            current: 0,
            line: 0,
        }
    }

    #[cfg(test)]
    fn rest(&self) -> &'a str {
        &self.source[self.current..]
    }

    #[cfg(test)]
    fn line(&self) -> usize {
        self.line
    }

    fn advance_byte(&mut self) {
        self.current += 1;
    }

    fn advance_bytes(&mut self, amount: usize) {
        assert!(amount > 0, "steps should be a number bigger then 0");
        self.current += amount;
    }

    fn advance_line(&mut self) {
        self.line += 1;
    }

    #[cfg(test)]
    fn drop_byte(&mut self) {
        self.advance_byte();
    }

    fn take_byte(&mut self) -> Option<u8> {
        let current = self.current;
        self.advance_byte();
        if self.current > self.as_bytes.len() {
            None
        } else {
            Some(self.as_bytes[current])
        }
    }

    #[cfg(test)]
    fn take_bytes(&mut self, amount: usize) -> Option<&str> {
        let current = self.current;
        if self.current + amount > self.as_bytes.len() {
            None
        } else {
            self.advance_bytes(amount);
            Some(&self.source[current..current + amount])
        }
    }

    fn take_whitespace(&mut self) {
        loop {
            match self.peek_byte() {
                Some(it) if it.is_ascii_whitespace() => {
                    if it.is_newline() {
                        self.advance_line();
                    }
                    self.advance_byte();
                }
                _ => break,
            }
        }
    }

    fn take_comment(&mut self) {
        while let Some(it) = self.take_byte() {
            if it == b'\n' {
                break;
            }
        }
    }

    fn peek_byte(&self) -> Option<u8> {
        if self.current >= self.as_bytes.len() {
            None
        } else {
            Some(self.as_bytes[self.current])
        }
    }

    fn peek_bytes(&self, amount: usize) -> Option<&str> {
        if self.current + amount > self.as_bytes.len() {
            None
        } else {
            Some(&self.source[self.current..self.current + amount])
        }
    }

    // Checkpoints the last  position
    fn checkpoint(&mut self) -> Option<u8> {
        self.checkpoint = self.current;
        if self.checkpoint >= self.as_bytes.len() {
            None
        } else {
            Some(self.as_bytes[self.checkpoint])
        }
    }

    // Matches the currently consumed byte and the following n one that
    // 1. match 'what'
    // 2. matches on a boundary
    fn match_bytes(&self, what: &str) -> bool {
        let is_match = self.peek_bytes(what.len()) == Some(what);
        let is_exact = match self.as_bytes.get(self.current + what.len()) {
            // Any alpha number or _ makes it not a boundary
            Some(it) if it.is_alphabetic_or_underscore() || it.is_ascii_digit() => false,
            Some(_) => true,
            None => true,
        };
        is_match && is_exact
    }

    fn create_token(&self, kind: TokenKind) -> Token<'a> {
        Token::new(
            kind,
            &self.source[self.checkpoint..self.current],
            self.checkpoint,
            self.line,
        )
    }

    fn make_token_with_length(&mut self, kind: TokenKind, length: usize) -> Option<Token<'a>> {
        assert!(length > 0, "token needs to be at least a byte long");
        self.checkpoint();
        self.advance_bytes(length);
        Some(self.create_token(kind))
    }

    fn make_string(&mut self) -> Option<Token<'a>> {
        self.checkpoint();
        // Skip the opening "
        self.advance_byte();
        // We are not handling newlines in strings as we assume strings are just one line with
        // escaped newline chars in it.
        while let Some(it) = self.take_byte() {
            if it == b'"' {
                return Some(self.create_token(String));
            }
        }
        // @TODO error unterminated string
        None
    }

    fn make_number(&mut self) -> Option<Token<'a>> {
        self.checkpoint();
        // We are not handling newlines in strings as we assume strings are just one line with
        // escaped newline chars in it.
        while let Some(it) = self.peek_byte() {
            if !it.is_ascii_digit() {
                break;
            }
            self.advance_byte();
        }
        return Some(self.create_token(Number));
    }

    fn make_identifier(&mut self) -> Option<Token<'a>> {
        self.checkpoint();
        while let Some(it) = self.peek_byte() {
            if it.is_alphabetic_or_underscore() || it.is_ascii_digit() {
                self.advance_byte();
            } else {
                break;
            }
        }
        Some(self.create_token(Identifier))
    }

    fn token(&mut self) -> Option<Token<'a>> {
        use TokenKind::*;

        match self.peek_byte()? {
            it if it.is_ascii_whitespace() => {
                self.take_whitespace();
                self.token()
            }
            b'(' => self.make_token_with_length(LeftParen, 1),
            b')' => self.make_token_with_length(RightParen, 1),
            b'{' => self.make_token_with_length(LeftBrace, 1),
            b'}' => self.make_token_with_length(RightBrace, 1),
            b';' => self.make_token_with_length(Semicolon, 1),
            b',' => self.make_token_with_length(Comma, 1),
            b'.' => self.make_token_with_length(Dot, 1),
            b'-' => self.make_token_with_length(Minus, 1),
            b'+' => self.make_token_with_length(Plus, 1),
            b'*' => self.make_token_with_length(Star, 1),
            b'/' => match self.peek_bytes(2) {
                Some("//") => {
                    self.take_comment();
                    self.token()
                }
                _ => self.make_token_with_length(Slash, 1),
            },
            b'!' => match self.peek_bytes(2) {
                Some("!=") => self.make_token_with_length(BangEqual, 2),
                _ => self.make_token_with_length(Bang, 1),
            },
            b'=' => match self.peek_bytes(2) {
                Some("==") => self.make_token_with_length(EqualEqual, 2),
                _ => self.make_token_with_length(Equal, 1),
            },
            b'<' => match self.peek_bytes(2) {
                Some("<=") => self.make_token_with_length(LessEqual, 2),
                _ => self.make_token_with_length(Less, 1),
            },
            b'>' => match self.peek_bytes(2) {
                Some(">=") => self.make_token_with_length(GreaterEqual, 2),
                _ => self.make_token_with_length(Greater, 1),
            },
            b'"' => self.make_string(),
            it if it.is_ascii_digit() => self.make_number(),
            _ if self.match_bytes("and") => self.make_token_with_length(And, 3),
            _ if self.match_bytes("class") => self.make_token_with_length(Class, 5),
            _ if self.match_bytes("else") => self.make_token_with_length(Else, 4),
            _ if self.match_bytes("if") => self.make_token_with_length(If, 2),
            _ if self.match_bytes("nil") => self.make_token_with_length(Nil, 3),
            _ if self.match_bytes("or") => self.make_token_with_length(Or, 2),
            _ if self.match_bytes("print") => self.make_token_with_length(Print, 5),
            _ if self.match_bytes("return") => self.make_token_with_length(Return, 6),
            _ if self.match_bytes("super") => self.make_token_with_length(Super, 5),
            _ if self.match_bytes("var") => self.make_token_with_length(Var, 3),
            _ if self.match_bytes("while") => self.make_token_with_length(While, 5),
            _ if self.match_bytes("false") => self.make_token_with_length(False, 5),
            _ if self.match_bytes("for") => self.make_token_with_length(For, 3),
            _ if self.match_bytes("fun") => self.make_token_with_length(Fun, 3),
            _ if self.match_bytes("this") => self.make_token_with_length(This, 4),
            _ if self.match_bytes("true") => self.make_token_with_length(True, 4),
            it if it.is_alphabetic_or_underscore() => self.make_identifier(),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::tokenizer::TokenKind::*;

    #[test]
    fn advance() {
        let mut t = Tokenizer::new("hello world");
        t.advance_byte();
        t.advance_byte();
        assert_eq!(t.current, 2);
        assert_eq!(t.rest(), "llo world");
    }

    #[test]
    fn drop() {
        let mut t = Tokenizer::new("hello world");
        t.drop_byte();
        t.drop_byte();
        assert_eq!(t.current, 2);
        assert_eq!(t.rest(), "llo world");
    }

    #[test]
    fn peek() {
        let mut t = Tokenizer::new("hello world");
        assert_eq!(t.peek_byte(), Some(b'h'));
        assert_eq!(t.peek_byte(), Some(b'h'));
        t.advance_byte();
        assert_eq!(t.peek_byte(), Some(b'e'));
        assert_eq!(t.peek_byte(), Some(b'e'));
        t.advance_bytes(100);
        assert_eq!(t.peek_byte(), None);
    }

    #[test]
    fn peek_more() {
        let mut t = Tokenizer::new("hello world");
        assert_eq!(t.peek_bytes(5), Some("hello"));
        assert_eq!(t.peek_bytes(5), Some("hello"));
        t.advance_bytes(6);
        assert_eq!(t.peek_bytes(5), Some("world"));
        assert_eq!(t.peek_bytes(5), Some("world"));
        assert_eq!(t.peek_bytes(6), None);
    }

    #[test]
    fn take_byte() {
        let mut t = Tokenizer::new("hello world");
        assert_eq!(t.take_byte(), Some(b'h'));
        assert_eq!(t.take_byte(), Some(b'e'));
        t.advance_byte();
        t.advance_byte();
        assert_eq!(t.take_byte(), Some(b'o'));
        t.advance_byte();
        assert_eq!(t.peek_byte(), Some(b'w'));
        assert_eq!(t.take_byte(), Some(b'w'));
        t.advance_bytes(100);
        assert_eq!(t.take_byte(), None);
    }

    #[test]
    fn take_more() {
        let mut t = Tokenizer::new("hello world");
        assert_eq!(t.take_byte(), Some(b'h'));
        assert_eq!(t.take_bytes(4), Some("ello"));
        assert_eq!(t.peek_byte(), Some(b' '));
        t.advance_byte();
        assert_eq!(t.take_bytes(10), None);
        assert_eq!(t.take_bytes(5), Some("world"));
    }

    #[test]
    fn checkpoint() {
        let mut t = Tokenizer::new("hello world");
        t.advance_bytes(6);
        assert_eq!(t.checkpoint(), Some(b'w'));
        assert_eq!(t.take_bytes(5), Some("world"));
    }

    #[test]
    fn make_token() {
        let mut t = Tokenizer::new("hello world");
        assert_eq!(t.checkpoint(), Some(b'h'));
        assert_eq!(t.take_byte(), Some(b'h'));
        assert_eq!(t.take_bytes(4), Some("ello"));
        assert_eq!(t.create_token(String), Token::new(String, "hello", 0, 0));

        t.advance_byte();

        t.checkpoint();
        t.take_bytes(5);
        assert_eq!(t.create_token(String), Token::new(String, "world", 6, 0));
    }

    #[test]
    fn token() {
        let mut t = Tokenizer::new("()");
        assert_eq!(t.token(), Some(Token::new(LeftParen, "(", 0, 0)));
        assert_eq!(t.token(), Some(Token::new(RightParen, ")", 1, 0)));
    }

    fn tokenize(source: &str) -> Vec<TokenKind> {
        let tokenizer = Tokenizer::new(source);

        tokenizer.map(|it| it.kind).collect::<Vec<_>>()
    }

    #[test]
    fn single_tokens() {
        let mut t = Tokenizer::new("()");

        assert_eq!(t.next().map(|it| it.kind), Some(LeftParen));
        assert_eq!(t.next().map(|it| it.kind), Some(RightParen));
    }

    #[test]
    fn single_tokens_2() {
        assert_eq!(
            tokenize("(){};,.-+/*"),
            vec!(
                LeftParen, RightParen, LeftBrace, RightBrace, Semicolon, Comma, Dot, Minus, Plus,
                Slash, Star
            )
        );
    }

    #[test]
    fn possible_double_tokens() {
        assert_eq!(
            tokenize("!=!.===.<=<.>=>"),
            vec!(
                BangEqual,
                Bang,
                Dot,
                EqualEqual,
                Equal,
                Dot,
                LessEqual,
                Less,
                Dot,
                GreaterEqual,
                Greater
            )
        );
    }

    #[test]
    fn handles_whitespace_1() {
        assert_eq!(tokenize("  ()"), vec!(LeftParen, RightParen));
    }

    #[test]
    fn handles_whitespace_2() {
        assert_eq!(
            tokenize("!= =       =="),
            vec!(BangEqual, Equal, EqualEqual)
        );
    }

    #[test]
    fn handles_whitespace_3() {
        assert_eq!(tokenize("====      "), vec!(EqualEqual, EqualEqual));
    }

    #[test]
    fn handles_comments_1() {
        assert_eq!(tokenize("// ok this is a comment"), vec!());
    }

    #[test]
    fn handles_comments_2() {
        assert_eq!(
            tokenize("// ok this is a comment \n    /()*"),
            vec!(Slash, LeftParen, RightParen, Star)
        );
    }

    #[test]
    fn handles_comments_3() {
        assert_eq!(tokenize("// ok this is a comment \n!"), vec!(Bang));
    }

    #[test]
    fn handles_newlines() {
        let mut t = Tokenizer::new("*\n!\n.");
        assert_eq!(t.next(), Some(Token::new(Star, "*", 0, 0)));
        assert_eq!(t.next(), Some(Token::new(Bang, "!", 2, 1)));
        assert_eq!(t.next(), Some(Token::new(Dot, ".", 4, 2)));
        assert_eq!(t.line(), 2);
    }

    #[test]
    fn handles_strings() {
        let mut t = Tokenizer::new("\"Hello world!\"");
        assert_eq!(t.next(), Some(Token::new(String, "\"Hello world!\"", 0, 0)));
    }

    #[test]
    fn handles_strings_() {
        let mut t = Tokenizer::new("!= \"Hello world!\"");
        assert_eq!(t.next(), Some(Token::new(BangEqual, "!=", 0, 0)));
        assert_eq!(t.next(), Some(Token::new(String, "\"Hello world!\"", 3, 0)));
    }

    #[test]
    fn handles_unterminated_strings() {
        // @TODO this should terminate with error
        let mut t = Tokenizer::new("\"Hello world!");
        assert_eq!(t.next(), None);
    }

    #[test]
    fn handles_numbers() {
        let mut t = Tokenizer::new("1009");
        assert_eq!(t.next(), Some(Token::new(Number, "1009", 0, 0)));
    }

    #[test]
    fn handles_numbers_2() {
        let mut t = Tokenizer::new("1");
        assert_eq!(t.next(), Some(Token::new(Number, "1", 0, 0)));
    }

    #[test]
    fn handles_numbers_3() {
        let mut t = Tokenizer::new("!1");
        assert_eq!(t.next(), Some(Token::new(Bang, "!", 0, 0)));
        assert_eq!(t.next(), Some(Token::new(Number, "1", 1, 0)));
    }

    #[test]
    fn handles_identifiers() {
        let mut t = Tokenizer::new("it _it it5");
        assert_eq!(t.next(), Some(Token::new(Identifier, "it", 0, 0)));
        assert_eq!(t.next(), Some(Token::new(Identifier, "_it", 3, 0)));
        assert_eq!(t.next(), Some(Token::new(Identifier, "it5", 7, 0)));
    }

    #[test]
    fn handles_keyword_and() {
        let mut t = Tokenizer::new("and ! and! !and andand");
        assert_eq!(t.next(), Some(Token::new(And, "and", 0, 0)));
        assert_eq!(t.next(), Some(Token::new(Bang, "!", 4, 0)));
        assert_eq!(t.next(), Some(Token::new(And, "and", 6, 0)));
        assert_eq!(t.next(), Some(Token::new(Bang, "!", 9, 0)));
        assert_eq!(t.next(), Some(Token::new(Bang, "!", 11, 0)));
        assert_eq!(t.next(), Some(Token::new(And, "and", 12, 0)));
        assert_eq!(t.next(), Some(Token::new(Identifier, "andand", 16, 0)));
    }

    #[test]
    fn handles_keyword_class() {
        let mut t = Tokenizer::new("class classes");
        assert_eq!(t.next(), Some(Token::new(Class, "class", 0, 0)));
        assert_eq!(t.next(), Some(Token::new(Identifier, "classes", 6, 0)));
    }

    #[test]
    fn handles_keyword_else() {
        let mut t = Tokenizer::new("else elsen");
        assert_eq!(t.next(), Some(Token::new(Else, "else", 0, 0)));
        assert_eq!(t.next(), Some(Token::new(Identifier, "elsen", 5, 0)));
    }

    #[test]
    fn handles_keyword_if() {
        let mut t = Tokenizer::new("if iff");
        assert_eq!(t.next(), Some(Token::new(If, "if", 0, 0)));
        assert_eq!(t.next(), Some(Token::new(Identifier, "iff", 3, 0)));
    }

    #[test]
    fn handles_keyword_nil() {
        let mut t = Tokenizer::new("nil nill");
        assert_eq!(t.next(), Some(Token::new(Nil, "nil", 0, 0)));
        assert_eq!(t.next(), Some(Token::new(Identifier, "nill", 4, 0)));
    }

    #[test]
    fn handles_keyword_or() {
        let mut t = Tokenizer::new("or ors");
        assert_eq!(t.next(), Some(Token::new(Or, "or", 0, 0)));
        assert_eq!(t.next(), Some(Token::new(Identifier, "ors", 3, 0)));
    }

    #[test]
    fn handles_keyword_print() {
        let mut t = Tokenizer::new("print prints");
        assert_eq!(t.next(), Some(Token::new(Print, "print", 0, 0)));
        assert_eq!(t.next(), Some(Token::new(Identifier, "prints", 6, 0)));
    }

    #[test]
    fn handles_keyword_return() {
        let mut t = Tokenizer::new("return returns");
        assert_eq!(t.next(), Some(Token::new(Return, "return", 0, 0)));
        assert_eq!(t.next(), Some(Token::new(Identifier, "returns", 7, 0)));
    }

    #[test]
    fn handles_keyword_super() {
        let mut t = Tokenizer::new("super supers");
        assert_eq!(t.next(), Some(Token::new(Super, "super", 0, 0)));
        assert_eq!(t.next(), Some(Token::new(Identifier, "supers", 6, 0)));
    }

    #[test]
    fn handles_keyword_var() {
        let mut t = Tokenizer::new("var vars");
        assert_eq!(t.next(), Some(Token::new(Var, "var", 0, 0)));
        assert_eq!(t.next(), Some(Token::new(Identifier, "vars", 4, 0)));
    }

    #[test]
    fn handles_keyword_while() {
        let mut t = Tokenizer::new("while whiles");
        assert_eq!(t.next(), Some(Token::new(While, "while", 0, 0)));
        assert_eq!(t.next(), Some(Token::new(Identifier, "whiles", 6, 0)));
    }

    #[test]
    fn handles_keyword_false() {
        let mut t = Tokenizer::new("false falses");
        assert_eq!(t.next(), Some(Token::new(False, "false", 0, 0)));
        assert_eq!(t.next(), Some(Token::new(Identifier, "falses", 6, 0)));
    }

    #[test]
    fn handles_keyword_for() {
        let mut t = Tokenizer::new("for fore");
        assert_eq!(t.next(), Some(Token::new(For, "for", 0, 0)));
        assert_eq!(t.next(), Some(Token::new(Identifier, "fore", 4, 0)));
    }

    #[test]
    fn handles_keyword_fun() {
        let mut t = Tokenizer::new("fun func");
        assert_eq!(t.next(), Some(Token::new(Fun, "fun", 0, 0)));
        assert_eq!(t.next(), Some(Token::new(Identifier, "func", 4, 0)));
    }

    #[test]
    fn handles_keyword_this() {
        let mut t = Tokenizer::new("this thiss");
        assert_eq!(t.next(), Some(Token::new(This, "this", 0, 0)));
        assert_eq!(t.next(), Some(Token::new(Identifier, "thiss", 5, 0)));
    }

    #[test]
    fn handles_keyword_true() {
        let mut t = Tokenizer::new("true trues");
        assert_eq!(t.next(), Some(Token::new(True, "true", 0, 0)));
        assert_eq!(t.next(), Some(Token::new(Identifier, "trues", 5, 0)));
    }
}
