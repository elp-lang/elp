use crate::cst::elp_type::{CSTElpType, CSTElpTypeValue};

use super::traits::FromCST;

fn intrinsic_cst_type(cst: CSTElpType) -> bool {
    match cst.value {
        CSTElpTypeValue::Array(arr) => intrinsic_cst_type(*arr.of_elp_type),
        CSTElpTypeValue::Parameter(param) => match param.name.value.as_str() {
            "int32" | "uint32" | "int64" | "uint64" => true,
            _ => false,
        },
    }
}

#[derive(Debug, PartialEq, PartialOrd)]
pub enum BuiltInTypes {
    Void,
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct TypeReference {
    pub as_array: bool,
    pub name: String,
}

#[derive(Debug, PartialEq, PartialOrd)]
pub enum ASTElpType {
    BuiltIn(BuiltInTypes),
    Reference(TypeReference),
}

impl FromCST<CSTElpType> for ASTElpType {
    fn from_cst(cst: &CSTElpType) -> Self {
        if intrinsic_cst_type(*cst) {
            ASTElpType::BuiltIn(BuiltInTypes::Void)
        } else {
            match cst.value {
                CSTElpTypeValue::Parameter(param) => ASTElpType::Reference(TypeReference {
                    as_array: false,
                    name: param.name.value,
                }),
                CSTElpTypeValue::Array(arr) => ASTElpType::Reference(TypeReference {
                    as_array: true,
                    name: arr.of_elp_type.value,
                }),
            }
        }
    }
}
