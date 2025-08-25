use crate::tokens::LexerTokens;

#[derive(Debug, PartialEq, Clone)]
pub struct CodeSpan {
    pub start: usize,
    pub end: usize,
    pub display_column: usize,
    // @TODO: Editors and other much more mature compiler products track
    // the width of graphemes which I don't do right now because I'm
    // lazy/not that smart.
    // pub grapheme_column: usize
    pub line: usize,
}

#[derive(Debug, PartialEq)]
pub struct SpannedToken {
    pub span: CodeSpan,
    pub token: LexerTokens,
}
