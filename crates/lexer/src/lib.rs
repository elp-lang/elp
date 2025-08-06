use crate::{span::SpannedToken, symbol_trie::SymbolTrie, tokens::LexerTokens};
use span::CodeSpan;
use std::{iter::Peekable, str::Chars};
use unic::emoji::char::is_emoji_presentation;
use unicode_width::UnicodeWidthChar;
use unicode_xid::UnicodeXID;

pub mod span;
pub mod symbol_trie;
pub mod tokens;
pub mod trie;

pub struct ElpLexer<'a> {
    chars: Peekable<Chars<'a>>,
    cursor: usize,
    line: usize,
    column: usize,

    symbols: SymbolTrie,
}

impl<'a> ElpLexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().peekable(),
            cursor: 0,
            line: 1,
            column: 1,
            symbols: Default::default(),
        }
    }

    fn peek(&mut self) -> Option<&char> {
        self.chars.peek()
    }

    fn next(&mut self) -> Option<char> {
        let next_char = self.chars.next();
        if let Some(c) = next_char {
            self.cursor += c.len_utf8();
            let width = UnicodeWidthChar::width(c).unwrap_or(0);
            self.column += width;
        }
        next_char
    }

    fn consume_identifier(&mut self, start: usize) -> SpannedToken {
        let column = self.column;

        let mut value = String::new();
        value.push(self.next().unwrap());
        while let Some(&peeked_char) = self.peek() {
            if UnicodeXID::is_xid_continue(peeked_char) || is_emoji_presentation(peeked_char) {
                self.next();
                value.push(peeked_char);
            } else {
                break;
            }
        }

        SpannedToken {
            span: CodeSpan {
                start,
                end: self.cursor,
                line: self.line,
                column,
            },
            token: LexerTokens::Identifier(value),
        }
    }

    fn consume_newline(&mut self, first_char: char) {
        match first_char {
            '\n' => {
                self.line += 1;
                self.column = 1;
            }
            '\r' => {
                if let Some('\n') = self.peek() {
                    self.next(); // consume '\n'
                }
                self.line += 1;
                self.column = 1;
            }
            _ => {}
        }
    }

    fn consume_numbers(&mut self, start: usize) -> SpannedToken {
        let column = self.column;

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

        SpannedToken {
            span: CodeSpan {
                start,
                end: self.cursor,
                line: self.line,
                column,
            },
            token: LexerTokens::Number(value),
        }
    }

    fn consume_symbol(&mut self, start: usize) -> SpannedToken {
        let column = self.column;

        if let Some((token, len)) = self.symbols.match_longest(&mut self.chars) {
            self.cursor += len;
            self.column += len;
            SpannedToken {
                span: CodeSpan {
                    start,
                    end: self.cursor,
                    line: self.line,
                    column,
                },
                token: LexerTokens::Symbol(token),
            }
        } else {
            // TODO: All symbols should be matched to something, surely or the token type shouldnt be
            // symbol and more likely identifier.
            panic!(
                "dang it Dave, fix this symbol in the trie lookup: '{:#?}'",
                self.chars.nth(start)
            )
        }
    }

    pub fn scan(&mut self) -> Vec<SpannedToken> {
        let mut tokens = vec![SpannedToken {
            span: CodeSpan {
                start: 0,
                end: 0,
                line: 1,
                column: 1,
            },
            token: LexerTokens::SOI,
        }];

        while let Some(c) = self.peek().cloned() {
            let start = self.cursor;

            if c.is_whitespace() {
                let start_char = self.next().unwrap();
                self.consume_newline(start_char);
                continue;
            }

            if UnicodeXID::is_xid_start(c) || is_emoji_presentation(c) {
                tokens.push(self.consume_identifier(self.cursor));
                continue;
            }

            if c.is_numeric() {
                tokens.push(self.consume_numbers(start));
                continue;
            }

            tokens.push(self.consume_symbol(start));
        }

        tokens.push(SpannedToken {
            span: CodeSpan {
                start: self.cursor,
                end: self.cursor,
                line: self.line,
                column: self.column,
            },
            token: LexerTokens::EOI,
        });

        tokens
    }
}

#[cfg(test)]
mod tests {
    use crate::tokens::LexerSymbol;

    use super::*;
    use pretty_assertions::assert_eq;

    macro_rules! s_token_test {
        ($start:expr, $end:expr, $line:expr, $column:expr, $token:expr) => {
            SpannedToken {
                span: CodeSpan {
                    start: $start,
                    end: $end,
                    line: $line,
                    column: $column,
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
        assert_eq!(
            vec![
                s_token_test!(0, 0, 1, 1, LexerTokens::SOI),
                s_token_test!(0, 0, 1, 1, LexerTokens::EOI)
            ],
            tokens
        );
    }

    #[test]
    fn test_single_identifier() {
        let input = "hello";
        let mut lexer = ElpLexer::new(input);
        let tokens = lexer.scan();
        assert_eq!(
            vec![
                s_token_test!(0, 0, 1, 1, LexerTokens::SOI),
                s_token_test!(0, 5, 1, 1, LexerTokens::Identifier("hello".into())),
                s_token_test!(5, 5, 1, 6, LexerTokens::EOI)
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
                s_token_test!(0, 0, 1, 1, LexerTokens::SOI),
                s_token_test!(0, 11, 1, 1, LexerTokens::Identifier("my_variable".into())),
                s_token_test!(11, 11, 1, 12, LexerTokens::EOI)
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
                s_token_test!(0, 0, 1, 1, LexerTokens::SOI),
                s_token_test!(0, 3, 1, 1, LexerTokens::Number("123".into())),
                s_token_test!(3, 3, 1, 4, LexerTokens::EOI)
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
                s_token_test!(0, 0, 1, 1, LexerTokens::SOI),
                s_token_test!(0, 7, 1, 1, LexerTokens::Number("3.14159".into())),
                s_token_test!(7, 7, 1, 8, LexerTokens::EOI)
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
                s_token_test!(0, 0, 1, 1, LexerTokens::SOI),
                s_token_test!(0, 9, 1, 1, LexerTokens::Number("1_000_000".into())),
                s_token_test!(9, 9, 1, 10, LexerTokens::EOI)
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
                s_token_test!(0, 0, 1, 1, LexerTokens::SOI),
                s_token_test!(0, 1, 1, 1, LexerTokens::Symbol(LexerSymbol::Plus)),
                s_token_test!(1, 2, 1, 2, LexerTokens::Symbol(LexerSymbol::Minus)),
                s_token_test!(2, 3, 1, 3, LexerTokens::Symbol(LexerSymbol::Multiply)),
                s_token_test!(3, 5, 1, 4, LexerTokens::Symbol(LexerSymbol::DivideEqual)),
                s_token_test!(5, 6, 1, 6, LexerTokens::Symbol(LexerSymbol::Equal)),
                s_token_test!(6, 6, 1, 7, LexerTokens::EOI)
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
                s_token_test!(0, 0, 1, 1, LexerTokens::SOI),
                s_token_test!(0, 3, 1, 1, LexerTokens::Identifier("let".into())),
                s_token_test!(4, 5, 1, 5, LexerTokens::Identifier("x".into())),
                s_token_test!(6, 7, 1, 7, LexerTokens::Symbol(LexerSymbol::Equal)),
                s_token_test!(8, 10, 1, 9, LexerTokens::Number("10".into())),
                s_token_test!(10, 11, 1, 11, LexerTokens::Symbol(LexerSymbol::SemiColon)),
                s_token_test!(11, 11, 1, 12, LexerTokens::EOI)
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
                s_token_test!(0, 0, 1, 1, LexerTokens::SOI),
                s_token_test!(0, 1, 1, 1, LexerTokens::Identifier("x".into())),
                s_token_test!(1, 2, 1, 2, LexerTokens::Symbol(LexerSymbol::Equal)),
                s_token_test!(2, 4, 1, 3, LexerTokens::Number("10".into())),
                s_token_test!(4, 5, 1, 5, LexerTokens::Symbol(LexerSymbol::SemiColon)),
                s_token_test!(5, 6, 1, 6, LexerTokens::Identifier("y".into())),
                s_token_test!(6, 7, 1, 7, LexerTokens::Symbol(LexerSymbol::Equal)),
                s_token_test!(7, 9, 1, 8, LexerTokens::Number("20".into())),
                s_token_test!(9, 9, 1, 10, LexerTokens::EOI)
            ],
            tokens
        );
    }

    #[test]
    fn test_with_unicode_characters() {
        let input_1 = "привет_мир"; // "hello_world" in Russian
        let mut lexer_1 = ElpLexer::new(input_1);
        let tokens_1 = lexer_1.scan();
        assert_eq!(
            vec![
                s_token_test!(0, 0, 1, 1, LexerTokens::SOI),
                s_token_test!(0, 19, 1, 1, LexerTokens::Identifier("привет_мир".into())),
                s_token_test!(19, 19, 1, 11, LexerTokens::EOI)
            ],
            tokens_1
        );

        let input_2 = "🤖";
        let mut lexer_2 = ElpLexer::new(input_2);
        let tokens_2 = lexer_2.scan();
        assert_eq!(
            vec![
                s_token_test!(0, 0, 1, 1, LexerTokens::SOI),
                s_token_test!(0, 4, 1, 1, LexerTokens::Identifier("🤖".into())),
                s_token_test!(4, 4, 1, 3, LexerTokens::EOI)
            ],
            tokens_2
        );
    }

    #[test]
    fn test_complex_program_like_input() {
        let input = "var my_var = 123.45 + 50;";
        let mut lexer = ElpLexer::new(input);
        let tokens = lexer.scan();
        assert_eq!(
            vec![
                s_token_test!(0, 0, 1, 1, LexerTokens::SOI),
                s_token_test!(0, 3, 1, 1, LexerTokens::Identifier("var".into())),
                s_token_test!(4, 10, 1, 5, LexerTokens::Identifier("my_var".into())),
                s_token_test!(11, 12, 1, 12, LexerTokens::Symbol(LexerSymbol::Equal)),
                s_token_test!(13, 19, 1, 14, LexerTokens::Number("123.45".into())),
                s_token_test!(20, 21, 1, 21, LexerTokens::Symbol(LexerSymbol::Plus)),
                s_token_test!(22, 24, 1, 23, LexerTokens::Number("50".into())),
                s_token_test!(24, 25, 1, 25, LexerTokens::Symbol(LexerSymbol::SemiColon)),
                s_token_test!(25, 25, 1, 26, LexerTokens::EOI)
            ],
            tokens
        );
    }

    #[test]
    fn test_whitespace_no_newlines() {
        let input = "   \t\t   ";
        let mut lexer = ElpLexer::new(input);
        let tokens = lexer.scan();
        assert_eq!(
            vec![
                s_token_test!(0, 0, 1, 1, LexerTokens::SOI),
                s_token_test!(8, 8, 1, 7, LexerTokens::EOI)
            ],
            tokens
        );
    }

    #[test]
    fn test_single_lf_newline() {
        let input = "hello\nworld";
        let mut lexer = ElpLexer::new(input);
        let tokens = lexer.scan();
        assert_eq!(
            vec![
                s_token_test!(0, 0, 1, 1, LexerTokens::SOI),
                s_token_test!(0, 5, 1, 1, LexerTokens::Identifier("hello".into())),
                s_token_test!(6, 11, 2, 1, LexerTokens::Identifier("world".into())),
                s_token_test!(11, 11, 2, 6, LexerTokens::EOI),
            ],
            tokens
        );
    }

    #[test]
    fn test_single_cr_newline() {
        let input = "hello\rworld";
        let mut lexer = ElpLexer::new(input);
        let tokens = lexer.scan();
        assert_eq!(
            vec![
                s_token_test!(0, 0, 1, 1, LexerTokens::SOI),
                s_token_test!(0, 5, 1, 1, LexerTokens::Identifier("hello".into())),
                s_token_test!(6, 11, 2, 1, LexerTokens::Identifier("world".into())),
                s_token_test!(11, 11, 2, 6, LexerTokens::EOI),
            ],
            tokens
        );
    }

    #[test]
    fn test_single_crlf_newline() {
        let input = "hello\r\nworld";
        let mut lexer = ElpLexer::new(input);
        let tokens = lexer.scan();
        assert_eq!(
            vec![
                s_token_test!(0, 0, 1, 1, LexerTokens::SOI),
                s_token_test!(0, 5, 1, 1, LexerTokens::Identifier("hello".into())),
                s_token_test!(7, 12, 2, 1, LexerTokens::Identifier("world".into())),
                s_token_test!(12, 12, 2, 6, LexerTokens::EOI),
            ],
            tokens
        );
    }

    #[test]
    fn test_multiple_consecutive_newlines() {
        let input = "a\n\nb";
        let mut lexer = ElpLexer::new(input);
        let tokens = lexer.scan();
        assert_eq!(
            vec![
                s_token_test!(0, 0, 1, 1, LexerTokens::SOI),
                s_token_test!(0, 1, 1, 1, LexerTokens::Identifier("a".into())),
                s_token_test!(3, 4, 3, 1, LexerTokens::Identifier("b".into())),
                s_token_test!(4, 4, 3, 2, LexerTokens::EOI),
            ],
            tokens
        );
    }

    #[test]
    fn test_mixed_newlines() {
        let input = "token1\n\r\ntoken2\r\r\n\n token3";
        let mut lexer = ElpLexer::new(input);
        let tokens = lexer.scan();
        assert_eq!(
            vec![
                s_token_test!(0, 0, 1, 1, LexerTokens::SOI),
                s_token_test!(0, 6, 1, 1, LexerTokens::Identifier("token1".into())),
                s_token_test!(9, 15, 3, 1, LexerTokens::Identifier("token2".into())),
                s_token_test!(20, 26, 6, 2, LexerTokens::Identifier("token3".into())),
                s_token_test!(26, 26, 6, 8, LexerTokens::EOI),
            ],
            tokens
        );
    }
}
