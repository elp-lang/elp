use lexer::span::CodeSpan;
use std::path::PathBuf;

#[derive(Debug)]
pub enum ParsingError {
    Unexpected(Option<CodeSpan>, String),
    Expected(String),
}

#[derive(Debug)]
pub enum ElpParserError {
    ScriptLexingError(CodeSpan, String),
    ScriptParsingError(ParsingError),
    FileSystemError(Option<PathBuf>, String),
    Vec(Vec<Self>),
}
