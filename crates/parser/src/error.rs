use lexer::span::{CodeSpan, SpannedToken};
use std::{fmt::Display, path::PathBuf};

#[derive(Debug)]
pub enum ParsingError {
    Unexpected(Option<CodeSpan>, String),
    UnexpectedToken(SpannedToken),
    Expected(String),
    ExpectedGot(Option<CodeSpan>, String),
}

impl std::error::Error for ParsingError {}

impl Display for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParsingError::Unexpected(Some(span), msg) => {
                write!(
                    f,
                    "unexpected {} at line {}, column {}",
                    msg, span.line, span.display_column
                )
            }
            ParsingError::UnexpectedToken(token) => {
                write!(
                    f,
                    "unexpected {} at line {}, column {}",
                    token.token, token.span.line, token.span.display_column
                )
            }
            ParsingError::Unexpected(None, msg) => {
                write!(f, "unexpected {}", msg)
            }
            ParsingError::Expected(msg) => {
                write!(f, "expected {}", msg)
            }
            ParsingError::ExpectedGot(None, msg) => {
                write!(f, "expected {}", msg)
            }
            ParsingError::ExpectedGot(Some(span), msg) => {
                write!(
                    f,
                    "expected {} at line {}, column {}",
                    msg, span.line, span.display_column
                )
            }
        }
    }
}

#[derive(Debug)]
pub enum ElpParserError {
    ScriptLexingError(CodeSpan, String),
    ScriptParsingError(ParsingError),
    FileSystemError(Option<PathBuf>, String),
    Vec(Vec<Self>),
}

impl Display for ElpParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ElpParserError::ScriptLexingError(span, msg) => {
                write!(
                    f,
                    "lexer error at line {}, column {}: {}",
                    span.line, span.display_column, msg
                )
            }
            ElpParserError::ScriptParsingError(err) => {
                write!(f, "parse error: {}", err)
            }
            ElpParserError::FileSystemError(Some(path), msg) => {
                write!(f, "filesystem error for '{}': {}", path.display(), msg)
            }
            ElpParserError::FileSystemError(None, msg) => {
                write!(f, "filesystem error: {}", msg)
            }
            ElpParserError::Vec(errors) => {
                writeln!(f, "{} errors:", errors.len())?;
                for (i, err) in errors.iter().enumerate() {
                    write!(f, "  {}: {}", i + 1, err)?;
                    if i < errors.len() - 1 {
                        writeln!(f)?;
                    }
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for ElpParserError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ElpParserError::ScriptParsingError(err) => Some(err),
            _ => None,
        }
    }
}
