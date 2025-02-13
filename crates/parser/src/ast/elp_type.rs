use crate::cst::{
    elp_type::{
        CSTElpType, CSTElpTypeArray, CSTElpTypeGenericParam, CSTElpTypeParameter, CSTElpTypeValue,
    },
    variable_access::CSTPointerSemantics,
    CSTMutabilitySelector,
};

use super::traits::FromCST;

#[derive(Debug, PartialEq, PartialOrd, Clone)]
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

#[derive(Debug, PartialEq, PartialOrd, Clone)]
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

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct ASTElpType {
    pub name: String,
    pub mutability: ASTMutability,
    pub pointer_semantics: Option<ASTPointerSemantics>,
    pub generic_parameters: Vec<ASTElpType>,
    pub type_constraints: Vec<ASTElpType>,
}

impl FromCST<CSTElpTypeGenericParam<'_>> for ASTElpType {
    fn from_cst(cst: &CSTElpTypeGenericParam) -> Self {
        let mut ast_elp_type = ASTElpType::from_cst(&cst.elp_type);
        ast_elp_type.type_constraints = match &cst.type_constraints {
            Some(constraint) => constraint
                .constraints
                .clone()
                .into_iter()
                .map(|constraint| ASTElpType::from_cst(&constraint))
                .collect(),
            None => vec![],
        };

        ast_elp_type
    }
}

impl FromCST<CSTElpTypeParameter<'_>> for ASTElpType {
    fn from_cst(cst: &CSTElpTypeParameter) -> Self {
        ASTElpType {
            name: cst.name.value.clone(),
            mutability: ASTMutability::Immutable,
            pointer_semantics: None,
            generic_parameters: match &cst.generics {
                Some(generic) => generic
                    .params
                    .clone()
                    .into_iter()
                    .map(|p| ASTElpType::from_cst(&p))
                    .collect(),
                None => vec![],
            },
            type_constraints: vec![],
        }
    }
}

impl FromCST<CSTElpTypeArray<'_>> for ASTElpType {
    fn from_cst(cst: &CSTElpTypeArray) -> Self {
        let generic_parameters = ASTElpType::from_cst(&*cst.of_type_param);
        let elp_type = ASTElpType {
            name: "Array".into(),
            mutability: ASTMutability::Immutable,
            pointer_semantics: None,
            generic_parameters: vec![generic_parameters],
            type_constraints: vec![],
        };

        elp_type.clone()
    }
}

impl FromCST<CSTElpType<'_>> for ASTElpType {
    fn from_cst(cst: &CSTElpType) -> Self {
        let mut elp_type = match &cst.value {
            CSTElpTypeValue::Array(arr) => ASTElpType::from_cst(arr),
            CSTElpTypeValue::Parameter(param) => ASTElpType::from_cst(param),
        };

        elp_type.pointer_semantics = cst
            .pointer_semantics
            .as_ref()
            .map(ASTPointerSemantics::from_cst);

        elp_type.clone()
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
                generics: None,
            }),
        };
        let ast_type_intrinsic = ASTElpType::from_cst(&cst_type_intrinsic);

        assert_eq!(
            ast_type_intrinsic,
            ASTElpType {
                name: "int32".into(),
                mutability: ASTMutability::Immutable,
                pointer_semantics: None,
                generic_parameters: vec![],
                type_constraints: vec![],
            }
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
                    generics: None,
                }),
            }),
        };
        let ast_type_array = ASTElpType::from_cst(&cst_type_array);

        assert_eq!(
            ast_type_array,
            ASTElpType {
                name: "Array".into(),
                mutability: ASTMutability::Immutable,
                pointer_semantics: None,
                generic_parameters: vec![ASTElpType {
                    name: "int32".into(),
                    mutability: ASTMutability::Immutable,
                    pointer_semantics: None,
                    generic_parameters: vec![],
                    type_constraints: vec![],
                }],
                type_constraints: vec![],
            }
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
                generics: None,
            }),
        };

        let ast_type_int_pointer = ASTElpType::from_cst(&cst_type_int_pointer);

        assert_eq!(
            ast_type_int_pointer,
            ASTElpType {
                name: "*int32".into(),
                mutability: ASTMutability::Immutable,
                pointer_semantics: Some(ASTPointerSemantics::Pointer),
                generic_parameters: vec![],
                type_constraints: vec![],
            }
        );
    }

    #[test]
    fn complex_elp_type_from_cst() {
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
                generics: Some(CSTElpTypeGeneric {
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
                                    generics: None,
                                }),
                            },
                            type_constraints: Some(CSTElpTypeGenericConstraint {
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
                                        generics: None,
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
                                    generics: None,
                                }),
                            },
                            type_constraints: Some(CSTElpTypeGenericConstraint {
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
                                            generics: None,
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
                                            generics: None,
                                        }),
                                    },
                                ],
                            }),
                        },
                    ],
                }),
            }),
        };

        let ast_type_simple_generic = ASTElpType::from_cst(&cst_type_simple_generic);

        assert_eq!(
            ast_type_simple_generic,
            ASTElpType {
                name: "SpecialType".into(),
                mutability: ASTMutability::Immutable,
                pointer_semantics: None,
                generic_parameters: vec![
                    ASTElpType {
                        name: "Number".into(),
                        mutability: ASTMutability::Immutable,
                        pointer_semantics: None,
                        generic_parameters: vec![],
                        type_constraints: vec![ASTElpType {
                            name: "Copy".into(),
                            mutability: ASTMutability::Immutable,
                            pointer_semantics: None,
                            generic_parameters: vec![],
                            type_constraints: vec![],
                        }],
                    },
                    ASTElpType {
                        name: "String".into(),
                        mutability: ASTMutability::Immutable,
                        pointer_semantics: None,
                        generic_parameters: vec![],
                        type_constraints: vec![
                            ASTElpType {
                                name: "Copy".into(),
                                mutability: ASTMutability::Immutable,
                                pointer_semantics: None,
                                generic_parameters: vec![],
                                type_constraints: vec![],
                            },
                            ASTElpType {
                                name: "Clone".into(),
                                mutability: ASTMutability::Immutable,
                                pointer_semantics: None,
                                generic_parameters: vec![],
                                type_constraints: vec![],
                            },
                        ]
                    }
                ],
                type_constraints: vec![],
            }
        )
    }
}
