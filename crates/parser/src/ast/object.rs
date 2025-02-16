use pest::Span;

use crate::cst::object::{CSTObject, CSTObjectImplements, CSTObjectMember};

use super::{elp_type::ASTElpType, traits::FromCST};

#[derive(Debug, PartialEq, Clone)]
pub struct ASTObject<'a> {
    pub span: &'a Span<'a>,
    pub name: String,
    pub members: Vec<ASTObjectMember<'a>>,
    pub implements: Vec<ASTObjectImplements<'a>>,
}

impl<'a> FromCST<'a, CSTObject<'a>> for ASTObject<'a> {
    fn from_cst(cst: &'a CSTObject) -> Self {
        ASTObject {
            span: &cst.span,
            name: String::new(),
            members: vec![],
            implements: vec![],
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ASTObjectImplements<'a> {
    pub span: &'a Span<'a>,
    pub types: Vec<ASTElpType<'a>>,
}

impl<'a> FromCST<'a, CSTObjectImplements<'a>> for ASTObjectImplements<'a> {
    fn from_cst(cst: &'a CSTObjectImplements) -> Self {
        ASTObjectImplements {
            span: &cst.span,
            types: cst.types.iter().map(ASTElpType::from_cst).collect(),
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
