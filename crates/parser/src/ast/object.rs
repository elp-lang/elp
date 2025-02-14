use pest::Span;

use crate::cst::object::{CSTObject, CSTObjectMember};

use super::traits::FromCST;

#[derive(Debug, PartialEq, Clone)]
pub struct ASTObject<'a> {
    pub span: &'a Span<'a>,
    pub name: String,
    pub members: Vec<ASTObjectMember<'a>>,
}

impl<'a> FromCST<'a, CSTObject<'a>> for ASTObject<'a> {
    fn from_cst(cst: &'a CSTObject) -> Self {
        ASTObject {
            span: &cst.span,
            name: String::new(),
            members: vec![],
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ASTObjectMember<'a> {
    pub span: &'a Span<'a>,
}

impl<'a> FromCST<'a, CSTObjectMember<'a>> for ASTObjectMember<'a> {
    fn from_cst(cst: &'a CSTObjectMember) -> Self {
        ASTObjectMember { span: &cst.span }
    }
}
