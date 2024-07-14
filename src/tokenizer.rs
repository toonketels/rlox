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

#[derive(Debug)]
struct IndexedToken<'a> {
    token: Token<'a>,
    index: usize,
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
        while let Some(IndexedToken { token, index }) = self.scan() {
            let advance = index + token.source.len();
            self.tokens.push(token);
            self.source = &self.source[advance..];
        }

        println!("Done line: {}", self.line);
        &self.tokens
    }

    fn scan(&self) -> Option<IndexedToken<'a>> {
        use TokenKind::*;

        macro_rules! make_single_char_token {
            ($token:ident, $index:tt) => {
                IndexedToken {
                    token: Token::new($token, &self.source[$index..$index + 1], self.line),
                    index: $index,
                }
            };
        }

        macro_rules! make_double_char_token {
            ($token:ident, $index:tt) => {
                IndexedToken {
                    token: Token::new($token, &self.source[$index..$index + 2], self.line),
                    index: $index,
                }
            };
        }

        let mut chars = self
            .source
            .char_indices()
            .skip_while(|(_, it)| it.is_whitespace())
            .peekable();
        match chars.next()? {
            (i, '(') => Some(make_single_char_token!(LeftParen, i)),
            (i, ')') => Some(make_single_char_token!(RightParen, i)),
            (i, '{') => Some(make_single_char_token!(LeftBrace, i)),
            (i, '}') => Some(make_single_char_token!(RightBrace, i)),
            (i, ';') => Some(make_single_char_token!(Semicolon, i)),
            (i, ',') => Some(make_single_char_token!(Comma, i)),
            (i, '.') => Some(make_single_char_token!(Dot, i)),
            (i, '-') => Some(make_single_char_token!(Minus, i)),
            (i, '+') => Some(make_single_char_token!(Plus, i)),
            (i, '/') => Some(make_single_char_token!(Slash, i)),
            (i, '*') => Some(make_single_char_token!(Star, i)),

            (i, '!') => match chars.peek().copied() {
                Some((_, '=')) => Some(make_double_char_token!(BangEqual, i)),
                _ => Some(make_single_char_token!(Bang, i)),
            },
            (i, '=') => match chars.peek().copied() {
                Some((_, '=')) => Some(make_double_char_token!(EqualEqual, i)),
                _ => Some(make_single_char_token!(Equal, i)),
            },
            (i, '<') => match chars.peek().copied() {
                Some((_, '=')) => Some(make_double_char_token!(LessEqual, i)),
                _ => Some(make_single_char_token!(Less, i)),
            },
            (i, '>') => match chars.peek().copied() {
                Some((_, '=')) => Some(make_double_char_token!(GreaterEqual, i)),
                _ => Some(make_single_char_token!(Greater, i)),
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

    #[test]
    fn handles_whitespace_1() {
        let source = "  ()";

        let mut tokenizer = Tokenizer::new(source);

        let r = tokenizer
            .tokenize()
            .iter()
            .map(|it| it.kind)
            .collect::<Vec<_>>();

        assert_eq!(r, vec!(LeftParen, RightParen));
    }

    #[test]
    fn handles_whitespace_2() {
        let source = "!= =       ==";

        let mut tokenizer = Tokenizer::new(source);

        let r = tokenizer
            .tokenize()
            .iter()
            .map(|it| it.kind)
            .collect::<Vec<_>>();

        assert_eq!(r, vec!(BangEqual, Equal, EqualEqual));
    }

    #[test]
    fn handles_whitespace_3() {
        let source = "====      ";

        let mut tokenizer = Tokenizer::new(source);

        let r = tokenizer
            .tokenize()
            .iter()
            .map(|it| it.kind)
            .collect::<Vec<_>>();

        assert_eq!(r, vec!(EqualEqual, EqualEqual));
    }
}
