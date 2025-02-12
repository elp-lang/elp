use crate::cst::object::{CSTObject, CSTObjectMember};

use super::traits::FromCST;

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct ASTObject {
    pub name: String,
    pub members: Vec<ASTObjectMember>,
}

impl FromCST<CSTObject<'_>> for ASTObject {
    fn from_cst(_cst: &CSTObject) -> Self {
        ASTObject {
            name: String::new(),
            members: vec![],
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct ASTObjectMember {}

impl FromCST<CSTObjectMember<'_>> for ASTObjectMember {
    fn from_cst(_cst: &CSTObjectMember) -> Self {
        ASTObjectMember {}
    }
}
