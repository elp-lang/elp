use crate::tokens::LexerTokens;

#[derive(Debug, PartialEq)]
pub struct CodeSpan {
    pub start: usize,
    pub end: usize,
    pub source: String,
}

#[derive(Debug, PartialEq)]
pub struct SpannedToken {
    pub span: CodeSpan,
    pub token: LexerTokens,
}
