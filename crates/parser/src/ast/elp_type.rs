use crate::cst::{
    elp_type::{
        CSTElpType, CSTElpTypeGeneric, CSTElpTypeGenericParam, CSTElpTypeParameter, CSTElpTypeValue,
    },
    ident::CSTIdent,
};

use super::{expression::ASTExpression, traits::FromCST};

#[derive(Debug, PartialEq, PartialOrd)]
pub enum NumericType {
    Int32,
    UInt32,
    Int64,
    UInt64,
}

#[derive(Debug, PartialEq, PartialOrd)]
pub enum BuiltInType {
    Array(ASTExpression),
    Numeric(NumericType),
    Void,
}

#[derive(Debug, PartialEq, PartialOrd)]
pub enum ASTMutability {
    Immutable,
    Mutable,
}

#[derive(Debug, PartialEq, PartialOrd)]
pub enum ASTPointerSemantics {
    Pointer,
    Reference,
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct TypeReference {
    pub name: String,
    pub mutability: ASTMutability,
    pub pointer_semantics: Option<ASTPointerSemantics>,
    pub generic_parameters: Vec<ASTElpType>,
}

#[derive(Debug, PartialEq, PartialOrd)]
pub enum ASTElpType {
    Intrinsic(BuiltInType),
    Reference(TypeReference),
}

impl FromCST<CSTElpType> for ASTElpType {
    fn from_cst(cst: &CSTElpType) -> Self {
        match &cst.value {
            CSTElpTypeValue::Array(arr) => ASTElpType::from_cst(&CSTElpType {
                mutability: cst.mutability,
                pointer_semantics: cst.pointer_semantics,
                value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                    name: CSTIdent {
                        value: "Array".into(),
                    },
                    generics: vec![CSTElpTypeGeneric {
                        params: vec![CSTElpTypeGenericParam {
                            elp_type: *arr.of_elp_type,
                            type_constraint: None,
                        }],
                    }],
                }),
            }),
            CSTElpTypeValue::Parameter(param) => match param.name.value.as_str() {
                "int32" => ASTElpType::Intrinsic(BuiltInType::Numeric(NumericType::Int32)),
                "uint32" => ASTElpType::Intrinsic(BuiltInType::Numeric(NumericType::UInt32)),
                "int64" => ASTElpType::Intrinsic(BuiltInType::Numeric(NumericType::Int64)),
                "uint64" => ASTElpType::Intrinsic(BuiltInType::Numeric(NumericType::UInt64)),
                "Array" => {
                    let generics: Vec<ASTElpType> = param
                        .generics
                        .iter()
                        .map(|g| {
                            let params = g
                                .params
                                .iter()
                                .map(|p| ASTElpType::from_cst(&p.elp_type))
                                .collect();

                            ASTElpType::Reference(TypeReference {
                                name: param.name.value.clone(),
                                mutability: ASTMutability::Immutable,
                                pointer_semantics: None,
                                generic_parameters: params,
                            })
                        })
                        .collect();

                    ASTElpType::Intrinsic(BuiltInType::Array(ASTExpression::ElpType(Box::new(
                        ASTElpType::Reference(TypeReference {
                            name: param.name.value,
                            mutability: ASTMutability::Immutable,
                            pointer_semantics: None,
                            generic_parameters: generics,
                        }),
                    ))))
                }
                &_ => ASTElpType::Reference(TypeReference {
                    name: param.name.value,
                    mutability: ASTMutability::Immutable,
                    pointer_semantics: None,
                    generic_parameters: vec![],
                }),
            },
        }
    }
}
