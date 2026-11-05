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

    // Lookahead beyond the single char `peek` gives us, without consuming.
    // `peek_at(0)` is equivalent to `peek`; `peek_at(1)` is the char after that.
    // Cloning `Peekable<Chars>` is cheap (it's just a cursor into the source str).
    fn peek_at(&self, n: usize) -> Option<char> {
        let mut iter = self.chars.clone();
        for _ in 0..n {
            iter.next()?;
        }
        iter.next()
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
                value.push(self.next().unwrap());
            } else {
                break;
            }
        }

        SpannedToken {
            span: CodeSpan {
                start,
                end: self.cursor,
                line: self.line,
                display_column: column,
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
        let mut seen_dot = false;
        value.push(self.next().unwrap());
        while let Some(&peeked_char) = self.peek() {
            if peeked_char == '.' {
                // Two dots in a row is the range operator, not a decimal
                // point, and a number can only have one decimal point —
                // leave both to the symbol trie / next number in either case.
                if seen_dot || self.peek_at(1) == Some('.') {
                    break;
                }
                seen_dot = true;
                self.next();
                value.push('.');
                continue;
            }

            if peeked_char.is_numeric() || peeked_char == '_' {
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
                display_column: column,
            },
            token: LexerTokens::Number(value),
        }
    }

    // `//` and `///` both start a line comment; three slashes marks it as a
    // doc comment. Consumes up to (not including) the terminating newline.
    fn consume_line_comment(&mut self, start: usize, is_doc: bool) -> SpannedToken {
        let column = self.column;

        for _ in 0..(if is_doc { 3 } else { 2 }) {
            self.next();
        }

        let mut value = String::new();
        while let Some(&c) = self.peek() {
            if c == '\n' || c == '\r' {
                break;
            }
            value.push(self.next().unwrap());
        }

        SpannedToken {
            span: CodeSpan {
                start,
                end: self.cursor,
                line: self.line,
                display_column: column,
            },
            token: if is_doc {
                LexerTokens::DocComment(value)
            } else {
                LexerTokens::LineComment(value)
            },
        }
    }

    // `/* ... */`, non-nesting — the first `*/` closes the comment, however
    // many `/*` opened inside it.
    // An unterminated block comment consumes to EOF rather than erroring —
    // there's no diagnostic-rendering machinery downstream yet to make an
    // error token more useful than the raw content would be.
    fn consume_block_comment(&mut self, start: usize) -> SpannedToken {
        let column = self.column;

        self.next(); // '/'
        self.next(); // '*'

        let mut value = String::new();
        while let Some(&c) = self.peek() {
            if c == '*' && self.peek_at(1) == Some('/') {
                self.next();
                self.next();
                break;
            }

            if c == '\n' || c == '\r' {
                let nl_char = self.next().unwrap();
                self.consume_newline(nl_char);
            } else {
                self.next();
            }
            value.push(c);
        }

        SpannedToken {
            span: CodeSpan {
                start,
                end: self.cursor,
                line: self.line,
                display_column: column,
            },
            token: LexerTokens::BlockComment(value),
        }
    }

    // Opaque string literal: `"` ... `"`. No interpolation yet (the design
    // is Swift/Kotlin-style `{ expr }` — see `grammar-script-notes.md` gap
    // #9, which needs chunked STRING tokens) but escapes are passed raw
    // (not interpreted) so an escaped quote doesn't terminate the literal
    // early. Doesn't span lines — there's no multi-line string syntax yet,
    // so a bare newline ends the scan the same lenient way an unterminated
    // block comment does, leaving the newline token for the caller.
    fn consume_string(&mut self, start: usize) -> SpannedToken {
        let column = self.column;

        let mut value = String::new();
        value.push(self.next().unwrap()); // opening quote

        while let Some(&c) = self.peek() {
            if c == '"' {
                value.push(self.next().unwrap());
                break;
            }

            if c == '\n' || c == '\r' {
                break;
            }

            if c == '\\' {
                value.push(self.next().unwrap());
                if self.peek().is_some() {
                    value.push(self.next().unwrap());
                }
                continue;
            }

            value.push(self.next().unwrap());
        }

        SpannedToken {
            span: CodeSpan {
                start,
                end: self.cursor,
                line: self.line,
                display_column: column,
            },
            token: LexerTokens::String(value),
        }
    }

    fn consume_symbol(&mut self, start: usize) -> SpannedToken {
        let column = self.column;

        if let Some((token, str_match)) = self.symbols.match_longest(&mut self.chars) {
            // We consumed `len` bytes from the iterator already, so update the cursor and column accordingly.
            self.cursor += str_match.len();
            self.column += str_match
                .chars()
                .map(|c| UnicodeWidthChar::width(c).unwrap_or(1))
                .sum::<usize>();

            SpannedToken {
                span: CodeSpan {
                    start,
                    end: self.cursor,
                    line: self.line,
                    display_column: column,
                },
                token: LexerTokens::Symbol(token),
            }
        } else {
            // Couldn't match any known symbol. This is a lexer error: unknown token.
            let unknown_char = self.next().unwrap();
            let width = UnicodeWidthChar::width(unknown_char).unwrap_or(1);

            self.column += width;

            SpannedToken {
                span: CodeSpan {
                    start,
                    end: self.cursor,
                    line: self.line,
                    display_column: column,
                },
                token: LexerTokens::Illegal(unknown_char.to_string()),
            }
        }
    }

    pub fn scan(&mut self) -> Vec<SpannedToken> {
        let mut tokens = vec![SpannedToken {
            span: CodeSpan {
                start: 0,
                end: 0,
                line: 1,
                display_column: 1,
            },
            token: LexerTokens::SOI,
        }];

        while let Some(c) = self.peek().cloned() {
            let start = self.cursor;

            if c == '\n' || c == '\r' {
                let nl_line = self.line;
                let nl_col = self.column;
                let start_char = self.next().unwrap();
                self.consume_newline(start_char);
                tokens.push(SpannedToken {
                    span: CodeSpan {
                        start,
                        end: self.cursor,
                        line: nl_line,
                        display_column: nl_col,
                    },
                    token: LexerTokens::Newline,
                });
                continue;
            }

            if c.is_whitespace() {
                self.next();
                continue;
            }

            // Leading underscore is not XID_Start (it's Unicode category Pc,
            // not a letter) but every language with underscore-prefixed
            // identifiers special-cases it; mid-identifier underscores
            // already work since Pc is XID_Continue.
            if UnicodeXID::is_xid_start(c) || c == '_' || is_emoji_presentation(c) {
                tokens.push(self.consume_identifier(self.cursor));
                continue;
            }

            if c.is_numeric() {
                tokens.push(self.consume_numbers(start));
                continue;
            }

            if c == '"' {
                tokens.push(self.consume_string(start));
                continue;
            }

            if c == '/' {
                match self.peek_at(1) {
                    Some('/') => {
                        let is_doc = self.peek_at(2) == Some('/');
                        tokens.push(self.consume_line_comment(start, is_doc));
                        continue;
                    }
                    Some('*') => {
                        tokens.push(self.consume_block_comment(start));
                        continue;
                    }
                    _ => {}
                }
            }

            tokens.push(self.consume_symbol(start));
        }

        tokens.push(SpannedToken {
            span: CodeSpan {
                start: self.cursor,
                end: self.cursor,
                line: self.line,
                display_column: self.column,
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
                    display_column: $column,
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
                s_token_test!(5, 6, 1, 6, LexerTokens::Newline),
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
                s_token_test!(5, 6, 1, 6, LexerTokens::Newline),
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
                s_token_test!(5, 7, 1, 6, LexerTokens::Newline),
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
                s_token_test!(1, 2, 1, 2, LexerTokens::Newline),
                s_token_test!(2, 3, 2, 1, LexerTokens::Newline),
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
                s_token_test!(6, 7, 1, 7, LexerTokens::Newline),
                s_token_test!(7, 9, 2, 1, LexerTokens::Newline),
                s_token_test!(9, 15, 3, 1, LexerTokens::Identifier("token2".into())),
                s_token_test!(15, 16, 3, 7, LexerTokens::Newline),
                s_token_test!(16, 18, 4, 1, LexerTokens::Newline),
                s_token_test!(18, 19, 5, 1, LexerTokens::Newline),
                s_token_test!(20, 26, 6, 2, LexerTokens::Identifier("token3".into())),
                s_token_test!(26, 26, 6, 8, LexerTokens::EOI),
            ],
            tokens
        );
    }

    #[test]
    fn test_column_tracking_scan_symbols() {
        let input = "   != ==";
        let mut lexer = ElpLexer::new(input);
        let tokens = lexer.scan();

        let expected = vec![
            SpannedToken {
                token: LexerTokens::SOI,
                span: CodeSpan {
                    start: 0,
                    end: 0,
                    display_column: 1,
                    line: 1,
                },
            },
            SpannedToken {
                token: LexerTokens::Symbol(LexerSymbol::NotEqual),
                span: CodeSpan {
                    start: 3,
                    end: 5,
                    line: 1,
                    display_column: 4,
                },
            },
            SpannedToken {
                token: LexerTokens::Symbol(LexerSymbol::EqualEqual),
                span: CodeSpan {
                    start: 6,
                    end: 8,
                    line: 1,
                    display_column: 7,
                },
            },
            SpannedToken {
                token: LexerTokens::EOI,
                span: CodeSpan {
                    start: 8,
                    end: 8,
                    display_column: 9,
                    line: 1,
                },
            },
        ];

        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_unicode_columns_in_elp_lexer() {
        let input = "α + б + 😀";
        let mut lexer = ElpLexer::new(input);
        let tokens: Vec<SpannedToken> = lexer.scan(); // or whatever method returns the Vec

        let expected = vec![
            SpannedToken {
                token: LexerTokens::SOI,
                span: CodeSpan {
                    start: 0,
                    end: 0,
                    display_column: 1,
                    line: 1,
                },
            },
            SpannedToken {
                span: CodeSpan {
                    start: 0,
                    end: 2, // 'α' is 2 bytes
                    line: 1,
                    display_column: 1,
                },
                token: LexerTokens::Identifier("α".into()),
            },
            SpannedToken {
                span: CodeSpan {
                    start: 3,
                    end: 4, // '+' is 1 byte, but preceded by 1 space
                    line: 1,
                    display_column: 3,
                },
                token: LexerTokens::Symbol(LexerSymbol::Plus),
            },
            SpannedToken {
                span: CodeSpan {
                    start: 5,
                    end: 7,
                    line: 1,
                    display_column: 5,
                },
                token: LexerTokens::Identifier("б".into()),
            },
            SpannedToken {
                span: CodeSpan {
                    start: 8,
                    end: 9, // '+' is 1 byte
                    line: 1,
                    display_column: 7,
                },
                token: LexerTokens::Symbol(LexerSymbol::Plus),
            },
            SpannedToken {
                span: CodeSpan {
                    start: 10,
                    end: 14, // '😀' is 4 bytes
                    line: 1,
                    display_column: 9,
                },
                token: LexerTokens::Identifier("😀".into()),
            },
            SpannedToken {
                token: LexerTokens::EOI,
                span: CodeSpan {
                    start: 14,
                    end: 14,
                    display_column: 11,
                    line: 1,
                },
            },
        ];

        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_cjk_wide_characters() {
        // Each CJK char is width 2, but column tracking may be off
        let input = "你好吗";
        let mut lexer = ElpLexer::new(input);
        let tokens = lexer.scan();
        assert_eq!(
            vec![
                SpannedToken {
                    token: LexerTokens::SOI,
                    span: CodeSpan {
                        start: 0,
                        end: 0,
                        display_column: 1,
                        line: 1,
                    },
                }, // CJK chars are 3 bytes each
                SpannedToken {
                    token: LexerTokens::Identifier("你好吗".into()),
                    span: CodeSpan {
                        start: 0,
                        end: 9,
                        display_column: 1,
                        line: 1,
                    },
                },
                SpannedToken {
                    token: LexerTokens::EOI,
                    span: CodeSpan {
                        start: 9,
                        end: 9,
                        display_column: 7,
                        line: 1,
                    },
                }
            ],
            tokens,
        );
    }

    #[test]
    fn test_wide_and_narrow_mixed() {
        let input = "a你b";
        let mut lexer = ElpLexer::new(input);
        let tokens = lexer.scan();
        assert_eq!(
            tokens,
            vec![
                SpannedToken {
                    span: CodeSpan {
                        start: 0,
                        end: 0,
                        line: 1,
                        display_column: 1,
                    },
                    token: LexerTokens::SOI,
                },
                SpannedToken {
                    span: CodeSpan {
                        start: 0,
                        end: 5,
                        line: 1,
                        display_column: 1,
                    },
                    token: LexerTokens::Identifier("a你b".into()),
                },
                SpannedToken {
                    span: CodeSpan {
                        start: 5,
                        end: 5,
                        line: 1,
                        display_column: 5,
                    },
                    token: LexerTokens::EOI,
                }
            ]
        );
    }

    #[test]
    fn test_emoji_single_and_double_width() {
        let input = "a😀b👍🏽";
        let mut lexer = ElpLexer::new(input);
        let tokens = lexer.scan();
        assert_eq!(
            tokens,
            vec![
                SpannedToken {
                    span: CodeSpan {
                        start: 0,
                        end: 0,
                        line: 1,
                        display_column: 1,
                    },
                    token: LexerTokens::SOI,
                },
                SpannedToken {
                    span: CodeSpan {
                        start: 0,
                        end: 14,
                        line: 1,
                        display_column: 1,
                    },
                    token: LexerTokens::Identifier("a😀b👍🏽".into()),
                },
                SpannedToken {
                    span: CodeSpan {
                        start: 14,
                        end: 14,
                        line: 1,
                        display_column: 9, // Check if emoji width is handled right
                    },
                    token: LexerTokens::EOI,
                }
            ]
        );
    }

    #[test]
    fn test_combining_mark() {
        // 'a' + combining acute accent, should be one grapheme, width 1
        let input = "a\u{0301}b";
        let mut lexer = ElpLexer::new(input);
        let tokens = lexer.scan();
        assert_eq!(
            tokens,
            vec![
                SpannedToken {
                    span: CodeSpan {
                        start: 0,
                        end: 0,
                        line: 1,
                        display_column: 1,
                    },
                    token: LexerTokens::SOI,
                },
                SpannedToken {
                    span: CodeSpan {
                        start: 0,
                        end: 4,
                        line: 1,
                        display_column: 1,
                    },
                    token: LexerTokens::Identifier("a\u{0301}b".into()),
                },
                SpannedToken {
                    span: CodeSpan {
                        start: 4,
                        end: 4,
                        line: 1,
                        display_column: 3, // Column may be off due to combining mark
                    },
                    token: LexerTokens::EOI,
                }
            ]
        );
    }

    #[test]
    fn test_zero_width_joiner_sequence() {
        // Woman technologist emoji: 👩‍💻 (woman + ZWJ + laptop)
        let input = "👩‍💻";
        let mut lexer = ElpLexer::new(input);
        let tokens = lexer.scan();
        assert_eq!(
            tokens,
            vec![
                SpannedToken {
                    span: CodeSpan {
                        start: 0,
                        end: 0,
                        line: 1,
                        display_column: 1,
                    },
                    token: LexerTokens::SOI,
                },
                SpannedToken {
                    span: CodeSpan {
                        start: 0,
                        end: 11,
                        line: 1,
                        display_column: 1,
                    },
                    token: LexerTokens::Identifier("👩‍💻".into()),
                },
                SpannedToken {
                    span: CodeSpan {
                        start: 11,
                        end: 11,
                        line: 1,
                        display_column: 5,
                    },
                    token: LexerTokens::EOI,
                }
            ]
        );
    }

    #[test]
    fn test_ambiguous_width_symbols() {
        // '§' is ambiguous width (sometimes 1, sometimes 2)
        let input = "§a";
        let mut lexer = ElpLexer::new(input);
        let tokens = lexer.scan();
        dbg!(&tokens);
        assert_eq!(
            tokens,
            vec![
                SpannedToken {
                    span: CodeSpan {
                        start: 0,
                        end: 0,
                        line: 1,
                        display_column: 1,
                    },
                    token: LexerTokens::SOI,
                },
                SpannedToken {
                    span: CodeSpan {
                        start: 0,
                        end: 2,
                        line: 1,
                        display_column: 1,
                    },
                    token: LexerTokens::Illegal("§".into()),
                },
                SpannedToken {
                    span: CodeSpan {
                        start: 2,
                        end: 3,
                        line: 1,
                        display_column: 3,
                    },
                    token: LexerTokens::Identifier("a".into()),
                },
                SpannedToken {
                    span: CodeSpan {
                        start: 3,
                        end: 3,
                        line: 1,
                        display_column: 4,
                    },
                    token: LexerTokens::EOI,
                }
            ]
        );
    }

    #[test]
    fn test_mixed_grapheme_clusters() {
        // "e\u{0301}" is é as 'e' + combining accent, then emoji, then CJK
        let input = "e\u{0301}😀你";
        let mut lexer = ElpLexer::new(input);
        let tokens = lexer.scan();
        assert_eq!(
            tokens,
            vec![
                SpannedToken {
                    span: CodeSpan {
                        start: 0,
                        end: 0,
                        line: 1,
                        display_column: 1,
                    },
                    token: LexerTokens::SOI,
                },
                SpannedToken {
                    span: CodeSpan {
                        start: 0,
                        end: 10,
                        line: 1,
                        display_column: 1,
                    },
                    token: LexerTokens::Identifier("e\u{0301}😀你".into()),
                },
                SpannedToken {
                    span: CodeSpan {
                        start: 10,
                        end: 10,
                        line: 1,
                        display_column: 6,
                    },
                    token: LexerTokens::EOI,
                }
            ]
        );
    }

    #[test]
    fn test_leading_underscore_identifier() {
        let input = "_unused";
        let mut lexer = ElpLexer::new(input);
        let tokens = lexer.scan();
        assert_eq!(
            vec![
                s_token_test!(0, 0, 1, 1, LexerTokens::SOI),
                s_token_test!(0, 7, 1, 1, LexerTokens::Identifier("_unused".into())),
                s_token_test!(7, 7, 1, 8, LexerTokens::EOI),
            ],
            tokens
        );
    }

    #[test]
    fn test_bare_underscore_wildcard() {
        let input = "_";
        let mut lexer = ElpLexer::new(input);
        let tokens = lexer.scan();
        assert_eq!(
            vec![
                s_token_test!(0, 0, 1, 1, LexerTokens::SOI),
                s_token_test!(0, 1, 1, 1, LexerTokens::Identifier("_".into())),
                s_token_test!(1, 1, 1, 2, LexerTokens::EOI),
            ],
            tokens
        );
    }

    #[test]
    fn test_new_single_and_multi_char_operators() {
        let input = "<= >= && || &= |= % & ! ? ( ) [ ] : @ ..";
        let mut lexer = ElpLexer::new(input);
        let tokens = lexer.scan();
        let symbols: Vec<LexerSymbol> = tokens
            .into_iter()
            .filter_map(|t| match t.token {
                LexerTokens::Symbol(s) => Some(s),
                _ => None,
            })
            .collect();
        assert_eq!(
            symbols,
            vec![
                LexerSymbol::LessEqual,
                LexerSymbol::GreaterEqual,
                LexerSymbol::AndAnd,
                LexerSymbol::OrOr,
                LexerSymbol::AmpersandEqual,
                LexerSymbol::PipeEqual,
                LexerSymbol::Percent,
                LexerSymbol::Ampersand,
                LexerSymbol::Bang,
                LexerSymbol::Question,
                LexerSymbol::LParen,
                LexerSymbol::RParen,
                LexerSymbol::LBracket,
                LexerSymbol::RBracket,
                LexerSymbol::Colon,
                LexerSymbol::At,
                LexerSymbol::Range,
            ]
        );
    }

    #[test]
    fn test_range_vs_period_longest_match() {
        // `400..499` must lex as Number Range Number, not Number Period Period Number.
        let input = "400..499";
        let mut lexer = ElpLexer::new(input);
        let tokens = lexer.scan();
        assert_eq!(
            vec![
                s_token_test!(0, 0, 1, 1, LexerTokens::SOI),
                s_token_test!(0, 3, 1, 1, LexerTokens::Number("400".into())),
                s_token_test!(3, 5, 1, 4, LexerTokens::Symbol(LexerSymbol::Range)),
                s_token_test!(5, 8, 1, 6, LexerTokens::Number("499".into())),
                s_token_test!(8, 8, 1, 9, LexerTokens::EOI),
            ],
            tokens
        );
    }

    #[test]
    fn test_line_comment_consumes_to_eol() {
        let input = "foo // a comment\nbar";
        let mut lexer = ElpLexer::new(input);
        let tokens = lexer.scan();
        assert_eq!(
            vec![
                s_token_test!(0, 0, 1, 1, LexerTokens::SOI),
                s_token_test!(0, 3, 1, 1, LexerTokens::Identifier("foo".into())),
                s_token_test!(4, 16, 1, 5, LexerTokens::LineComment(" a comment".into())),
                s_token_test!(16, 17, 1, 17, LexerTokens::Newline),
                s_token_test!(17, 20, 2, 1, LexerTokens::Identifier("bar".into())),
                s_token_test!(20, 20, 2, 4, LexerTokens::EOI),
            ],
            tokens
        );
    }

    #[test]
    fn test_doc_comment_vs_line_comment() {
        let input = "/// docs\n// not docs";
        let mut lexer = ElpLexer::new(input);
        let tokens = lexer.scan();
        assert_eq!(
            vec![
                s_token_test!(0, 0, 1, 1, LexerTokens::SOI),
                s_token_test!(0, 8, 1, 1, LexerTokens::DocComment(" docs".into())),
                s_token_test!(8, 9, 1, 9, LexerTokens::Newline),
                s_token_test!(9, 20, 2, 1, LexerTokens::LineComment(" not docs".into())),
                s_token_test!(20, 20, 2, 12, LexerTokens::EOI),
            ],
            tokens
        );
    }

    #[test]
    fn test_block_comment_single_line() {
        let input = "/* hello */x";
        let mut lexer = ElpLexer::new(input);
        let tokens = lexer.scan();
        assert_eq!(
            vec![
                s_token_test!(0, 0, 1, 1, LexerTokens::SOI),
                s_token_test!(0, 11, 1, 1, LexerTokens::BlockComment(" hello ".into())),
                s_token_test!(11, 12, 1, 12, LexerTokens::Identifier("x".into())),
                s_token_test!(12, 12, 1, 13, LexerTokens::EOI),
            ],
            tokens
        );
    }

    #[test]
    fn test_block_comment_spans_lines_and_tracks_position() {
        let input = "/*\nmulti\nline\n*/x";
        let mut lexer = ElpLexer::new(input);
        let tokens = lexer.scan();
        assert_eq!(
            tokens,
            vec![
                s_token_test!(0, 0, 1, 1, LexerTokens::SOI),
                s_token_test!(
                    0,
                    16,
                    4,
                    1,
                    LexerTokens::BlockComment("\nmulti\nline\n".into())
                ),
                s_token_test!(16, 17, 4, 3, LexerTokens::Identifier("x".into())),
                s_token_test!(17, 17, 4, 4, LexerTokens::EOI),
            ]
        );
    }

    #[test]
    fn test_unterminated_block_comment_consumes_to_eof() {
        let input = "/* never closed";
        let mut lexer = ElpLexer::new(input);
        let tokens = lexer.scan();
        assert_eq!(
            vec![
                s_token_test!(0, 0, 1, 1, LexerTokens::SOI),
                s_token_test!(
                    0,
                    15,
                    1,
                    1,
                    LexerTokens::BlockComment(" never closed".into())
                ),
                s_token_test!(15, 15, 1, 16, LexerTokens::EOI),
            ],
            tokens
        );
    }

    #[test]
    fn test_divide_and_divide_equal_still_work_alongside_comments() {
        let input = "a / b /= c";
        let mut lexer = ElpLexer::new(input);
        let tokens = lexer.scan();
        let symbols: Vec<LexerSymbol> = tokens
            .into_iter()
            .filter_map(|t| match t.token {
                LexerTokens::Symbol(s) => Some(s),
                _ => None,
            })
            .collect();
        assert_eq!(symbols, vec![LexerSymbol::Divide, LexerSymbol::DivideEqual]);
    }

    #[test]
    fn test_simple_string_literal() {
        let input = "\"hello world\"";
        let mut lexer = ElpLexer::new(input);
        let tokens = lexer.scan();
        assert_eq!(
            vec![
                s_token_test!(0, 0, 1, 1, LexerTokens::SOI),
                s_token_test!(0, 13, 1, 1, LexerTokens::String("\"hello world\"".into())),
                s_token_test!(13, 13, 1, 14, LexerTokens::EOI),
            ],
            tokens
        );
    }

    #[test]
    fn test_string_with_escaped_quote() {
        let input = r#""say \"hi\""x"#;
        let mut lexer = ElpLexer::new(input);
        let tokens = lexer.scan();
        assert_eq!(
            vec![
                s_token_test!(0, 0, 1, 1, LexerTokens::SOI),
                s_token_test!(0, 12, 1, 1, LexerTokens::String(r#""say \"hi\"""#.into())),
                s_token_test!(12, 13, 1, 13, LexerTokens::Identifier("x".into())),
                s_token_test!(13, 13, 1, 14, LexerTokens::EOI),
            ],
            tokens
        );
    }

    #[test]
    fn test_empty_string_literal() {
        let input = "\"\"";
        let mut lexer = ElpLexer::new(input);
        let tokens = lexer.scan();
        assert_eq!(
            vec![
                s_token_test!(0, 0, 1, 1, LexerTokens::SOI),
                s_token_test!(0, 2, 1, 1, LexerTokens::String("\"\"".into())),
                s_token_test!(2, 2, 1, 3, LexerTokens::EOI),
            ],
            tokens
        );
    }

    #[test]
    fn test_unterminated_string_stops_at_newline() {
        let input = "\"oops\nnext";
        let mut lexer = ElpLexer::new(input);
        let tokens = lexer.scan();
        assert_eq!(
            vec![
                s_token_test!(0, 0, 1, 1, LexerTokens::SOI),
                s_token_test!(0, 5, 1, 1, LexerTokens::String("\"oops".into())),
                s_token_test!(5, 6, 1, 6, LexerTokens::Newline),
                s_token_test!(6, 10, 2, 1, LexerTokens::Identifier("next".into())),
                s_token_test!(10, 10, 2, 5, LexerTokens::EOI),
            ],
            tokens
        );
    }

    #[test]
    fn test_unterminated_string_consumes_to_eof() {
        let input = "\"never closed";
        let mut lexer = ElpLexer::new(input);
        let tokens = lexer.scan();
        assert_eq!(
            vec![
                s_token_test!(0, 0, 1, 1, LexerTokens::SOI),
                s_token_test!(0, 13, 1, 1, LexerTokens::String("\"never closed".into())),
                s_token_test!(13, 13, 1, 14, LexerTokens::EOI),
            ],
            tokens
        );
    }
}
