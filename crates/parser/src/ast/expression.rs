use lexer::{
    span::SpannedToken,
    tokens::{LexerSymbol, LexerTokens},
};

use crate::{ElpItem, TokenCursor};

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

impl From<&mut TokenCursor<'_>> for Expr {
    fn from(cursor: &mut TokenCursor) -> Self {
        let lexer_token = cursor.peek();
        if lexer_token.is_none() {
            panic!("No token to peek at, this is a compiler bug");
        }
        match lexer_token.unwrap().token {
            LexerTokens::Symbol(symbol) => Self::from_symbol_tokens(symbol.clone(), cursor),
            LexerTokens::Identifier(ident) => Self::from_ident_tokens(ident.clone(), cursor),
            LexerTokens::Number(num) => Self::from_number_tokens(num.clone(), cursor),
            LexerTokens::String(string) => Self::from_string_tokens(string.clone(), cursor),
            LexerTokens::Boolean(bool) => Self::from_boolean_tokens(bool.clone(), cursor),
            LexerTokens::Error(err) => Self::from_error(err.clone(), cursor),
            LexerTokens::Illegal(illegal) => Self::from_illegal_tokens(illegal.clone(), cursor),
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
