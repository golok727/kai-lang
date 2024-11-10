use std::fmt::Display;

use ecow::EcoString;

#[derive(Clone, Debug, PartialOrd, PartialEq)]
pub enum Token {
    // keywords
    As,
    Class,
    ClassSelf,
    Let,
    Mut,
    Panic,
    Pub,
    If,
    Fn,
    For,
    In,
    Loop,
    Else,
    Return,
    Using,
    Todo,

    Name { name: EcoString },
    // _thing
    DiscardName { name: EcoString },
    // TODO: maybe switch to bigint
    Int { value: EcoString, int_value: i32 },
    Float { value: EcoString },
    // qoutes not included
    String { value: EcoString },

    Plus,
    Minus,
    Star,
    Slash,

    LParen,
    RParen,
    LBracket,
    RBracket,
    LCurly,
    RCurly,

    ArrowRight,

    At,
    Dot,
    DotDot,
    DotDotDot,
    Eq,
    EqEq,
    SemiColon,
    Colon,
    Unknown,

    NewLine,
    Eof,
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Token::As => "as",
            Token::Class => "class",
            Token::Let => "let",
            Token::Mut => "mut",
            Token::Panic => "panic",
            Token::Pub => "pub",
            Token::Return => "return",
            Token::ClassSelf => "self",
            Token::Using => "using",
            Token::If => "if",
            Token::Else => "else",
            Token::Todo => "todo",
            Token::Fn => "fn",
            Token::For => "for",
            Token::In => "in",
            Token::Loop => "loop",

            Token::LParen => "(",
            Token::RParen => ")",
            Token::LBracket => "[",
            Token::RBracket => "]",
            Token::LCurly => "{",
            Token::RCurly => "}",

            Token::ArrowRight => "->",

            Token::At => "@",
            Token::Dot => ".",
            Token::DotDot => "..",
            Token::DotDotDot => "...",
            Token::Eq => "=",
            Token::EqEq => "==",
            Token::SemiColon => ";",
            Token::Colon => ":",

            Token::Name { name } | Token::DiscardName { name } => name.as_str(),
            Token::Int { value, .. } => &format!("int({})", value),
            Token::Float { value } => &format!("float({})", value),
            Token::String { value } => value.as_str(),

            Token::NewLine => "NewLine",
            Token::Eof => "EOF",

            Token::Plus => "+",
            Token::Minus => "-",
            Token::Star => "*",
            Token::Slash => "/",

            Token::Unknown => "Unknown",
        };

        write!(f, "{s}")
    }
}
