#[derive(Debug, Default, PartialEq, Clone)]
pub enum LexerSymbol {
    Equal,
    Plus,
    Minus,
    Multiply,
    Divide,
    SemiColon,
    EqualEqual,
    Arrow,
    PlusEqual,
    MinusEqual,
    MultiplyEqual,
    DivideEqual,
    AmpersandEqual,
    PipeEqual,
    NotEqual,
    #[default]
    Error,
}

#[derive(Debug, PartialEq)]
pub enum LexerTokens {
    Identifier(String),
    Number(String),
    String(String),
    Boolean(bool),
    Symbol(LexerSymbol),
    Error(String),
    Unknown(String),
    SOI,
    EOI,
}
