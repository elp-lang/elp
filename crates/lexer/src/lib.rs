use span::CodeSpan;

use crate::{span::SpannedToken, tokens::LexerTokens};

pub mod span;
pub mod tokens;

pub struct ElpLexer {
    input: String,
    cursor: usize,
    line: usize,
}

impl ElpLexer {
    pub fn new(input: String) -> Self {
        Self {
            input,
            cursor: 0,
            line: 1,
        }
    }

    fn is_at_end(&self) -> bool {
        self.cursor >= self.input.len()
    }

    fn consume(&mut self) {
        self.cursor += 1;
    }

    fn peek_next(&self) -> char {
        if self.cursor + 1 >= self.input.len() {
            return '\0';
        }
        self.input.chars().nth(self.cursor + 1).unwrap()
    }

    fn consume_identifier(&mut self) -> SpannedToken {
        let mut token = SpannedToken {
            span: CodeSpan {
                start: self.cursor,
                end: self.cursor,
                source: "".into(),
            },
            token: LexerTokens::Identifier("".into()),
        };
        let mut value: String = "".into();

        loop {
            if self.is_at_end() {
                break;
            }
            let char = self.input.chars().nth(self.cursor).unwrap();
            if self.is_at_end() || char.is_whitespace() || (!char.is_alphanumeric() && char != '_')
            {
                dbg!(self.is_at_end(), char);
                self.consume();
                break;
            }

            self.consume();
            value.push(char);
        }
        token.span.end = self.cursor;
        token.span.source = value.clone();
        token.token = LexerTokens::Identifier(value);

        token
    }

    fn consume_numeric(&mut self) -> SpannedToken {
        let mut token = SpannedToken {
            span: CodeSpan {
                start: self.cursor,
                end: self.cursor,
                source: "".into(),
            },
            token: LexerTokens::Identifier("".into()),
        };
        let mut value: String = "".into();

        loop {
            if self.is_at_end() {
                break;
            }
            let char = self.input.chars().nth(self.cursor).unwrap();
            if self.is_at_end()
                || char.is_whitespace()
                || (!char.is_numeric() && char != '_' && char != '.')
            {
                break;
            }

            self.consume();
            value.push(char);
        }
        token.span.end = self.cursor;
        token.span.source = value.clone();
        token.token = LexerTokens::Number(value);

        token
    }

    fn consume_symbols(&mut self) -> SpannedToken {
        let mut token = SpannedToken {
            span: CodeSpan {
                start: self.cursor,
                end: self.cursor,
                source: "".into(),
            },
            token: LexerTokens::Identifier("".into()),
        };
        let mut value: String = "".into();

        loop {
            let char = self.peek_next();
            if self.is_at_end() || char.is_whitespace() || char.is_alphanumeric() {
                break;
            }

            self.consume();
            value.push(char);
        }

        token.span.end = self.cursor;
        token.token = LexerTokens::Symbol(value);

        token
    }

    fn scan(&mut self) -> Vec<SpannedToken> {
        let mut tokens = vec![SpannedToken {
            span: CodeSpan {
                start: 0,
                end: 0,
                source: "".into(),
            },
            token: LexerTokens::SOI,
        }];

        while !self.is_at_end() {
            let char = self.input.chars().nth(self.cursor).unwrap();

            if char == '\0' {
                break;
            }

            if char.is_whitespace() {
                if char == 0xA as char {
                    self.line += 1;
                }
                self.consume();
                continue;
            } else if char.is_alphabetic() || char == '_' {
                tokens.push(self.consume_identifier());
            } else if char.is_numeric() {
                tokens.push(self.consume_numeric());
            } else {
                tokens.push(self.consume_symbols());
            }
        }

        tokens
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_new() {
        let input = String::from("hello");
        let lexer = ElpLexer::new(input);
        assert_eq!(lexer.input, "hello");
    }

    #[test]
    fn test_is_at_end() {
        let input = String::from("hello");
        let mut lexer = ElpLexer::new(input);
        assert!(!lexer.is_at_end());
        while !lexer.is_at_end() {
            lexer.consume();
        }
        assert!(lexer.is_at_end());
    }

    #[test]
    fn test_scan_identifier() {
        let input = String::from("hello");
        let mut lexer = ElpLexer::new(input);
        let tokens = lexer.scan();
        assert_eq!(
            vec![
                SpannedToken {
                    span: CodeSpan {
                        start: 0,
                        end: 0,
                        source: "".into()
                    },
                    token: LexerTokens::SOI
                },
                SpannedToken {
                    span: CodeSpan {
                        start: 0,
                        end: 5,
                        source: "hello".into()
                    },
                    token: LexerTokens::Identifier("hello".into())
                }
            ],
            tokens
        )
    }

    #[test]
    fn test_scan_numeric() {
        let input_1 = String::from("1");
        let mut lexer_1 = ElpLexer::new(input_1);
        let tokens_1 = lexer_1.scan();
        assert_eq!(
            vec![
                SpannedToken {
                    span: CodeSpan {
                        start: 0,
                        end: 0,
                        source: "".into()
                    },
                    token: LexerTokens::SOI
                },
                SpannedToken {
                    span: CodeSpan {
                        start: 0,
                        end: 1,
                        source: "1".into()
                    },
                    token: LexerTokens::Number("1".into())
                }
            ],
            tokens_1
        );

        let input_123 = String::from("123");
        let mut lexer_123 = ElpLexer::new(input_123);
        let tokens_123 = lexer_123.scan();
        assert_eq!(
            vec![
                SpannedToken {
                    span: CodeSpan {
                        start: 0,
                        end: 0,
                        source: "".into()
                    },
                    token: LexerTokens::SOI
                },
                SpannedToken {
                    span: CodeSpan {
                        start: 0,
                        end: 3,
                        source: "123".into()
                    },
                    token: LexerTokens::Number("123".into())
                }
            ],
            tokens_123
        );

        let input_float = String::from("123.123");
        let mut lexer_float = ElpLexer::new(input_float);
        let tokens_float = lexer_float.scan();
        assert_eq!(
            vec![
                SpannedToken {
                    span: CodeSpan {
                        start: 0,
                        end: 0,
                        source: "".into()
                    },
                    token: LexerTokens::SOI
                },
                SpannedToken {
                    span: CodeSpan {
                        start: 0,
                        end: 7,
                        source: "123.123".into()
                    },
                    token: LexerTokens::Number("123.123".into())
                }
            ],
            tokens_float
        );

        let input_formatted = String::from("123_123");
        let mut lexer_formatted = ElpLexer::new(input_formatted);
        let tokens_formatted = lexer_formatted.scan();
        assert_eq!(
            vec![
                SpannedToken {
                    span: CodeSpan {
                        start: 0,
                        end: 0,
                        source: "".into()
                    },
                    token: LexerTokens::SOI
                },
                SpannedToken {
                    span: CodeSpan {
                        start: 0,
                        end: 7,
                        source: "123_123".into()
                    },
                    token: LexerTokens::Number("123_123".into())
                }
            ],
            tokens_formatted
        )
    }
}
