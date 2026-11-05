use std::fmt::Debug;

use lexer::span::CodeSpan;

use crate::error::ElpParserError;

#[derive(Debug, PartialEq)]
pub struct Spanned<T: Debug + PartialEq> {
    pub node: T,
    pub span: CodeSpan,
}

pub enum SpanControlFlow {
    // Continue and consume.
    Continue,
    // Exit without consuming.
    Break,
    // Exit after consuming.
    BreakAfter,
}

pub fn spans_extent(spans: Vec<CodeSpan>) -> Result<CodeSpan, ElpParserError> {
    match spans.len() {
        0 => Err(ElpParserError::ScriptParsingError(
            crate::error::ParsingError::Expected("Any tokens, this is a compiler bug".into()),
        )),
        1 => Ok(spans[0].clone()),
        _ => Ok(CodeSpan {
            start: spans[0].start,
            end: spans.iter().last().unwrap().end,
            display_column: spans[0].display_column,
            line: spans[0].line,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_spans_extent_empty_errors_cleanly() {
        let err = spans_extent(vec![]).unwrap_err();
        assert!(
            format!("{err}").contains("Any tokens"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn test_spans_extent() {
        assert_eq!(
            spans_extent(vec![CodeSpan {
                start: 0,
                end: 10,
                display_column: 1,
                line: 1
            }])
            .unwrap(),
            CodeSpan {
                start: 0,
                end: 10,
                display_column: 1,
                line: 1
            }
        );
        assert_eq!(
            spans_extent(vec![
                CodeSpan {
                    start: 0,
                    end: 10,
                    display_column: 1,
                    line: 1
                },
                CodeSpan {
                    start: 11,
                    end: 20,
                    display_column: 11,
                    line: 1
                }
            ])
            .unwrap(),
            CodeSpan {
                start: 0,
                end: 20,
                display_column: 1,
                line: 1
            }
        );
    }
}
