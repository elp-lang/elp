#[derive(Debug, PartialEq)]
pub enum LexerTokens {
    Identifier(String),
    Number(String),
    String(String),
    Boolean(bool),
    Symbol(String),
    SOI,
    EOF,
}
