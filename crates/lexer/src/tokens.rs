#[derive(Debug, PartialEq)]
pub enum LexerSymbol {
    Equal,
    Plus,
    Minus,
    Multiply,
    Divide,
    SemiColon,
    EqualEqual,
}

#[derive(Debug, PartialEq)]
pub enum LexerTokens {
    Identifier(String),
    Number(String),
    String(String),
    Boolean(bool),
    Symbol(LexerSymbol),
    Error(String),
    SOI,
    EOF,
}
