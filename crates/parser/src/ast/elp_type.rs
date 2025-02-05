use crate::cst::{
    elp_type::{CSTElpType, CSTElpTypeValue},
    variable_access::CSTPointerSemantics,
    CSTMutabilitySelector,
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
}

#[derive(Debug, PartialEq, PartialOrd)]
pub enum ASTMutability {
    Immutable,
    Mutable,
}

impl FromCST<CSTMutabilitySelector<'_>> for ASTMutability {
    fn from_cst(cst: &CSTMutabilitySelector) -> Self {
        match cst {
            CSTMutabilitySelector::Mutable(_) => ASTMutability::Mutable,
            CSTMutabilitySelector::Immutable(_) => ASTMutability::Immutable,
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd)]
pub enum ASTPointerSemantics {
    Pointer,
    Reference,
}

impl FromCST<CSTPointerSemantics<'_>> for ASTPointerSemantics {
    fn from_cst(cst: &CSTPointerSemantics) -> Self {
        match cst {
            CSTPointerSemantics::Pointer(_) => ASTPointerSemantics::Pointer,
            CSTPointerSemantics::Reference(_) => ASTPointerSemantics::Reference,
        }
    }
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

impl FromCST<CSTElpType<'_>> for ASTElpType {
    fn from_cst(cst: &CSTElpType) -> Self {
        match &cst.value {
            CSTElpTypeValue::Array(arr) => {
                ASTElpType::Intrinsic(BuiltInType::Array(ASTExpression::ElpType(Box::new(
                    ASTElpType::from_cst(&arr.of_elp_type.clone().to_elp_type()),
                ))))
            }
            CSTElpTypeValue::Parameter(param) => match param.name.value.as_str() {
                "int32" => ASTElpType::Intrinsic(BuiltInType::Numeric(NumericType::Int32)),
                "uint32" => ASTElpType::Intrinsic(BuiltInType::Numeric(NumericType::UInt32)),
                "int64" => ASTElpType::Intrinsic(BuiltInType::Numeric(NumericType::Int64)),
                "uint64" => ASTElpType::Intrinsic(BuiltInType::Numeric(NumericType::UInt64)),
                "Array" => ASTElpType::from_cst(&param.to_elp_type()),
                &_ => ASTElpType::Reference(TypeReference {
                    name: param.name.value.clone(),
                    mutability: ASTMutability::Immutable,
                    pointer_semantics: None,
                    generic_parameters: vec![],
                }),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::cst::{
        elp_type::{CSTElpTypeArray, CSTElpTypeParameter},
        ident::CSTIdent,
    };
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn elp_type_from_cst() {
        // int32
        let cst_type_intrinsic = crate::cst::elp_type::CSTElpType {
            span: pest::Span::new("int32", 0, 5).unwrap(),
            mutability: None,
            pointer_semantics: None,
            value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                span: pest::Span::new("int32", 0, 5).unwrap(),
                name: CSTIdent {
                    span: pest::Span::new("int32", 0, 5).unwrap(),
                    value: "int32".into(),
                },
                generics: vec![],
            }),
        };
        let ast_type_intrinsic = ASTElpType::from_cst(&cst_type_intrinsic);

        assert_eq!(
            ast_type_intrinsic,
            ASTElpType::Intrinsic(BuiltInType::Numeric(NumericType::Int32))
        );

        // [int32]
        let cst_type_array = crate::cst::elp_type::CSTElpType {
            span: pest::Span::new("int32", 0, 5).unwrap(),
            mutability: None,
            pointer_semantics: None,
            value: CSTElpTypeValue::Array(CSTElpTypeArray {
                span: pest::Span::new("int32", 0, 5).unwrap(),
                of_elp_type: Box::new(CSTElpTypeParameter {
                    span: pest::Span::new("int32", 0, 5).unwrap(),
                    name: CSTIdent {
                        span: pest::Span::new("int32", 0, 5).unwrap(),
                        value: "int32".into(),
                    },
                    generics: vec![],
                }),
            }),
        };
        let ast_type_array = ASTElpType::from_cst(&cst_type_array);

        assert_eq!(
            ast_type_array,
            ASTElpType::Intrinsic(BuiltInType::Array(ASTExpression::ElpType(Box::new(
                ASTElpType::Intrinsic(BuiltInType::Numeric(NumericType::Int32))
            ))))
        )
    }
}
