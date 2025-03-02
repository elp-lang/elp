use pest::Span;

use crate::cst::variable_access::{CSTVariableAccess, CSTVariableAccessNames};

use super::{
    elp_type::{ASTElpType, ASTPointerSemantics},
    traits::FromCST,
};

#[derive(Debug, PartialEq, Clone)]
pub struct ASTVariableAccess<'a> {
    pub span: &'a Span<'a>,
    pub pointer_semantics: Vec<ASTPointerSemantics>,
    pub names: ASTVariableAccessNames<'a>,
}

impl<'a> FromCST<'a, CSTVariableAccess<'a>> for ASTVariableAccess<'a> {
    fn from_cst(cst: &'a CSTVariableAccess<'a>) -> Self {
        Self {
            span: &cst.span,
            pointer_semantics: cst
                .pointer_semantics
                .iter()
                .map(ASTPointerSemantics::from_cst)
                .collect(),
            names: ASTVariableAccessNames::from_cst(&cst.names),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ASTVariableAccessNames<'a> {
    pub span: &'a Span<'a>,
    pub names: Vec<String>,
}

impl<'a> FromCST<'a, CSTVariableAccessNames<'a>> for ASTVariableAccessNames<'a> {
    fn from_cst(cst: &'a CSTVariableAccessNames<'a>) -> Self {
        Self {
            span: &cst.span,
            names: cst.names.iter().map(|s| s.value.clone()).collect(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ASTContextualVariableAccess<'a> {
    pub context_type: Option<ASTElpType<'a>>,
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;
}
