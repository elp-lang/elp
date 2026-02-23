use lexer::span::SpannedToken;

use crate::{
    Spanned, TokenCursor,
    error::{ElpParserError, ParsingError},
};

#[derive(Debug, PartialEq)]
pub enum Expr {
    Identifier {
        name: String,
    },
    Float32 {
        value: i32,
    },
    Float64 {
        value: i64,
    },
    Int32 {
        value: i32,
    },
    Int64 {
        value: i64,
    },
    Import {
        // the import path split on the period.
        module_path: Vec<String>,

        // names are stored as tuples, Import name -> Alias
        names: Vec<(String, Option<String>)>,
    },
    Symbol {},
}

impl Expr {
    pub fn from_cursor(cursor: &mut TokenCursor) -> Result<Spanned<Self>, ElpParserError> {
        let lexer_token = cursor.peek();
        if lexer_token.is_none() {
            return Err(ElpParserError::ScriptParsingError(ParsingError::Expected(
                "No token to peek at, this is a compiler bug".into(),
            )));
        }

        let item = match &lexer_token.unwrap().token {
            /*LexerTokens::Symbol(symbol) => Some(Self::from_symbol_tokens(symbol.clone(), cursor)?),
            LexerTokens::Identifier(ident) => Some(Self::from_ident_tokens(ident.clone(), cursor)?),
            LexerTokens::Number(num) => Some(Self::from_number_tokens(num.clone(), cursor)?),
            LexerTokens::String(string) => Some(Self::from_string_tokens(string.clone(), cursor)?),
            LexerTokens::Boolean(bool) => Some(Self::from_boolean_tokens(bool.clone(), cursor)?),
            LexerTokens::Error(err) => Some(Self::from_error(err.clone(), cursor)?),
            LexerTokens::Illegal(illegal) => {
                Some(Self::from_illegal_tokens(illegal.clone(), cursor)?)
            }*/
            _ => None,
        };

        match item {
            Some(item) => Ok(item),
            None => Err(ElpParserError::ScriptParsingError(ParsingError::Expected(
                "Expected an expression".into(),
            ))),
        }
    }

    /*
    pub fn from_symbol_tokens(
        symbol: LexerSymbol,
        cursor: &mut TokenCursor,
    ) -> Result<Spanned<Self>, ElpParserError> {
    }

    pub fn from_ident_tokens(
        symbol: String,
        cursor: &mut TokenCursor,
    ) -> Result<Spanned<Self>, ElpParserError> {
    }

    pub fn from_number_tokens(
        symbol: String,
        cursor: &mut TokenCursor,
    ) -> Result<Spanned<Self>, ElpParserError> {
    }

    pub fn from_string_tokens(
        symbol: String,
        cursor: &mut TokenCursor,
    ) -> Result<Spanned<Self>, ElpParserError> {
    }

    pub fn from_boolean_tokens(
        symbol: bool,
        cursor: &mut TokenCursor,
    ) -> Result<Spanned<Self>, ElpParserError> {
    }

    pub fn from_error(
        symbol: String,
        cursor: &mut TokenCursor,
    ) -> Result<Spanned<Self>, ElpParserError> {
    }

    pub fn from_illegal_tokens(
        symbol: String,
        cursor: &mut TokenCursor,
    ) -> Result<Spanned<Self>, ElpParserError> {
    }
    */
}
