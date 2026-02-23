use std::task::Wake;

use lexer::tokens::LexerTokens;

use crate::{
    Spanned, TokenCursor,
    error::{ElpParserError, ParsingError},
};

#[derive(Debug, PartialEq)]
pub enum Decl {
    Function {},
    Variable {},
    Constant {},
    Import {
        var_names: Vec<String>,
        from_module: String,
    },
    Object {},
    Interface {},
    Export {},
    Macro {},
}

impl Decl {
    pub fn from_cursor(cursor: &mut TokenCursor) -> Result<Spanned<Self>, ElpParserError> {
        let lexer_token_opt = cursor.peek();
        if lexer_token_opt.is_none() {
            return Err(ElpParserError::ScriptParsingError(ParsingError::Expected(
                "No token to peek at, this is a compiler bug".into(),
            )));
        }

        let decl = match &lexer_token_opt.unwrap().token {
            LexerTokens::Identifier(ident) => decl_from_ident(ident.clone(), cursor),
            _ => Err(ElpParserError::ScriptParsingError(
                ParsingError::Unexpected(None, "Unexpected token".into()),
            )),
        };

        println!("{:#?}", decl);

        Err(ElpParserError::ScriptParsingError(ParsingError::Expected(
            "Finished too early".into(),
        )))
    }

    pub fn matches_ident(ident: &str) -> bool {
        matches!(
            ident,
            "fn" | "var" | "const" | "import" | "object" | "interface" | "export"
        )
    }
}

fn decl_from_ident(ident: String, cursor: &mut TokenCursor) -> Result<Decl, ElpParserError> {
    match ident.as_str() {
        "import" => import_from_cursor(cursor),
        _ => Err(ElpParserError::ScriptParsingError(
            ParsingError::Unexpected(None, "Unexpected token".into()),
        )),
    }
}

fn import_from_cursor(cursor: &mut TokenCursor) -> Result<Decl, ElpParserError> {
    let mut var_names: Vec<String> = vec![];
    let mut from_module: String = "".into();

	// We need to skip the opening brace.
	cursor.next_token()?;

	while let Some(token) = cursor.peek() {
		match token.token {
			LexerTokens::Identifier(ident) => var_names.push(ident),
			LexerTokens::Symbol(symbol) =>
		}
	}

    Ok(Decl::Import { var_names, from_module })
}
