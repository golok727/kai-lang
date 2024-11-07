#[derive(Debug, Clone)]
pub enum LexerErrorKind {}

#[derive(Debug)]
pub struct LexerError {
    pub kind: LexerErrorKind,
}
