use crate::TokenCursor;

#[derive(Debug, PartialEq)]
pub enum Decl {
    Macro {},
}

impl From<&mut TokenCursor<'_>> for Decl {
    fn from(value: &mut TokenCursor) -> Self {}
}
