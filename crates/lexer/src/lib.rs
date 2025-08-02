use span::CodeSpan;

use crate::{span::SpannedToken, tokens::LexerTokens};

pub mod span;
pub mod tokens;

pub struct ElpLexer {
    input: String,
    start: usize,
    cursor: usize,
    line: usize,
}

impl<'l> ElpLexer {
    pub fn new(input: String) -> Self {
        Self {
            input,
            start: 0,
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

    fn put_back(&mut self) {
        self.cursor -= 1;
    }

    fn peek_next(&self) -> char {
        if self.cursor + 1 >= self.input.len() {
            return '\0';
        }
        self.input.chars().nth(self.cursor + 1).unwrap()
    }

    fn skip_whitespace(&mut self) {
        while !self.is_at_end() && self.peek_next().is_whitespace() {
            self.consume()
        }
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
            let char = self.peek_next();
            if char.is_ascii_whitespace() || !char.is_ascii_alphanumeric() || char != '_' {
                break;
            }

            self.consume();
            value.push(self.input.chars().nth(self.cursor).unwrap());
        }
        token.span.end = self.cursor;
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
            let char = self.peek_next();
            if char.is_ascii_whitespace() || !char.is_numeric() || char != '_' {
                break;
            }

            self.consume();
            value.push(self.input.chars().nth(self.cursor).unwrap());
        }
        token.span.end = self.cursor;
        token.token = LexerTokens::Number(value);

        token
    }

    fn consume_symbols(&mut self) -> SpannedToken {}

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
            let next_char = self.peek_next();

            if next_char == '\0' {
                break;
            }

            if next_char.is_alphabetic() || next_char == '_' {
                tokens.push(self.consume_identifier());
            } else if next_char.is_numeric() {
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

    #[test]
    fn test_new() {
        let input = String::from("hello");
        let lexer = ElpLexer::new(input);
        assert_eq!(lexer.input, "hello");
    }

    #[test]
    fn test_is_at_end() {
        let input = String::from("hello");
        let lexer = ElpLexer::new(input);
        assert!(!lexer.is_at_end());
    }
}
