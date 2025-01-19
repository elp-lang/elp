use crate::cst::object::{CSTObject, CSTObjectMember};

use super::traits::FromCST;

#[derive(Debug, PartialEq, PartialOrd)]
pub struct ASTObject {
    pub name: String,
    pub members: Vec<ASTObjectMember>,
}

impl FromCST<CSTObject> for ASTObject {
    fn from_cst(_cst: &CSTObject) -> Self {
        ASTObject {
            name: String::new(),
            members: vec![],
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct ASTObjectMember {}

impl FromCST<CSTObjectMember> for ASTObjectMember {
    fn from_cst(_cst: &CSTObjectMember) -> Self {
        ASTObjectMember {}
    }
}
