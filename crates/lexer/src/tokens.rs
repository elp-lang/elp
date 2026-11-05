use std::fmt::Display;

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
    LCurley,
    RCurley,
    Comma,
    Period,
    Range,
    Colon,
    At,
    LessThan,
    GreaterThan,
    LessEqual,
    GreaterEqual,
    AndAnd,
    OrOr,
    Percent,
    Ampersand,
    Bang,
    Question,
    LParen,
    RParen,
    LBracket,
    RBracket,
    #[default]
    Error,
}

#[derive(Debug, PartialEq, Clone)]
pub enum LexerTokens {
    Identifier(String),
    Number(String),
    String(String),
    Boolean(bool),
    Symbol(LexerSymbol),
    Error(String),
    Illegal(String),
    Newline,
    LineComment(String),
    DocComment(String),
    BlockComment(String),
    SOI,
    EOI,
}

impl Display for LexerTokens {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LexerTokens::Identifier(ident) => write!(f, "{}", ident),
            LexerTokens::Number(num) => write!(f, "{}", num),
            LexerTokens::String(str) => write!(f, "{}", str),
            LexerTokens::Boolean(b) => write!(f, "{}", b),
            LexerTokens::Symbol(lexer_symbol) => write!(f, "{:?}", lexer_symbol),
            LexerTokens::Error(e) => write!(f, "{}", e),
            LexerTokens::Illegal(i) => write!(f, "{}", i),
            LexerTokens::Newline => write!(f, "Newline"),
            LexerTokens::LineComment(c) => write!(f, "//{}", c),
            LexerTokens::DocComment(c) => write!(f, "///{}", c),
            LexerTokens::BlockComment(c) => write!(f, "/*{}*/", c),
            LexerTokens::SOI => write!(f, "SOI"),
            LexerTokens::EOI => write!(f, "EOI"),
        }
    }
}
