use crate::{
    span::SpannedToken,
    tokens::{LexerSymbol, LexerTokens},
};
use span::CodeSpan;
use std::{iter::Peekable, str::Chars};

pub mod span;
pub mod tokens;

pub struct ElpLexer<'a> {
    chars: Peekable<Chars<'a>>,
    cursor: usize,
    line: usize,
}

impl<'a> ElpLexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().peekable(),
            cursor: 0,
            line: 1,
        }
    }

    fn peek(&mut self) -> Option<&char> {
        self.chars.peek()
    }

    fn next(&mut self) -> Option<char> {
        let next_char = self.chars.next();
        if let Some(c) = next_char {
            self.cursor += c.len_utf8();
        }
        next_char
    }

    pub fn scan(&mut self) -> Vec<SpannedToken> {
        let mut tokens = vec![SpannedToken {
            span: CodeSpan {
                start: 0,
                end: 0,
                line: 1,
            },
            token: LexerTokens::SOI,
        }];

        while let Some(c) = self.peek().cloned() {
            let start = self.cursor;

            // Handle whitespace
            if c.is_whitespace() {
                if c == 0xA as char {
                    self.line += 1;
                }
                self.next();
                continue;
            }

            // Handle identifiers and keywords
            if c.is_alphabetic() || c == '_' {
                let mut value = String::new();
                value.push(self.next().unwrap());
                while let Some(&peeked_char) = self.peek() {
                    if peeked_char.is_alphanumeric() || peeked_char == '_' {
                        self.next();
                        value.push(peeked_char);
                    } else {
                        break;
                    }
                }

                tokens.push(SpannedToken {
                    span: CodeSpan {
                        start,
                        end: self.cursor,
                        line: self.line,
                    },
                    token: LexerTokens::Identifier(value),
                });
                continue;
            }

            // Handle numbers
            if c.is_numeric() {
                let mut value = String::new();
                value.push(self.next().unwrap());
                while let Some(&peeked_char) = self.peek() {
                    if peeked_char.is_numeric() || peeked_char == '_' || peeked_char == '.' {
                        self.next();
                        value.push(peeked_char);
                    } else {
                        break;
                    }
                }

                tokens.push(SpannedToken {
                    span: CodeSpan {
                        start,
                        end: self.cursor,
                        line: self.line,
                    },
                    token: LexerTokens::Number(value),
                });
                continue;
            }

            // Handle symbols
            {
                let start = self.cursor;
                self.next();
                // Peek and match for multi-character symbols first
                if let Some(&peeked) = self.peek() {
                    if c == '=' && peeked == '=' {
                        self.next();
                        tokens.push(SpannedToken {
                            span: CodeSpan {
                                start,
                                end: self.cursor,
                                line: self.line,
                            },
                            token: LexerTokens::Symbol(LexerSymbol::EqualEqual),
                        });
                        continue;
                    }
                }

                // Then, match for single-character symbols
                match c {
                    '=' => {
                        tokens.push(SpannedToken {
                            span: CodeSpan {
                                start,
                                end: self.cursor,
                                line: self.line,
                            },
                            token: LexerTokens::Symbol(LexerSymbol::Equal),
                        });
                    }
                    '+' => {
                        tokens.push(SpannedToken {
                            span: CodeSpan {
                                start,
                                end: self.cursor,
                                line: self.line,
                            },
                            token: LexerTokens::Symbol(LexerSymbol::Plus),
                        });
                    }
                    '-' => {
                        tokens.push(SpannedToken {
                            span: CodeSpan {
                                start,
                                end: self.cursor,
                                line: self.line,
                            },
                            token: LexerTokens::Symbol(LexerSymbol::Minus),
                        });
                    }
                    '*' => {
                        tokens.push(SpannedToken {
                            span: CodeSpan {
                                start,
                                end: self.cursor,
                                line: self.line,
                            },
                            token: LexerTokens::Symbol(LexerSymbol::Multiply),
                        });
                    }
                    '/' => {
                        tokens.push(SpannedToken {
                            span: CodeSpan {
                                start,
                                end: self.cursor,
                                line: self.line,
                            },
                            token: LexerTokens::Symbol(LexerSymbol::Divide),
                        });
                    }
                    ';' => {
                        tokens.push(SpannedToken {
                            span: CodeSpan {
                                start,
                                end: self.cursor,
                                line: self.line,
                            },
                            token: LexerTokens::Symbol(LexerSymbol::SemiColon),
                        });
                    }
                    _ => {
                        // Handle an unexpected character or an error
                        tokens.push(SpannedToken {
                            span: CodeSpan {
                                start,
                                end: self.cursor,
                                line: self.line,
                            },
                            token: LexerTokens::Error(c.to_string()),
                        });
                    }
                }
            }
        }
        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    macro_rules! s_token_test {
        ($start:expr, $end:expr, $line:expr, $token:expr) => {
            SpannedToken {
                span: CodeSpan {
                    start: $start,
                    end: $end,
                    line: $line,
                },
                token: $token,
            }
        };
    }

    #[test]
    fn test_empty_input() {
        let input = "";
        let mut lexer = ElpLexer::new(input);
        let tokens = lexer.scan();
        assert_eq!(vec![s_token_test!(0, 0, 1, LexerTokens::SOI)], tokens);
    }

    #[test]
    fn test_single_identifier() {
        let input = "hello";
        let mut lexer = ElpLexer::new(input);
        let tokens = lexer.scan();
        assert_eq!(
            vec![
                s_token_test!(0, 0, 1, LexerTokens::SOI),
                s_token_test!(0, 5, 1, LexerTokens::Identifier("hello".into())),
            ],
            tokens
        );
    }

    #[test]
    fn test_identifier_with_underscore() {
        let input = "my_variable";
        let mut lexer = ElpLexer::new(input);
        let tokens = lexer.scan();
        assert_eq!(
            vec![
                s_token_test!(0, 0, 1, LexerTokens::SOI),
                s_token_test!(0, 11, 1, LexerTokens::Identifier("my_variable".into())),
            ],
            tokens
        );
    }

    #[test]
    fn test_single_numeric() {
        let input = "123";
        let mut lexer = ElpLexer::new(input);
        let tokens = lexer.scan();
        assert_eq!(
            vec![
                s_token_test!(0, 0, 1, LexerTokens::SOI),
                s_token_test!(0, 3, 1, LexerTokens::Number("123".into())),
            ],
            tokens
        );
    }

    #[test]
    fn test_float_numeric() {
        let input = "3.14159";
        let mut lexer = ElpLexer::new(input);
        let tokens = lexer.scan();
        assert_eq!(
            vec![
                s_token_test!(0, 0, 1, LexerTokens::SOI),
                s_token_test!(0, 7, 1, LexerTokens::Number("3.14159".into())),
            ],
            tokens
        );
    }

    #[test]
    fn test_formatted_numeric() {
        let input = "1_000_000";
        let mut lexer = ElpLexer::new(input);
        let tokens = lexer.scan();
        assert_eq!(
            vec![
                s_token_test!(0, 0, 1, LexerTokens::SOI),
                s_token_test!(0, 9, 1, LexerTokens::Number("1_000_000".into())),
            ],
            tokens
        );
    }

    #[test]
    fn test_symbols() {
        let input = "+-*/==";
        let mut lexer = ElpLexer::new(input);
        let tokens = lexer.scan();
        assert_eq!(
            vec![
                s_token_test!(0, 0, 1, LexerTokens::SOI),
                s_token_test!(0, 1, 1, LexerTokens::Symbol(LexerSymbol::Plus)),
                s_token_test!(1, 2, 1, LexerTokens::Symbol(LexerSymbol::Minus)),
                s_token_test!(2, 3, 1, LexerTokens::Symbol(LexerSymbol::Multiply)),
                s_token_test!(3, 4, 1, LexerTokens::Symbol(LexerSymbol::Divide)),
                s_token_test!(4, 6, 1, LexerTokens::Symbol(LexerSymbol::EqualEqual)),
            ],
            tokens
        );
    }

    #[test]
    fn test_mixed_tokens_with_whitespace() {
        let input = "let x = 10;";
        let mut lexer = ElpLexer::new(input);
        let tokens = lexer.scan();
        assert_eq!(
            vec![
                s_token_test!(0, 0, 1, LexerTokens::SOI),
                s_token_test!(0, 3, 1, LexerTokens::Identifier("let".into())),
                s_token_test!(4, 5, 1, LexerTokens::Identifier("x".into())),
                s_token_test!(6, 7, 1, LexerTokens::Symbol(LexerSymbol::Equal)),
                s_token_test!(8, 10, 1, LexerTokens::Number("10".into())),
                s_token_test!(10, 11, 1, LexerTokens::Symbol(LexerSymbol::SemiColon)),
            ],
            tokens
        );
    }

    #[test]
    fn test_mixed_tokens_no_whitespace() {
        let input = "x=10;y=20";
        let mut lexer = ElpLexer::new(input);
        let tokens = lexer.scan();
        assert_eq!(
            vec![
                s_token_test!(0, 0, 1, LexerTokens::SOI),
                s_token_test!(0, 1, 1, LexerTokens::Identifier("x".into())),
                s_token_test!(1, 2, 1, LexerTokens::Symbol(LexerSymbol::Equal)),
                s_token_test!(2, 4, 1, LexerTokens::Number("10".into())),
                s_token_test!(4, 5, 1, LexerTokens::Symbol(LexerSymbol::SemiColon)),
                s_token_test!(5, 6, 1, LexerTokens::Identifier("y".into())),
                s_token_test!(6, 7, 1, LexerTokens::Symbol(LexerSymbol::Equal)),
                s_token_test!(7, 9, 1, LexerTokens::Number("20".into())),
            ],
            tokens
        );
    }

    #[test]
    fn test_with_unicode_characters() {
        let input = "привет_мир"; // "hello_world" in Russian
        let mut lexer = ElpLexer::new(input);
        let tokens = lexer.scan();
        assert_eq!(
            vec![
                s_token_test!(0, 0, 1, LexerTokens::SOI),
                s_token_test!(0, 19, 1, LexerTokens::Identifier("привет_мир".into())),
            ],
            tokens
        );
    }

    #[test]
    fn test_complex_program_like_input() {
        let input = "let my_var = 123.45 + 50;";
        let mut lexer = ElpLexer::new(input);
        let tokens = lexer.scan();
        assert_eq!(
            vec![
                s_token_test!(0, 0, 1, LexerTokens::SOI),
                s_token_test!(0, 3, 1, LexerTokens::Identifier("let".into())),
                s_token_test!(4, 10, 1, LexerTokens::Identifier("my_var".into())),
                s_token_test!(11, 12, 1, LexerTokens::Symbol(LexerSymbol::Equal)),
                s_token_test!(13, 19, 1, LexerTokens::Number("123.45".into())),
                s_token_test!(20, 21, 1, LexerTokens::Symbol(LexerSymbol::Plus)),
                s_token_test!(22, 24, 1, LexerTokens::Number("50".into())),
                s_token_test!(24, 25, 1, LexerTokens::Symbol(LexerSymbol::SemiColon)),
            ],
            tokens
        );
    }
}
