use crate::cst::{elp_type::CSTElpType, object::CSTObjectMember};

use super::traits::FromCST;

#[derive(Debug, PartialEq, PartialOrd)]
pub enum BuiltInTypes {
    Void,
    Int32(i32),
    Int64(i64),
    Float32(f32),
    Float64(f64),
    Float(f64),
    Bool(bool),
    String(String),
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct ASTObject {
    pub name: String,
    pub members: Vec<ASTObjectMember>,
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct ASTObjectMember {}

#[derive(Debug, PartialEq, PartialOrd)]
pub enum ASTElpType {
    BuiltIn(BuiltInTypes),
    Object(ASTObject),
}

impl FromCST<CSTElpType> for ASTElpType {
    fn from_cst(_cst: &CSTElpType) -> Self {
        ASTElpType::BuiltIn(BuiltInTypes::Void)
    }
}

impl FromCST<CSTObjectMember> for ASTObjectMember {
    fn from_cst(_cst: &CSTObjectMember) -> Self {
        ASTObjectMember {}
    }
}
