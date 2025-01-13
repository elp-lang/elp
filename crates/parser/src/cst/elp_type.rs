use super::{ident::Ident, variable_access::PointerSemantics, MutabilitySelector};
use crate::parser::Rule;
use pest_ast::FromPest;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::elp_type))]
pub struct ElpType {
    pub pointer_semantics: Option<PointerSemantics>,
    pub mutability: Option<MutabilitySelector>,
    pub value: ElpTypeValue,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::elp_type_parameter))]
pub struct ElpTypeParameter {
    pub name: Ident,
    pub generics: Vec<ElpTypeGeneric>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::elp_type_array))]
pub struct ElpTypeArray {
    pub of_elp_type: Vec<ElpTypeParameter>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::elp_type_value))]
pub enum ElpTypeValue {
    Array(ElpTypeArray),
    Parameter(ElpTypeParameter),
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::elp_type_generic_param))]
pub struct ElpTypeGenericParam {
    pub elp_type: ElpType,
    pub type_constraint: Option<ElpTypeGenericConstraint>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::elp_type_generic))]
pub struct ElpTypeGeneric {
    pub params: Vec<ElpTypeGenericParam>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::elp_type_generic_constraint))]
pub struct ElpTypeGenericConstraint {
    pub constraints: Vec<ElpType>,
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
        let ast = ElpType::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            ElpType {
                mutability: None,
                pointer_semantics: None,
                value: ElpTypeValue::Parameter(ElpTypeParameter {
                    name: Ident {
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
        let ast = ElpType::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            ElpType {
                mutability: None,
                pointer_semantics: None,
                value: ElpTypeValue::Parameter(ElpTypeParameter {
                    name: Ident {
                        value: "Into".into()
                    },
                    generics: vec![ElpTypeGeneric {
                        params: vec![ElpTypeGenericParam {
                            elp_type: ElpType {
                                mutability: None,
                                pointer_semantics: None,
                                value: ElpTypeValue::Parameter(ElpTypeParameter {
                                    name: Ident {
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
        let ast = ElpTypeGeneric::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            ElpTypeGeneric {
                params: vec![ElpTypeGenericParam {
                    elp_type: ElpType {
                        mutability: None,
                        pointer_semantics: None,
                        value: ElpTypeValue::Parameter(ElpTypeParameter {
                            name: Ident {
                                value: "String".into()
                            },
                            generics: vec![]
                        })
                    },
                    type_constraint: Some(ElpTypeGenericConstraint {
                        constraints: vec![ElpType {
                            mutability: None,
                            pointer_semantics: None,
                            value: ElpTypeValue::Parameter(ElpTypeParameter {
                                name: Ident {
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
        let ast = ElpTypeGeneric::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            ElpTypeGeneric {
                params: vec![ElpTypeGenericParam {
                    elp_type: ElpType {
                        mutability: None,
                        pointer_semantics: None,
                        value: ElpTypeValue::Parameter(ElpTypeParameter {
                            name: Ident {
                                value: "String".into()
                            },
                            generics: vec![]
                        })
                    },
                    type_constraint: Some(ElpTypeGenericConstraint {
                        constraints: vec![
                            ElpType {
                                mutability: None,
                                pointer_semantics: None,
                                value: ElpTypeValue::Parameter(ElpTypeParameter {
                                    name: Ident {
                                        value: "Copy".into()
                                    },
                                    generics: vec![]
                                })
                            },
                            ElpType {
                                mutability: None,
                                pointer_semantics: None,
                                value: ElpTypeValue::Parameter(ElpTypeParameter {
                                    name: Ident {
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
        let ast = ElpTypeGeneric::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            ElpTypeGeneric {
                params: vec![
                    ElpTypeGenericParam {
                        elp_type: ElpType {
                            mutability: None,
                            pointer_semantics: None,
                            value: ElpTypeValue::Parameter(ElpTypeParameter {
                                name: Ident {
                                    value: "Number".into()
                                },
                                generics: vec![]
                            })
                        },
                        type_constraint: None
                    },
                    ElpTypeGenericParam {
                        elp_type: ElpType {
                            mutability: None,
                            pointer_semantics: None,
                            value: ElpTypeValue::Parameter(ElpTypeParameter {
                                name: Ident {
                                    value: "String".into()
                                },
                                generics: vec![]
                            })
                        },
                        type_constraint: Some(ElpTypeGenericConstraint {
                            constraints: vec![
                                ElpType {
                                    mutability: None,
                                    pointer_semantics: None,
                                    value: ElpTypeValue::Parameter(ElpTypeParameter {
                                        name: Ident {
                                            value: "Copy".into()
                                        },
                                        generics: vec![]
                                    })
                                },
                                ElpType {
                                    mutability: None,
                                    pointer_semantics: None,
                                    value: ElpTypeValue::Parameter(ElpTypeParameter {
                                        name: Ident {
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
        let ast = ElpTypeGeneric::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            ElpTypeGeneric {
                params: vec![
                    ElpTypeGenericParam {
                        elp_type: ElpType {
                            mutability: None,
                            pointer_semantics: None,
                            value: ElpTypeValue::Parameter(ElpTypeParameter {
                                name: Ident {
                                    value: "Number".into()
                                },
                                generics: vec![]
                            })
                        },
                        type_constraint: Some(ElpTypeGenericConstraint {
                            constraints: vec![ElpType {
                                mutability: None,
                                pointer_semantics: None,
                                value: ElpTypeValue::Parameter(ElpTypeParameter {
                                    name: Ident {
                                        value: "Copy".into()
                                    },
                                    generics: vec![]
                                })
                            }]
                        })
                    },
                    ElpTypeGenericParam {
                        elp_type: ElpType {
                            mutability: None,
                            pointer_semantics: None,
                            value: ElpTypeValue::Parameter(ElpTypeParameter {
                                name: Ident {
                                    value: "String".into()
                                },
                                generics: vec![]
                            })
                        },
                        type_constraint: Some(ElpTypeGenericConstraint {
                            constraints: vec![
                                ElpType {
                                    mutability: None,
                                    pointer_semantics: None,
                                    value: ElpTypeValue::Parameter(ElpTypeParameter {
                                        name: Ident {
                                            value: "Copy".into()
                                        },
                                        generics: vec![]
                                    })
                                },
                                ElpType {
                                    mutability: None,
                                    pointer_semantics: None,
                                    value: ElpTypeValue::Parameter(ElpTypeParameter {
                                        name: Ident {
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
