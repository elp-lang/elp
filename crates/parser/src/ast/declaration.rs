use crate::ast::r#macro::MacroNode;

#[derive(Debug, PartialEq)]
pub enum Decl {
    Macro(MacroNode),
}
