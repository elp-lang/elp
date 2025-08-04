use crate::tokens::LexerTokens;

#[derive(Debug, PartialEq)]
pub struct CodeSpan {
    pub start: usize,
    pub end: usize,
    pub line: usize,
}

#[derive(Debug, PartialEq)]
pub struct SpannedToken {
    pub span: CodeSpan,
    pub token: LexerTokens,
}
