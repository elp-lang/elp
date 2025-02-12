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
    pub generics: Option<CSTElpTypeGeneric<'a>>,
}

#[derive(Debug, FromPest, PartialEq, Eq, Clone)]
#[pest_ast(rule(Rule::elp_type_array))]
pub struct CSTElpTypeArray<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
    pub of_type_param: Box<CSTElpTypeParameter<'a>>,
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
    pub type_constraints: Option<CSTElpTypeGenericConstraint<'a>>,
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
    use pretty_assertions::assert_eq;

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
                    generics: None,
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
                span: pest::Span::new(expression_str, 0, 12).unwrap(),
                mutability: None,
                pointer_semantics: None,
                value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                    span: pest::Span::new(expression_str, 0, 12).unwrap(),
                    name: CSTIdent {
                        span: pest::Span::new(expression_str, 0, 4).unwrap(),
                        value: "Into".into()
                    },
                    generics: Some(CSTElpTypeGeneric {
                        span: pest::Span::new(expression_str, 4, 12).unwrap(),
                        params: vec![CSTElpTypeGenericParam {
                            span: pest::Span::new(expression_str, 5, 11).unwrap(),
                            elp_type: CSTElpType {
                                span: pest::Span::new(expression_str, 5, 11).unwrap(),
                                mutability: None,
                                pointer_semantics: None,
                                value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                    span: pest::Span::new(expression_str, 5, 11).unwrap(),
                                    name: CSTIdent {
                                        span: pest::Span::new(expression_str, 5, 11).unwrap(),
                                        value: "String".into()
                                    },
                                    generics: None
                                })
                            },
                            type_constraints: None
                        }]
                    })
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
                    span: pest::Span::new(expression_str, 1, 13).unwrap(),
                    elp_type: CSTElpType {
                        span: pest::Span::new(expression_str, 1, 7).unwrap(),
                        mutability: None,
                        pointer_semantics: None,
                        value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                            span: pest::Span::new(expression_str, 1, 7).unwrap(),
                            name: CSTIdent {
                                span: pest::Span::new(expression_str, 1, 7).unwrap(),
                                value: "String".into()
                            },
                            generics: None
                        })
                    },
                    type_constraints: Some(CSTElpTypeGenericConstraint {
                        span: pest::Span::new(expression_str, 7, 13).unwrap(),
                        constraints: vec![CSTElpType {
                            span: pest::Span::new(expression_str, 9, 13).unwrap(),
                            mutability: None,
                            pointer_semantics: None,
                            value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                span: pest::Span::new(expression_str, 9, 13).unwrap(),
                                name: CSTIdent {
                                    span: pest::Span::new(expression_str, 9, 13).unwrap(),
                                    value: "Copy".into()
                                },
                                generics: None
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
                span: pest::Span::new(expression_str, 0, 22).unwrap(),
                params: vec![CSTElpTypeGenericParam {
                    span: pest::Span::new(expression_str, 1, 21).unwrap(),
                    elp_type: CSTElpType {
                        span: pest::Span::new(expression_str, 1, 7).unwrap(),
                        mutability: None,
                        pointer_semantics: None,
                        value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                            span: pest::Span::new(expression_str, 1, 7).unwrap(),
                            name: CSTIdent {
                                span: pest::Span::new(expression_str, 1, 7).unwrap(),
                                value: "String".into()
                            },
                            generics: None
                        })
                    },
                    type_constraints: Some(CSTElpTypeGenericConstraint {
                        span: pest::Span::new(expression_str, 7, 21).unwrap(),
                        constraints: vec![
                            CSTElpType {
                                span: pest::Span::new(expression_str, 9, 14).unwrap(),
                                mutability: None,
                                pointer_semantics: None,
                                value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                    span: pest::Span::new(expression_str, 9, 14).unwrap(),
                                    name: CSTIdent {
                                        span: pest::Span::new(expression_str, 9, 13).unwrap(),
                                        value: "Copy".into()
                                    },
                                    generics: None
                                })
                            },
                            CSTElpType {
                                span: pest::Span::new(expression_str, 16, 21).unwrap(),
                                mutability: None,
                                pointer_semantics: None,
                                value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                    span: pest::Span::new(expression_str, 16, 21).unwrap(),
                                    name: CSTIdent {
                                        span: pest::Span::new(expression_str, 16, 21).unwrap(),
                                        value: "Clone".into()
                                    },
                                    generics: None
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
                span: pest::Span::new(expression_str, 0, 30).unwrap(),
                params: vec![
                    CSTElpTypeGenericParam {
                        span: pest::Span::new(expression_str, 1, 7).unwrap(),
                        elp_type: CSTElpType {
                            span: pest::Span::new(expression_str, 1, 7).unwrap(),
                            mutability: None,
                            pointer_semantics: None,
                            value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                span: pest::Span::new(expression_str, 1, 7).unwrap(),
                                name: CSTIdent {
                                    span: pest::Span::new(expression_str, 1, 7).unwrap(),
                                    value: "Number".into()
                                },
                                generics: None
                            })
                        },
                        type_constraints: None
                    },
                    CSTElpTypeGenericParam {
                        span: pest::Span::new(expression_str, 9, 29).unwrap(),
                        elp_type: CSTElpType {
                            span: pest::Span::new(expression_str, 9, 15).unwrap(),
                            mutability: None,
                            pointer_semantics: None,
                            value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                span: pest::Span::new(expression_str, 9, 15).unwrap(),
                                name: CSTIdent {
                                    span: pest::Span::new(expression_str, 9, 15).unwrap(),
                                    value: "String".into()
                                },
                                generics: None
                            })
                        },
                        type_constraints: Some(CSTElpTypeGenericConstraint {
                            span: pest::Span::new(expression_str, 15, 29).unwrap(),
                            constraints: vec![
                                CSTElpType {
                                    span: pest::Span::new(expression_str, 17, 22).unwrap(),
                                    mutability: None,
                                    pointer_semantics: None,
                                    value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                        span: pest::Span::new(expression_str, 17, 22).unwrap(),
                                        name: CSTIdent {
                                            span: pest::Span::new(expression_str, 17, 21).unwrap(),
                                            value: "Copy".into()
                                        },
                                        generics: None
                                    })
                                },
                                CSTElpType {
                                    span: pest::Span::new(expression_str, 24, 29).unwrap(),
                                    mutability: None,
                                    pointer_semantics: None,
                                    value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                        span: pest::Span::new(expression_str, 24, 29).unwrap(),
                                        name: CSTIdent {
                                            span: pest::Span::new(expression_str, 24, 29).unwrap(),
                                            value: "Clone".into()
                                        },
                                        generics: None
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
    fn elp_type_array() {
        let expression_str = "[Number]";
        let mut pairs = ElpParser::parse(Rule::elp_type_array, expression_str).unwrap();
        let ast = CSTElpTypeArray::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTElpTypeArray {
                span: pest::Span::new(expression_str, 0, 8).unwrap(),
                of_type_param: Box::new(CSTElpTypeParameter {
                    span: pest::Span::new(expression_str, 1, 7).unwrap(),
                    name: CSTIdent {
                        span: pest::Span::new(expression_str, 1, 7).unwrap(),
                        value: "Number".into()
                    },
                    generics: None
                })
            }
        );
    }

    #[test]
    fn elp_multiple_generic_constraints() {
        let expression_str = "<Number: Copy, String: Copy + Clone>";
        let mut pairs = ElpParser::parse(Rule::elp_type_generic, expression_str).unwrap();
        let ast = CSTElpTypeGeneric::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTElpTypeGeneric {
                span: pest::Span::new(expression_str, 0, 36).unwrap(),
                params: vec![
                    CSTElpTypeGenericParam {
                        span: pest::Span::new(expression_str, 1, 13).unwrap(),
                        elp_type: CSTElpType {
                            span: pest::Span::new(expression_str, 1, 7).unwrap(),
                            mutability: None,
                            pointer_semantics: None,
                            value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                span: pest::Span::new(expression_str, 1, 7).unwrap(),
                                name: CSTIdent {
                                    span: pest::Span::new(expression_str, 1, 7).unwrap(),
                                    value: "Number".into()
                                },
                                generics: None
                            })
                        },
                        type_constraints: Some(CSTElpTypeGenericConstraint {
                            span: pest::Span::new(expression_str, 7, 13).unwrap(),
                            constraints: vec![CSTElpType {
                                span: pest::Span::new(expression_str, 9, 13).unwrap(),
                                mutability: None,
                                pointer_semantics: None,
                                value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                    span: pest::Span::new(expression_str, 9, 13).unwrap(),
                                    name: CSTIdent {
                                        span: pest::Span::new(expression_str, 9, 13).unwrap(),
                                        value: "Copy".into()
                                    },
                                    generics: None
                                })
                            }]
                        })
                    },
                    CSTElpTypeGenericParam {
                        span: pest::Span::new(expression_str, 15, 35).unwrap(),
                        elp_type: CSTElpType {
                            span: pest::Span::new(expression_str, 15, 21).unwrap(),
                            mutability: None,
                            pointer_semantics: None,
                            value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                span: pest::Span::new(expression_str, 15, 21).unwrap(),
                                name: CSTIdent {
                                    span: pest::Span::new(expression_str, 15, 21).unwrap(),
                                    value: "String".into()
                                },
                                generics: None
                            })
                        },
                        type_constraints: Some(CSTElpTypeGenericConstraint {
                            span: pest::Span::new(expression_str, 21, 35).unwrap(),
                            constraints: vec![
                                CSTElpType {
                                    span: pest::Span::new(expression_str, 23, 28).unwrap(),
                                    mutability: None,
                                    pointer_semantics: None,
                                    value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                        span: pest::Span::new(expression_str, 23, 28).unwrap(),
                                        name: CSTIdent {
                                            span: pest::Span::new(expression_str, 23, 27).unwrap(),
                                            value: "Copy".into()
                                        },
                                        generics: None
                                    })
                                },
                                CSTElpType {
                                    span: pest::Span::new(expression_str, 30, 35).unwrap(),
                                    mutability: None,
                                    pointer_semantics: None,
                                    value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                        span: pest::Span::new(expression_str, 30, 35).unwrap(),
                                        name: CSTIdent {
                                            span: pest::Span::new(expression_str, 30, 35).unwrap(),
                                            value: "Clone".into()
                                        },
                                        generics: None
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
