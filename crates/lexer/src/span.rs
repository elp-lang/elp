use crate::tokens::LexerTokens;

pub struct CodeSpan {
    pub start: usize,
    pub end: usize,
    pub source: String,
}

pub struct SpannedToken {
    pub span: CodeSpan,
    pub token: LexerTokens,
}
