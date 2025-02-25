use crate::cst::CSTModule;

use super::{expression::ASTExpression, traits::FromCST};

#[derive(Debug, PartialEq, Clone)]
pub struct ASTModule<'a> {
    pub name: Option<String>,
    pub expressions: Vec<ASTExpression<'a>>,
}

impl<'a> FromCST<'a, CSTModule<'a>> for ASTModule<'a> {
    fn from_cst(cst: &'a CSTModule) -> Self {
        ASTModule {
            name: None,
            expressions: cst
                .expressions
                .iter()
                .map(ASTExpression::from_cst)
                .collect(),
        }
    }
}
