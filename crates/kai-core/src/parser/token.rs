use std::fmt::Display;

use ecow::EcoString;
#[derive(Clone, Debug, PartialOrd, PartialEq)]
pub enum Token {
    Let,
    Name(EcoString),
    DiscardName(EcoString),
    Mut,
    Eq,
    SemiColon,
    Eof,
    Return,
    Unknown,
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Token::Let => "let",
            Token::Mut => "mut",
            Token::Return => "return",

            Token::Eq => "=",
            Token::SemiColon => ";",

            Token::Eof => "EOF",
            Token::Unknown => "Unknown",
            Token::Name(name) => name.as_str(),
            Token::DiscardName(name) => name.as_str(),
        };
        write!(f, "{s}")
    }
}
