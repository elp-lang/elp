use span::CodeSpan;

pub mod span;
pub mod tokens;

pub struct ElpLexer {
    input: String,
    start: usize,
    current: usize,
    line: usize,
}

impl<'l> ElpLexer {
    pub fn new(input: String) -> Self {
        Self {
            input,
            start: 0,
            current: 0,
            line: 1,
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.input.len()
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.input.chars().nth(self.current - 1).unwrap()
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        self.input.chars().nth(self.current).unwrap()
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.input.len() {
            return '\0';
        }
        self.input.chars().nth(self.current + 1).unwrap()
    }

    fn produce() -> Vec<CodeSpan<'l>> {}
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
    fn test_advance() {
        let input = String::from("hello");
        let mut lexer = ElpLexer::new(input);
        assert_eq!(lexer.advance(), 'h');
        assert_eq!(lexer.current, 1);
    }

    #[test]
    fn test_is_at_end() {
        let input = String::from("hello");
        let lexer = ElpLexer::new(input);
        assert!(!lexer.is_at_end());
    }

    #[test]
    fn test_peek() {
        let input = String::from("hello");
        let lexer = ElpLexer::new(input);
        assert_eq!(lexer.peek(), 'h');
    }
}
