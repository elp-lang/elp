use crate::cst::{
    elp_type::{CSTElpType, CSTElpTypeParameter, CSTElpTypeValue},
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
    pub type_constraints: Vec<ASTElpType>,
}

#[derive(Debug, PartialEq, PartialOrd)]
pub enum ASTElpType {
    Intrinsic(BuiltInType),
    Reference(TypeReference),
}

impl FromCST<CSTElpTypeValue<'_>> for ASTElpType {
    fn from_cst(cst: &CSTElpTypeValue) -> Self {
        match cst {
            CSTElpTypeValue::Parameter(param) => ASTElpType::from_cst(param),
            CSTElpTypeValue::Array(arr) => ASTElpType::from_cst(&arr.of_type_param.to_elp_type()),
        }
    }
}

impl FromCST<CSTElpTypeParameter<'_>> for ASTElpType {
    fn from_cst(cst: &CSTElpTypeParameter) -> Self {
        ASTElpType::from_cst(&cst.to_elp_type())
    }
}

impl FromCST<CSTElpType<'_>> for ASTElpType {
    fn from_cst(cst: &CSTElpType) -> Self {
        match &cst.value {
            CSTElpTypeValue::Array(arr) => {
                ASTElpType::Intrinsic(BuiltInType::Array(ASTExpression::ElpType(Box::new(
                    ASTElpType::from_cst(&arr.of_type_param.clone().to_elp_type()),
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
                    pointer_semantics: cst
                        .pointer_semantics
                        .as_ref()
                        .map(ASTPointerSemantics::from_cst),
                    generic_parameters: match &cst.value {
                        CSTElpTypeValue::Parameter(param) => param
                            .generics
                            .clone()
                            .into_iter()
                            .flat_map(|e| {
                                e.params
                                    .into_iter()
                                    .map(|p| ASTElpType::from_cst(&p.elp_type))
                                    .collect::<Vec<ASTElpType>>()
                            })
                            .collect(),
                        CSTElpTypeValue::Array(arr) => {
                            vec![ASTElpType::from_cst(&arr.of_type_param.to_elp_type())]
                        }
                    },
                    type_constraints: vec![],
                }),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::cst::{
        elp_type::{
            CSTElpTypeArray, CSTElpTypeGeneric, CSTElpTypeGenericConstraint,
            CSTElpTypeGenericParam, CSTElpTypeParameter,
        },
        ident::CSTIdent,
        variable_access::CSTPointer,
    };
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn basic_elp_type_from_cst() {
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
                of_type_param: Box::new(CSTElpTypeParameter {
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
        );

        // *int32
        let cst_type_int_pointer = crate::cst::elp_type::CSTElpType {
            span: pest::Span::new("*int32", 0, 5).unwrap(),
            mutability: None,
            pointer_semantics: Some(CSTPointerSemantics::Pointer(CSTPointer {
                span: pest::Span::new("*int32", 0, 1).unwrap(),
            })),
            value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                span: pest::Span::new("*int32", 1, 5).unwrap(),
                name: CSTIdent {
                    span: pest::Span::new("*int32", 0, 5).unwrap(),
                    value: "*int32".into(),
                },
                generics: vec![],
            }),
        };

        let ast_type_int_pointer = ASTElpType::from_cst(&cst_type_int_pointer);

        assert_eq!(
            ast_type_int_pointer,
            ASTElpType::Reference(TypeReference {
                name: "*int32".into(),
                mutability: ASTMutability::Immutable,
                pointer_semantics: Some(ASTPointerSemantics::Pointer),
                generic_parameters: vec![],
                type_constraints: vec![],
            })
        );
    }

    #[test]
    fn complex_elp_type_from_cst() {
        // SpecialType<int32, Array<bool>>
        let generic_str = "SpecialType<Number: Copy, String: Copy + Clone>";
        let cst_type_simple_generic = CSTElpType {
            span: pest::Span::new(generic_str, 0, 17).unwrap(),
            mutability: None,
            pointer_semantics: None,
            value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                span: pest::Span::new(generic_str, 0, 17).unwrap(),
                name: CSTIdent {
                    span: pest::Span::new(generic_str, 0, 10).unwrap(),
                    value: "SpecialType".into(),
                },
                generics: vec![CSTElpTypeGeneric {
                    span: pest::Span::new(generic_str, 0, 36).unwrap(),
                    params: vec![
                        CSTElpTypeGenericParam {
                            span: pest::Span::new(generic_str, 1, 13).unwrap(),
                            elp_type: CSTElpType {
                                span: pest::Span::new(generic_str, 1, 7).unwrap(),
                                mutability: None,
                                pointer_semantics: None,
                                value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                    span: pest::Span::new(generic_str, 1, 7).unwrap(),
                                    name: CSTIdent {
                                        span: pest::Span::new(generic_str, 1, 7).unwrap(),
                                        value: "Number".into(),
                                    },
                                    generics: vec![],
                                }),
                            },
                            type_constraint: Some(CSTElpTypeGenericConstraint {
                                span: pest::Span::new(generic_str, 7, 13).unwrap(),
                                constraints: vec![CSTElpType {
                                    span: pest::Span::new(generic_str, 9, 13).unwrap(),
                                    mutability: None,
                                    pointer_semantics: None,
                                    value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                        span: pest::Span::new(generic_str, 9, 13).unwrap(),
                                        name: CSTIdent {
                                            span: pest::Span::new(generic_str, 9, 13).unwrap(),
                                            value: "Copy".into(),
                                        },
                                        generics: vec![],
                                    }),
                                }],
                            }),
                        },
                        CSTElpTypeGenericParam {
                            span: pest::Span::new(generic_str, 15, 35).unwrap(),
                            elp_type: CSTElpType {
                                span: pest::Span::new(generic_str, 15, 21).unwrap(),
                                mutability: None,
                                pointer_semantics: None,
                                value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                    span: pest::Span::new(generic_str, 15, 21).unwrap(),
                                    name: CSTIdent {
                                        span: pest::Span::new(generic_str, 15, 21).unwrap(),
                                        value: "String".into(),
                                    },
                                    generics: vec![],
                                }),
                            },
                            type_constraint: Some(CSTElpTypeGenericConstraint {
                                span: pest::Span::new(generic_str, 21, 35).unwrap(),
                                constraints: vec![
                                    CSTElpType {
                                        span: pest::Span::new(generic_str, 23, 28).unwrap(),
                                        mutability: None,
                                        pointer_semantics: None,
                                        value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                            span: pest::Span::new(generic_str, 23, 28).unwrap(),
                                            name: CSTIdent {
                                                span: pest::Span::new(generic_str, 23, 27).unwrap(),
                                                value: "Copy".into(),
                                            },
                                            generics: vec![],
                                        }),
                                    },
                                    CSTElpType {
                                        span: pest::Span::new(generic_str, 30, 35).unwrap(),
                                        mutability: None,
                                        pointer_semantics: None,
                                        value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                            span: pest::Span::new(generic_str, 30, 35).unwrap(),
                                            name: CSTIdent {
                                                span: pest::Span::new(generic_str, 30, 35).unwrap(),
                                                value: "Clone".into(),
                                            },
                                            generics: vec![],
                                        }),
                                    },
                                ],
                            }),
                        },
                    ],
                }],
            }),
        };

        let ast_type_simple_generic = ASTElpType::from_cst(&cst_type_simple_generic);

        assert_eq!(
            ast_type_simple_generic,
            ASTElpType::Reference(TypeReference {
                name: "SpecialType".into(),
                mutability: ASTMutability::Immutable,
                pointer_semantics: None,
                generic_parameters: vec![
                    ASTElpType::Reference(TypeReference {
                        name: "Number".into(),
                        mutability: ASTMutability::Immutable,
                        pointer_semantics: None,
                        generic_parameters: vec![],
                        type_constraints: vec![ASTElpType::Reference(TypeReference {
                            name: "Copy".into(),
                            mutability: ASTMutability::Immutable,
                            pointer_semantics: None,
                            generic_parameters: vec![],
                            type_constraints: vec![],
                        })],
                    }),
                    ASTElpType::Reference(TypeReference {
                        name: "String".into(),
                        mutability: ASTMutability::Immutable,
                        pointer_semantics: None,
                        generic_parameters: vec![],
                        type_constraints: vec![
                            ASTElpType::Reference(TypeReference {
                                name: "Copy".into(),
                                mutability: ASTMutability::Immutable,
                                pointer_semantics: None,
                                generic_parameters: vec![],
                                type_constraints: vec![],
                            }),
                            ASTElpType::Reference(TypeReference {
                                name: "Clone".into(),
                                mutability: ASTMutability::Immutable,
                                pointer_semantics: None,
                                generic_parameters: vec![],
                                type_constraints: vec![],
                            }),
                        ]
                    })
                ],
                type_constraints: vec![],
            })
        )
    }
}
