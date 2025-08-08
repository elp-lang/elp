use crate::{ElpItem, Spanned};

#[derive(Debug, PartialEq)]
pub struct ExportNode {
    pub names: Vec<Spanned<ElpItem>>,
}
