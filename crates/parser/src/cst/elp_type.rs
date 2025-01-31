use super::{ident::CSTIdent, variable_access::CSTPointerSemantics, CSTMutabilitySelector};
use crate::parser::Rule;
use pest::Span;
use pest_ast::FromPest;

#[derive(Debug, FromPest, PartialEq, Eq, Clone)]
#[pest_ast(rule(Rule::elp_type))]
pub struct CSTElpType<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
    pub pointer_semantics: Option<CSTPointerSemantics<'a>>,
    pub mutability: Option<CSTMutabilitySelector<'a>>,
    pub value: CSTElpTypeValue<'a>,
}

#[derive(Debug, FromPest, PartialEq, Eq, Clone)]
#[pest_ast(rule(Rule::elp_type_parameter))]
pub struct CSTElpTypeParameter<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
    pub name: CSTIdent<'a>,
    pub generics: Vec<CSTElpTypeGeneric<'a>>,
}

#[derive(Debug, FromPest, PartialEq, Eq, Clone)]
#[pest_ast(rule(Rule::elp_type_array))]
pub struct CSTElpTypeArray<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
    pub of_elp_type: Box<CSTElpType<'a>>,
}

#[derive(Debug, FromPest, PartialEq, Eq, Clone)]
#[pest_ast(rule(Rule::elp_type_value))]
pub enum CSTElpTypeValue<'a> {
    Array(CSTElpTypeArray<'a>),
    Parameter(CSTElpTypeParameter<'a>),
}

#[derive(Debug, FromPest, PartialEq, Eq, Clone)]
#[pest_ast(rule(Rule::elp_type_generic_param))]
pub struct CSTElpTypeGenericParam<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
    pub elp_type: CSTElpType<'a>,
    pub type_constraint: Option<CSTElpTypeGenericConstraint<'a>>,
}

#[derive(Debug, FromPest, PartialEq, Eq, Clone)]
#[pest_ast(rule(Rule::elp_type_generic))]
pub struct CSTElpTypeGeneric<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
    pub params: Vec<CSTElpTypeGenericParam<'a>>,
}

#[derive(Debug, FromPest, PartialEq, Eq, Clone)]
#[pest_ast(rule(Rule::elp_type_generic_constraint))]
pub struct CSTElpTypeGenericConstraint<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
    pub constraints: Vec<CSTElpType<'a>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::parser::ElpParser;
    use from_pest::FromPest;
    use pest::Parser;

    #[test]
    fn elp_type() {
        let expression_str = "String";
        let mut pairs = ElpParser::parse(Rule::elp_type, expression_str).unwrap();
        let ast = CSTElpType::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTElpType {
                span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                mutability: None,
                pointer_semantics: None,
                value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                    span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                    name: CSTIdent {
                        span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                        value: "String".into()
                    },
                    generics: vec![]
                })
            }
        )
    }

    #[test]
    fn elp_type_with_generic() {
        let expression_str = "Into<String>";
        let mut pairs = ElpParser::parse(Rule::elp_type, expression_str).unwrap();
        let ast = CSTElpType::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTElpType {
                span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                mutability: None,
                pointer_semantics: None,
                value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                    span: pest::Span::new(expression_str, 0, 4).unwrap(),
                    name: CSTIdent {
                        span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                        value: "Into".into()
                    },
                    generics: vec![CSTElpTypeGeneric {
                        span: pest::Span::new(expression_str, 4, expression_str.len()).unwrap(),
                        params: vec![CSTElpTypeGenericParam {
                            span: pest::Span::new(expression_str, 4, expression_str.len()).unwrap(),
                            elp_type: CSTElpType {
                                span: pest::Span::new(expression_str, 4, expression_str.len())
                                    .unwrap(),
                                mutability: None,
                                pointer_semantics: None,
                                value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                    span: pest::Span::new(expression_str, 4, expression_str.len())
                                        .unwrap(),
                                    name: CSTIdent {
                                        span: pest::Span::new(
                                            expression_str,
                                            0,
                                            expression_str.len()
                                        )
                                        .unwrap(),
                                        value: "String".into()
                                    },
                                    generics: vec![]
                                })
                            },
                            type_constraint: None
                        }]
                    }]
                })
            }
        )
    }

    #[test]
    fn elp_generic() {
        let expression_str = "<String: Copy>";
        let mut pairs = ElpParser::parse(Rule::elp_type_generic, expression_str).unwrap();
        let ast = CSTElpTypeGeneric::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTElpTypeGeneric {
                span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                params: vec![CSTElpTypeGenericParam {
                    span: pest::Span::new(expression_str, 0, 7).unwrap(),
                    elp_type: CSTElpType {
                        span: pest::Span::new(expression_str, 1, 6).unwrap(),
                        mutability: None,
                        pointer_semantics: None,
                        value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                            span: pest::Span::new(expression_str, 1, 6).unwrap(),
                            name: CSTIdent {
                                span: pest::Span::new(expression_str, 0, expression_str.len())
                                    .unwrap(),
                                value: "String".into()
                            },
                            generics: vec![]
                        })
                    },
                    type_constraint: Some(CSTElpTypeGenericConstraint {
                        span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                        constraints: vec![CSTElpType {
                            span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                            mutability: None,
                            pointer_semantics: None,
                            value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                span: pest::Span::new(expression_str, 0, expression_str.len())
                                    .unwrap(),
                                name: CSTIdent {
                                    span: pest::Span::new(expression_str, 0, expression_str.len())
                                        .unwrap(),
                                    value: "Copy".into()
                                },
                                generics: vec![]
                            })
                        }]
                    })
                }]
            }
        )
    }

    #[test]
    fn elp_single_generic_constraint() {
        let expression_str = "<String: Copy + Clone>";
        let mut pairs = ElpParser::parse(Rule::elp_type_generic, expression_str).unwrap();
        let ast = CSTElpTypeGeneric::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTElpTypeGeneric {
                span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                params: vec![CSTElpTypeGenericParam {
                    span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                    elp_type: CSTElpType {
                        span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                        mutability: None,
                        pointer_semantics: None,
                        value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                            span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                            name: CSTIdent {
                                span: pest::Span::new(expression_str, 0, expression_str.len())
                                    .unwrap(),
                                value: "String".into()
                            },
                            generics: vec![]
                        })
                    },
                    type_constraint: Some(CSTElpTypeGenericConstraint {
                        span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                        constraints: vec![
                            CSTElpType {
                                span: pest::Span::new(expression_str, 0, expression_str.len())
                                    .unwrap(),
                                mutability: None,
                                pointer_semantics: None,
                                value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                    span: pest::Span::new(expression_str, 0, expression_str.len())
                                        .unwrap(),
                                    name: CSTIdent {
                                        span: pest::Span::new(
                                            expression_str,
                                            0,
                                            expression_str.len()
                                        )
                                        .unwrap(),
                                        value: "Copy".into()
                                    },
                                    generics: vec![]
                                })
                            },
                            CSTElpType {
                                span: pest::Span::new(expression_str, 0, expression_str.len())
                                    .unwrap(),
                                mutability: None,
                                pointer_semantics: None,
                                value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                    span: pest::Span::new(expression_str, 0, expression_str.len())
                                        .unwrap(),
                                    name: CSTIdent {
                                        span: pest::Span::new(
                                            expression_str,
                                            0,
                                            expression_str.len()
                                        )
                                        .unwrap(),
                                        value: "Clone".into()
                                    },
                                    generics: vec![]
                                })
                            }
                        ]
                    })
                }]
            }
        )
    }

    #[test]
    fn elp_mixed_generic_constraints() {
        let expression_str = "<Number, String: Copy + Clone>";
        let mut pairs = ElpParser::parse(Rule::elp_type_generic, expression_str).unwrap();
        let ast = CSTElpTypeGeneric::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTElpTypeGeneric {
                span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                params: vec![
                    CSTElpTypeGenericParam {
                        span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                        elp_type: CSTElpType {
                            span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                            mutability: None,
                            pointer_semantics: None,
                            value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                span: pest::Span::new(expression_str, 0, expression_str.len())
                                    .unwrap(),
                                name: CSTIdent {
                                    span: pest::Span::new(expression_str, 0, expression_str.len())
                                        .unwrap(),
                                    value: "Number".into()
                                },
                                generics: vec![]
                            })
                        },
                        type_constraint: None
                    },
                    CSTElpTypeGenericParam {
                        span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                        elp_type: CSTElpType {
                            span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                            mutability: None,
                            pointer_semantics: None,
                            value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                span: pest::Span::new(expression_str, 0, expression_str.len())
                                    .unwrap(),
                                name: CSTIdent {
                                    span: pest::Span::new(expression_str, 0, expression_str.len())
                                        .unwrap(),
                                    value: "String".into()
                                },
                                generics: vec![]
                            })
                        },
                        type_constraint: Some(CSTElpTypeGenericConstraint {
                            span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                            constraints: vec![
                                CSTElpType {
                                    span: pest::Span::new(expression_str, 0, expression_str.len())
                                        .unwrap(),
                                    mutability: None,
                                    pointer_semantics: None,
                                    value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                        span: pest::Span::new(
                                            expression_str,
                                            0,
                                            expression_str.len()
                                        )
                                        .unwrap(),
                                        name: CSTIdent {
                                            span: pest::Span::new(
                                                expression_str,
                                                0,
                                                expression_str.len()
                                            )
                                            .unwrap(),
                                            value: "Copy".into()
                                        },
                                        generics: vec![]
                                    })
                                },
                                CSTElpType {
                                    span: pest::Span::new(expression_str, 0, expression_str.len())
                                        .unwrap(),
                                    mutability: None,
                                    pointer_semantics: None,
                                    value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                        span: pest::Span::new(
                                            expression_str,
                                            0,
                                            expression_str.len()
                                        )
                                        .unwrap(),
                                        name: CSTIdent {
                                            span: pest::Span::new(
                                                expression_str,
                                                0,
                                                expression_str.len()
                                            )
                                            .unwrap(),
                                            value: "Clone".into()
                                        },
                                        generics: vec![]
                                    })
                                }
                            ]
                        })
                    }
                ]
            }
        )
    }

    #[test]
    fn elp_multiple_generic_constraints() {
        let expression_str = "<Number: Copy, String: Copy + Clone>";
        let mut pairs = ElpParser::parse(Rule::elp_type_generic, expression_str).unwrap();
        let ast = CSTElpTypeGeneric::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTElpTypeGeneric {
                span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                params: vec![
                    CSTElpTypeGenericParam {
                        span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                        elp_type: CSTElpType {
                            span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                            mutability: None,
                            pointer_semantics: None,
                            value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                span: pest::Span::new(expression_str, 0, expression_str.len())
                                    .unwrap(),
                                name: CSTIdent {
                                    span: pest::Span::new(expression_str, 0, expression_str.len())
                                        .unwrap(),
                                    value: "Number".into()
                                },
                                generics: vec![]
                            })
                        },
                        type_constraint: Some(CSTElpTypeGenericConstraint {
                            span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                            constraints: vec![CSTElpType {
                                span: pest::Span::new(expression_str, 0, expression_str.len())
                                    .unwrap(),
                                mutability: None,
                                pointer_semantics: None,
                                value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                    span: pest::Span::new(expression_str, 0, expression_str.len())
                                        .unwrap(),
                                    name: CSTIdent {
                                        span: pest::Span::new(
                                            expression_str,
                                            0,
                                            expression_str.len()
                                        )
                                        .unwrap(),
                                        value: "Copy".into()
                                    },
                                    generics: vec![]
                                })
                            }]
                        })
                    },
                    CSTElpTypeGenericParam {
                        span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                        elp_type: CSTElpType {
                            span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                            mutability: None,
                            pointer_semantics: None,
                            value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                span: pest::Span::new(expression_str, 0, expression_str.len())
                                    .unwrap(),
                                name: CSTIdent {
                                    span: pest::Span::new(expression_str, 0, expression_str.len())
                                        .unwrap(),
                                    value: "String".into()
                                },
                                generics: vec![]
                            })
                        },
                        type_constraint: Some(CSTElpTypeGenericConstraint {
                            span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                            constraints: vec![
                                CSTElpType {
                                    span: pest::Span::new(expression_str, 0, expression_str.len())
                                        .unwrap(),
                                    mutability: None,
                                    pointer_semantics: None,
                                    value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                        span: pest::Span::new(
                                            expression_str,
                                            0,
                                            expression_str.len()
                                        )
                                        .unwrap(),
                                        name: CSTIdent {
                                            span: pest::Span::new(
                                                expression_str,
                                                0,
                                                expression_str.len()
                                            )
                                            .unwrap(),
                                            value: "Copy".into()
                                        },
                                        generics: vec![]
                                    })
                                },
                                CSTElpType {
                                    span: pest::Span::new(expression_str, 0, expression_str.len())
                                        .unwrap(),
                                    mutability: None,
                                    pointer_semantics: None,
                                    value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                        span: pest::Span::new(
                                            expression_str,
                                            0,
                                            expression_str.len()
                                        )
                                        .unwrap(),
                                        name: CSTIdent {
                                            span: pest::Span::new(
                                                expression_str,
                                                0,
                                                expression_str.len()
                                            )
                                            .unwrap(),
                                            value: "Clone".into()
                                        },
                                        generics: vec![]
                                    })
                                }
                            ]
                        })
                    }
                ]
            }
        )
    }
}
