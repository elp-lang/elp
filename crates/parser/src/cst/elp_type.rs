use super::{ident::CSTIdent, variable_access::CSTPointerSemantics, CSTMutabilitySelector};
use crate::parser::Rule;
use pest_ast::FromPest;

#[derive(Debug, FromPest, PartialEq, Eq, Clone)]
#[pest_ast(rule(Rule::elp_type))]
pub struct CSTElpType {
    pub pointer_semantics: Option<CSTPointerSemantics>,
    pub mutability: Option<CSTMutabilitySelector>,
    pub value: CSTElpTypeValue,
}

#[derive(Debug, FromPest, PartialEq, Eq, Clone)]
#[pest_ast(rule(Rule::elp_type_parameter))]
pub struct CSTElpTypeParameter {
    pub name: CSTIdent,
    pub generics: Vec<CSTElpTypeGeneric>,
}

#[derive(Debug, FromPest, PartialEq, Eq, Clone)]
#[pest_ast(rule(Rule::elp_type_array))]
pub struct CSTElpTypeArray {
    pub of_elp_type: Box<CSTElpType>,
}

#[derive(Debug, FromPest, PartialEq, Eq, Clone)]
#[pest_ast(rule(Rule::elp_type_value))]
pub enum CSTElpTypeValue {
    Array(CSTElpTypeArray),
    Parameter(CSTElpTypeParameter),
}

#[derive(Debug, FromPest, PartialEq, Eq, Clone)]
#[pest_ast(rule(Rule::elp_type_generic_param))]
pub struct CSTElpTypeGenericParam {
    pub elp_type: CSTElpType,
    pub type_constraint: Option<CSTElpTypeGenericConstraint>,
}

#[derive(Debug, FromPest, PartialEq, Eq, Clone)]
#[pest_ast(rule(Rule::elp_type_generic))]
pub struct CSTElpTypeGeneric {
    pub params: Vec<CSTElpTypeGenericParam>,
}

#[derive(Debug, FromPest, PartialEq, Eq, Clone)]
#[pest_ast(rule(Rule::elp_type_generic_constraint))]
pub struct CSTElpTypeGenericConstraint {
    pub constraints: Vec<CSTElpType>,
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
                mutability: None,
                pointer_semantics: None,
                value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                    name: CSTIdent {
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
                mutability: None,
                pointer_semantics: None,
                value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                    name: CSTIdent {
                        value: "Into".into()
                    },
                    generics: vec![CSTElpTypeGeneric {
                        params: vec![CSTElpTypeGenericParam {
                            elp_type: CSTElpType {
                                mutability: None,
                                pointer_semantics: None,
                                value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                    name: CSTIdent {
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
                params: vec![CSTElpTypeGenericParam {
                    elp_type: CSTElpType {
                        mutability: None,
                        pointer_semantics: None,
                        value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                            name: CSTIdent {
                                value: "String".into()
                            },
                            generics: vec![]
                        })
                    },
                    type_constraint: Some(CSTElpTypeGenericConstraint {
                        constraints: vec![CSTElpType {
                            mutability: None,
                            pointer_semantics: None,
                            value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                name: CSTIdent {
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
                params: vec![CSTElpTypeGenericParam {
                    elp_type: CSTElpType {
                        mutability: None,
                        pointer_semantics: None,
                        value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                            name: CSTIdent {
                                value: "String".into()
                            },
                            generics: vec![]
                        })
                    },
                    type_constraint: Some(CSTElpTypeGenericConstraint {
                        constraints: vec![
                            CSTElpType {
                                mutability: None,
                                pointer_semantics: None,
                                value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                    name: CSTIdent {
                                        value: "Copy".into()
                                    },
                                    generics: vec![]
                                })
                            },
                            CSTElpType {
                                mutability: None,
                                pointer_semantics: None,
                                value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                    name: CSTIdent {
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
                params: vec![
                    CSTElpTypeGenericParam {
                        elp_type: CSTElpType {
                            mutability: None,
                            pointer_semantics: None,
                            value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                name: CSTIdent {
                                    value: "Number".into()
                                },
                                generics: vec![]
                            })
                        },
                        type_constraint: None
                    },
                    CSTElpTypeGenericParam {
                        elp_type: CSTElpType {
                            mutability: None,
                            pointer_semantics: None,
                            value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                name: CSTIdent {
                                    value: "String".into()
                                },
                                generics: vec![]
                            })
                        },
                        type_constraint: Some(CSTElpTypeGenericConstraint {
                            constraints: vec![
                                CSTElpType {
                                    mutability: None,
                                    pointer_semantics: None,
                                    value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                        name: CSTIdent {
                                            value: "Copy".into()
                                        },
                                        generics: vec![]
                                    })
                                },
                                CSTElpType {
                                    mutability: None,
                                    pointer_semantics: None,
                                    value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                        name: CSTIdent {
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
                params: vec![
                    CSTElpTypeGenericParam {
                        elp_type: CSTElpType {
                            mutability: None,
                            pointer_semantics: None,
                            value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                name: CSTIdent {
                                    value: "Number".into()
                                },
                                generics: vec![]
                            })
                        },
                        type_constraint: Some(CSTElpTypeGenericConstraint {
                            constraints: vec![CSTElpType {
                                mutability: None,
                                pointer_semantics: None,
                                value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                    name: CSTIdent {
                                        value: "Copy".into()
                                    },
                                    generics: vec![]
                                })
                            }]
                        })
                    },
                    CSTElpTypeGenericParam {
                        elp_type: CSTElpType {
                            mutability: None,
                            pointer_semantics: None,
                            value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                name: CSTIdent {
                                    value: "String".into()
                                },
                                generics: vec![]
                            })
                        },
                        type_constraint: Some(CSTElpTypeGenericConstraint {
                            constraints: vec![
                                CSTElpType {
                                    mutability: None,
                                    pointer_semantics: None,
                                    value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                        name: CSTIdent {
                                            value: "Copy".into()
                                        },
                                        generics: vec![]
                                    })
                                },
                                CSTElpType {
                                    mutability: None,
                                    pointer_semantics: None,
                                    value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                        name: CSTIdent {
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
