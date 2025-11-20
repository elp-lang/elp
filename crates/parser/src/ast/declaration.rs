use crate::{Spanned, TokenCursor, error::ElpParserError, parser::ElpParserConfig};

#[derive(Debug, PartialEq)]
pub enum Decl {
    Function {},
    Variable {},
    Constant {},
    Import {},
    Object {},
    Interface {},
    Export {},
    Macro {},
}

impl Decl {
    pub fn from_cursor(
        parser_config: &ElpParserConfig,
        cursor: &mut TokenCursor,
    ) -> Result<Spanned<Self>, ElpParserError> {
        let lexer_token = cursor.peek();
        if lexer_token.is_none() {
            return Err(ElpParserError::ScriptParsingError(ParsingError::Expected(
                "No token to peek at, this is a compiler bug".into(),
            )));
        }

        let item = match &lexer_token.unwrap().token {}

    }

    pub fn matches_ident(ident: &str) -> bool {
        matches!(
            ident,
            "fn" | "var" | "const" | "import" | "object" | "interface" | "export"
        )
    }
}
