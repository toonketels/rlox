use std::cmp::PartialEq;
use std::str::CharIndices;

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

#[derive(PartialEq, Debug)]
pub struct Token<'a> {
    kind: TokenKind,
    source: &'a str,
    line: usize,
}

impl<'a> Token<'a> {
    pub fn new(kind: TokenKind, source: &'a str, line: usize) -> Self {
        Self { kind, source, line }
    }
}

#[derive(Debug)]
pub struct Tokenizer<'a> {
    line: usize,
    source: &'a str,
    chars: CharIndices<'a>,
    tokens: Vec<Token<'a>>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            chars: source.char_indices(),
            tokens: Vec::new(),
            line: 0,
        }
    }

    pub fn tokenize(&mut self) -> &Vec<Token<'a>> {
        while let Some(token) = self.scan() {
            let advance = token.source.len();
            self.tokens.push(token);
            self.source = &self.source[advance..];
        }

        println!("Done line: {}", self.line);
        &self.tokens
    }

    fn scan(&self) -> Option<Token<'a>> {
        use TokenKind::*;

        macro_rules! make_single_char_token {
            ($token:ident) => {
                Token::new($token, &self.source[0..1], self.line)
            };
        }

        macro_rules! make_double_char_token {
            ($token:ident) => {
                Token::new($token, &self.source[0..2], self.line)
            };
        }

        let mut chars = self.source.chars().peekable();
        match chars.next()? {
            '(' => Some(make_single_char_token!(LeftParen)),
            ')' => Some(make_single_char_token!(RightParen)),
            '{' => Some(make_single_char_token!(LeftBrace)),
            '}' => Some(make_single_char_token!(RightBrace)),
            ';' => Some(make_single_char_token!(Semicolon)),
            ',' => Some(make_single_char_token!(Comma)),
            '.' => Some(make_single_char_token!(Dot)),
            '-' => Some(make_single_char_token!(Minus)),
            '+' => Some(make_single_char_token!(Plus)),
            '/' => Some(make_single_char_token!(Slash)),
            '*' => Some(make_single_char_token!(Star)),

            '!' => match chars.peek().copied() {
                Some('=') => Some(make_double_char_token!(BangEqual)),
                _ => Some(make_single_char_token!(Bang)),
            },
            '=' => match chars.peek().copied() {
                Some('=') => Some(make_double_char_token!(EqualEqual)),
                _ => Some(make_single_char_token!(Equal)),
            },
            '<' => match chars.peek().copied() {
                Some('=') => Some(make_double_char_token!(LessEqual)),
                _ => Some(make_single_char_token!(Less)),
            },
            '>' => match chars.peek().copied() {
                Some('=') => Some(make_double_char_token!(GreaterEqual)),
                _ => Some(make_single_char_token!(Greater)),
            },
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::TokenKind::*;

    #[test]
    fn single_tokens() {
        let source = "(){};,.-+/*";

        let mut tokenizer = Tokenizer::new(source);

        println!("{:?}", tokenizer);
        println!("{:?}", tokenizer.tokenize());
        println!("{:?}", tokenizer.tokenize());

        let r = tokenizer
            .tokenize()
            .iter()
            .map(|it| it.kind)
            .collect::<Vec<_>>();

        assert_eq!(
            r,
            vec!(
                LeftParen, RightParen, LeftBrace, RightBrace, Semicolon, Comma, Dot, Minus, Plus,
                Slash, Star
            )
        );
    }

    #[test]
    fn possible_double_tokens() {
        let source = "!=!.===.<=<.>=>";

        let mut tokenizer = Tokenizer::new(source);

        println!("{:?}", tokenizer);
        println!("{:?}", tokenizer.tokenize());
        println!("{:?}", tokenizer.tokenize());

        let r = tokenizer
            .tokenize()
            .iter()
            .map(|it| it.kind)
            .collect::<Vec<_>>();

        assert_eq!(
            r,
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
}
