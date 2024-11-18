use std::fmt::Display;

use ecow::EcoString;

#[derive(Clone, Debug, PartialOrd, PartialEq)]
pub enum TokenKind {
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
    PlusEq,
    MinusEq,
    MulEq,
    DivEq,

    Comment,
    DocComment { comment: EcoString },

    LParen,
    RParen,
    LBracket,
    RBracket,
    LCurly,
    RCurly,

    ArrowRight,

    Gt,
    Lt,
    LtEq,
    GtEq,

    At,
    Dot,
    DotDot,
    DotDotEq,
    DotDotDot,
    Eq,
    EqEq,
    Bang,
    NotEq,
    SemiColon,
    Colon,
    Unknown,

    NewLine,
    Eof,
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TokenKind::As => "as",
            TokenKind::Class => "class",
            TokenKind::Let => "let",
            TokenKind::Mut => "mut",
            TokenKind::Panic => "panic",
            TokenKind::Pub => "pub",
            TokenKind::Return => "return",
            TokenKind::ClassSelf => "self",
            TokenKind::Using => "using",
            TokenKind::If => "if",
            TokenKind::Else => "else",
            TokenKind::Todo => "todo",
            TokenKind::Fn => "fn",
            TokenKind::For => "for",
            TokenKind::In => "in",
            TokenKind::Loop => "loop",

            TokenKind::Gt => ">",
            TokenKind::Lt => "<",
            TokenKind::LtEq => "<=",
            TokenKind::GtEq => ">=",

            TokenKind::LParen => "(",
            TokenKind::RParen => ")",
            TokenKind::LBracket => "[",
            TokenKind::RBracket => "]",
            TokenKind::LCurly => "{",
            TokenKind::RCurly => "}",

            TokenKind::ArrowRight => "->",

            TokenKind::At => "@",
            TokenKind::Dot => ".",
            TokenKind::DotDot => "..",
            TokenKind::DotDotEq => "..=",
            TokenKind::DotDotDot => "...",
            TokenKind::Bang => "!",
            TokenKind::NotEq => "!=",
            TokenKind::Eq => "=",
            TokenKind::EqEq => "==",
            TokenKind::SemiColon => ";",
            TokenKind::Colon => ":",

            TokenKind::Comment => "// ... comment",
            TokenKind::DocComment { .. } => "Doc comment",

            TokenKind::Name { name } | TokenKind::DiscardName { name } => name.as_str(),
            TokenKind::Int { value, .. } => &format!("int({})", value),
            TokenKind::Float { value } => &format!("float({})", value),
            TokenKind::String { value } => value.as_str(),

            TokenKind::NewLine => "NewLine",
            TokenKind::Eof => "EOF",

            TokenKind::Plus => "+",
            TokenKind::Minus => "-",
            TokenKind::Star => "*",
            TokenKind::Slash => "/",

            TokenKind::PlusEq => "+=",
            TokenKind::MinusEq => "-=",
            TokenKind::MulEq => "*=",
            TokenKind::DivEq => "/=",

            TokenKind::Unknown => "Unknown",
        };

        write!(f, "{s}")
    }
}
