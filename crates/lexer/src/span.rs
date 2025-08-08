use crate::tokens::LexerTokens;

#[derive(Debug, PartialEq, Clone)]
pub struct CodeSpan {
    pub start: usize,
    pub end: usize,
    pub column: usize,
    pub line: usize,
}

#[derive(Debug, PartialEq)]
pub struct SpannedToken {
    pub span: CodeSpan,
    pub token: LexerTokens,
}
