use lexer::span::{CodeSpan, SpannedToken};

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub span: CodeSpan,
}

type ParseResult<'a, T> = Result<(T, &'a [SpannedToken]), ParseError>;

pub struct ElpParser {}

impl ElpParser {
    pub fn parse(program: String) -> Result {}
}
