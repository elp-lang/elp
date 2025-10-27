use lexer::{
    span::SpannedToken,
    tokens::{LexerSymbol, LexerTokens},
};

use crate::{ElpItem, ItemFromTokens, TokenCursor};

#[derive(Debug, PartialEq)]
pub enum Expr {
    Identifier {
        name: String,
        span: SpannedToken,
    },
    Int64 {
        value: i64,
        span: SpannedToken,
    },
    Import {
        module_path: Vec<String>,

        // names are stored as tuples, Import name -> Alias
        names: Vec<(String, Option<String>)>,
        span: SpannedToken,
    },
    Symbol {},
}

impl ItemFromTokens for Expr {
    fn from_tokens(tokens: &mut Vec<SpannedToken>) -> anyhow::Result<ElpItem> {
        let token = tokens.first().expect("Didn't find any tokens to pparse.");
        let mut cursor = TokenCursor::new(tokens);
        // We consume the first token to begin with.
        cursor.position += 1;

        match &token.token {
            LexerTokens::Symbol(symbol) => Self::from_symbol_tokens(symbol.clone(), tokens),
            LexerTokens::Identifier(ident) => Self::from_ident_tokens(ident.clone(), tokens),
            LexerTokens::Number(num) => Self::from_number_tokens(num.clone(), tokens),
            LexerTokens::String(string) => Self::from_string_tokens(string.clone(), tokens),
            LexerTokens::Boolean(bool) => Self::from_boolean_tokens(bool.clone(), tokens),
            LexerTokens::Error(err) => Self::from_error(err.clone(), tokens),
            LexerTokens::Illegal(illegal) => Self::from_illegal_tokens(illegal.clone(), tokens),
            LexerTokens::SOI => todo!(),
            LexerTokens::EOI => todo!(),
        }
    }
}

impl Expr {
    fn from_symbol_tokens(
        symbol: LexerSymbol,
        tokens: &mut Vec<SpannedToken>,
    ) -> anyhow::Result<ElpItem> {
    }

    fn from_ident_tokens(
        symbol: String,
        tokens: &mut Vec<SpannedToken>,
    ) -> anyhow::Result<ElpItem> {
    }

    fn from_number_tokens(
        symbol: String,
        tokens: &mut Vec<SpannedToken>,
    ) -> anyhow::Result<ElpItem> {
    }

    fn from_string_tokens(
        symbol: String,
        tokens: &mut Vec<SpannedToken>,
    ) -> anyhow::Result<ElpItem> {
    }

    fn from_boolean_tokens(
        symbol: bool,
        tokens: &mut Vec<SpannedToken>,
    ) -> anyhow::Result<ElpItem> {
    }

    fn from_error(symbol: String, tokens: &mut Vec<SpannedToken>) -> anyhow::Result<ElpItem> {}

    fn from_illegal_tokens(
        symbol: String,
        tokens: &mut Vec<SpannedToken>,
    ) -> anyhow::Result<ElpItem> {
    }
}
