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
    Int { value: EcoString },
    Float { value: EcoString },

    Eq,
    SemiColon,
    Eof,
    Unknown,
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Token::Let => "let",
            Token::Mut => "mut",
            Token::Pub => "pub",
            Token::Return => "return",

            Token::Eq => "=",
            Token::SemiColon => ";",

            Token::Eof => "EOF",
            Token::Unknown => "Unknown",
            Token::Name { name } | Token::DiscardName { name } => name.as_str(),
            Token::Int { value } => &format!("int({})", value),
            Token::Float { value } => &format!("float({})", value),
        };

        write!(f, "{s}")
    }
}
