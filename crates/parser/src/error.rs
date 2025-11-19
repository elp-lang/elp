use lexer::span::CodeSpan;
use std::path::PathBuf;

#[derive(Debug)]
pub enum ElpParserError {
    ScriptLexingError(CodeSpan, String),
    ScriptParsingError(Option<CodeSpan>, String),
    FileSystemError(Option<PathBuf>, String),
    Vec(Vec<Self>),
}
