use std::fmt::Display;

use ecow::EcoString;

#[derive(Clone, Debug, PartialOrd, PartialEq)]
pub enum Token {
    // keywords
    Let,
    Mut,
    Pub,
    Return,

    Name { name: EcoString },
    DiscardName { name: EcoString },
    // TODO: maybe switch to bigint
    Int { value: EcoString, int_value: i32 },
    Float { value: EcoString },

    Plus,
    Minus,
    Star,
    Slash,

    Eq,
    EqEq,
    SemiColon,
    Unknown,

    NewLine,
    Eof,
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Token::Let => "let",
            Token::Mut => "mut",
            Token::Pub => "pub",
            Token::Return => "return",

            Token::Eq => "=",
            Token::EqEq => "==",
            Token::SemiColon => ";",

            Token::Name { name } | Token::DiscardName { name } => name.as_str(),
            Token::Int { value, .. } => &format!("int({})", value),
            Token::Float { value } => &format!("float({})", value),

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
