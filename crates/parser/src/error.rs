use lexer::span::{CodeSpan, SpannedToken};
use std::{fmt::Display, path::PathBuf};

#[derive(Debug, PartialEq)]
pub enum ParsingError {
    Unexpected(Option<CodeSpan>, String),
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

#[derive(Debug, PartialEq)]
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

/// Shared by every `ast::*` parsing module so error messages stay
/// consistent instead of each module hand-rolling its own phrasing.
pub fn unexpected(token: &SpannedToken, ctx: &str) -> ElpParserError {
    ElpParserError::ScriptParsingError(ParsingError::Unexpected(
        Some(token.span.clone()),
        format!("{ctx}: '{:?}'", token.token),
    ))
}

pub fn expected_got(token: Option<&SpannedToken>, msg: &str) -> ElpParserError {
    ElpParserError::ScriptParsingError(ParsingError::ExpectedGot(
        token.map(|t| t.span.clone()),
        msg.to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn span(line: usize, column: usize) -> CodeSpan {
        CodeSpan {
            start: 0,
            end: 1,
            line,
            display_column: column,
        }
    }

    #[test]
    fn parsing_error_unexpected_with_span() {
        let err = ParsingError::Unexpected(Some(span(3, 7)), "a widget".to_string());
        assert_eq!(format!("{err}"), "unexpected a widget at line 3, column 7");
    }

    #[test]
    fn parsing_error_unexpected_without_span() {
        let err = ParsingError::Unexpected(None, "a widget".to_string());
        assert_eq!(format!("{err}"), "unexpected a widget");
    }

    #[test]
    fn parsing_error_expected() {
        let err = ParsingError::Expected("a widget".to_string());
        assert_eq!(format!("{err}"), "expected a widget");
    }

    #[test]
    fn parsing_error_expected_got_without_span() {
        let err = ParsingError::ExpectedGot(None, "a widget".to_string());
        assert_eq!(format!("{err}"), "expected a widget");
    }

    #[test]
    fn parsing_error_expected_got_with_span() {
        let err = ParsingError::ExpectedGot(Some(span(2, 9)), "a widget".to_string());
        assert_eq!(format!("{err}"), "expected a widget at line 2, column 9");
    }

    #[test]
    fn elp_parser_error_script_lexing_error() {
        let err = ElpParserError::ScriptLexingError(span(1, 1), "bad byte".to_string());
        assert_eq!(
            format!("{err}"),
            "lexer error at line 1, column 1: bad byte"
        );
    }

    #[test]
    fn elp_parser_error_script_parsing_error_wraps_and_prefixes() {
        let err =
            ElpParserError::ScriptParsingError(ParsingError::Expected("a widget".to_string()));
        assert_eq!(format!("{err}"), "parse error: expected a widget");
    }

    #[test]
    fn elp_parser_error_file_system_error_with_path() {
        let err = ElpParserError::FileSystemError(
            Some(PathBuf::from("/tmp/missing.elp")),
            "permission denied".to_string(),
        );
        assert_eq!(
            format!("{err}"),
            "filesystem error for '/tmp/missing.elp': permission denied"
        );
    }

    #[test]
    fn elp_parser_error_file_system_error_without_path() {
        let err = ElpParserError::FileSystemError(None, "glob pattern invalid".to_string());
        assert_eq!(format!("{err}"), "filesystem error: glob pattern invalid");
    }

    #[test]
    fn elp_parser_error_vec_empty() {
        let err = ElpParserError::Vec(vec![]);
        assert_eq!(format!("{err}"), "0 errors:\n");
    }

    #[test]
    fn elp_parser_error_vec_single() {
        let err = ElpParserError::Vec(vec![ElpParserError::FileSystemError(
            None,
            "oops".to_string(),
        )]);
        assert_eq!(format!("{err}"), "1 errors:\n  1: filesystem error: oops");
    }

    #[test]
    fn elp_parser_error_vec_multiple_separates_with_newlines_but_no_trailing_one() {
        // Exercises both branches of the loop: the `writeln!` after every
        // error except the last, and the last one *not* getting one.
        let err = ElpParserError::Vec(vec![
            ElpParserError::FileSystemError(None, "first".to_string()),
            ElpParserError::FileSystemError(None, "second".to_string()),
            ElpParserError::FileSystemError(None, "third".to_string()),
        ]);
        assert_eq!(
            format!("{err}"),
            "3 errors:\n  1: filesystem error: first\n  2: filesystem error: second\n  3: filesystem error: third"
        );
    }

    #[test]
    fn source_is_some_for_script_parsing_error_and_none_otherwise() {
        use std::error::Error;

        let wrapped = ElpParserError::ScriptParsingError(ParsingError::Expected("x".to_string()));
        assert!(wrapped.source().is_some());

        let unwrapped = ElpParserError::FileSystemError(None, "x".to_string());
        assert!(unwrapped.source().is_none());
    }

    #[test]
    fn unexpected_helper_formats_the_token_debug_repr() {
        use lexer::{span::SpannedToken, tokens::LexerTokens};

        let token = SpannedToken {
            span: span(4, 2),
            token: LexerTokens::Identifier("foo".to_string()),
        };
        let err = unexpected(&token, "unexpected thing");
        assert_eq!(
            format!("{err}"),
            "parse error: unexpected unexpected thing: 'Identifier(\"foo\")' at line 4, column 2"
        );
    }

    #[test]
    fn expected_got_helper_with_and_without_a_token() {
        use lexer::{span::SpannedToken, tokens::LexerTokens};

        let with_token = expected_got(None, "a widget");
        assert_eq!(format!("{with_token}"), "parse error: expected a widget");

        let token = SpannedToken {
            span: span(5, 3),
            token: LexerTokens::Identifier("bar".to_string()),
        };
        let another = expected_got(Some(&token), "a widget");
        assert_eq!(
            format!("{another}"),
            "parse error: expected a widget at line 5, column 3"
        );
    }
}
