use crate::{TokenCursor, error::ElpParserError, parser::ElpParserConfig};

#[derive(Debug, PartialEq)]
pub enum Decl {
    Macro {},
}

impl Decl {
    fn from_cursor(
        parser_config: &ElpParserConfig,
        cursor: &mut TokenCursor,
    ) -> Result<Self, ElpParserError> {
    }
}
