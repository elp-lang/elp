use crate::ast::{export::ExportNode, import::ImportNode, r#macro::MacroNode};

#[derive(Debug, PartialEq)]
pub enum Decl {
    Import(ImportNode),
    Export(ExportNode),
    Macro(MacroNode),
}
