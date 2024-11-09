use crate::ast::span::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LexerErrorKind {
    NumberTrailingUnderScore,
    BadExponent,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct LexerError {
    pub kind: LexerErrorKind,
    pub location: Span,
}
