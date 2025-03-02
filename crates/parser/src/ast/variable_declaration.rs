use pest::Span;

use crate::cst::variable_declaration::CSTVariableDeclaration;

use super::{
    elp_type::{ASTElpType, ASTMutability},
    traits::FromCST,
};

#[derive(Debug, PartialEq, Clone)]
pub struct ASTVariableDeclaration<'a> {
    pub span: &'a Span<'a>,
    pub mutability: ASTMutability,
    pub name: String,
    pub type_annotation: Option<Box<ASTElpType<'a>>>,
}

impl<'a> FromCST<'a, CSTVariableDeclaration<'a>> for ASTVariableDeclaration<'a> {
    fn from_cst(cst: &'a CSTVariableDeclaration<'a>) -> Self {
        Self {
            span: &cst.span,
            mutability: ASTMutability::from_cst(&cst.mutability),
            name: cst.name.value.clone(),
            type_annotation: cst
                .type_annotation
                .as_ref()
                .map(|boxed| ASTElpType::from_cst(&**boxed))
                .map(Box::new),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
