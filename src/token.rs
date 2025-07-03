use std::fmt;

// This stucture is currently heavily unoptimized.
pub struct Token {
    pos: usize,
    len: usize,
    pub kind: TokenKind,
}

impl Token {
    pub fn new(kind: TokenKind, pos: usize, len: usize) -> Self {
        Self { pos, len, kind }
    }
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Token {:?}", self.kind)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TokenKind {
    Class,
    Else,
    If,
    Fi,
    In,
    Inherits,
    IsVoid,
    Let,
    While,
    Loop,
    Pool,
    Then,
    Case,
    Esac,
    New,
    Of,
    Not,
    True,
    False,
    Plus,
    Minus,
    Star,
    Slash,
    Tilde,
    Eq,
    Less,
    LessEq,
    GreaterEq,
    Greater,
    Assign,
    DoubleArrow,
    Colon,
    Semicolon,
    Comma,
    Dot,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    At,
    Identifer,
    String,
    EscapedString,
    Number,
    InlineComment,
    MultiComment,
    Whitespace,
    Eof,
}

impl TokenKind {
    pub fn is_eof(&self) -> bool {
        *self == TokenKind::Eof
    }
}

pub static KEYWORDS: phf::Map<&'static str, TokenKind> = phf::phf_map! {
    "class" => TokenKind::Class,
    "else" => TokenKind::Else,
    "false" => TokenKind::False,
    "fi" => TokenKind::Fi,
    "if" => TokenKind::If,
    "in" => TokenKind::In,
    "inherits" => TokenKind::Inherits,
    "isvoid" => TokenKind::IsVoid,
    "let" => TokenKind::Let,
    "loop" => TokenKind::Loop,
    "pool" => TokenKind::Pool,
    "then" => TokenKind::Then,
    "while" => TokenKind::While,
    "case" => TokenKind::Case,
    "esac" => TokenKind::Esac,
    "new" => TokenKind::New,
    "of" => TokenKind::Of,
    "not" => TokenKind::Not,
    "true" => TokenKind::True,
};
