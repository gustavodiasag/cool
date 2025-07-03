use std::iter::Peekable;

use crate::token::{Token, TokenKind};

pub const SUGGESTED_CAPACITY: usize = 8_192;

pub fn lex(src: &str) -> Vec<Token> {
    Lexer::new(src).lex()
}

pub struct Lexer<'a> {
    src: &'a str,
    iter: Peekable<std::str::Chars<'a>>,
    pos: usize,
    cur: usize,
}

impl Lexer<'_> {
    pub fn new(src: &str) -> Lexer {
        Lexer {
            src,
            iter: src.chars().peekable(),
            pos: 0,
            cur: 0,
        }
    }
}

impl Lexer<'_> {
    pub fn lex(&mut self) -> Vec<Token> {
        let mut tokens = Vec::with_capacity(SUGGESTED_CAPACITY);

        loop {
            let kind = self.scan();
            let eof = kind.is_eof();
            let tok = self.tokenize(kind);
            tokens.push(tok);

            if eof {
                break;
            }
        }

        tokens
    }

    fn scan(&mut self) -> TokenKind {
        use TokenKind::*;
        match self.mark_advance() {
            '\0' => Eof,
            '+' => Plus,
            '-' => match self.peek() {
                '-' => self.inline_comment(),
                _ => Minus,
            },
            '*' => Star,
            '/' => Slash,
            '!' => Not,
            '~' => Tilde,
            '=' => match self.peek() {
                '>' => self.
            }
        }
    }

    fn inline_comment(&mut self) -> TokenKind {
        assert_eq!(self.advance(), '-');
        while !matches!(self.peek(), '\n' | '\0') {
            self.advance();
        }
        TokenKind::InlineComment
    }
}

impl Lexer<'_> {
    fn mark_advance(&mut self) -> char {
        self.pos = self.cur;
        self.advance()
    }

    fn advance(&mut self) -> char {
        self.iter
            .next()
            .inspect(|c| self.cur += c.len_utf8())
            .unwrap_or('\n')
    }

    fn peek(&mut self) -> char {
        self.iter.peek().copied().unwrap_or('\0')
    }

    fn tokenize(&mut self, kind: TokenKind) -> Token {
        Token::new(kind, self.pos, self.cur - self.pos)
    }
}
