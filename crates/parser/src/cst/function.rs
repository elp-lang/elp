use super::elp_type::ElpTypeGeneric;
use super::variable_access::{ContextualVariableAccess, PointerSemantics};
use super::{
    block::Block, elp_type::ElpType, expression::CSTExpression, variable_access::VariableAccess,
};
use crate::cst::ident::Ident;
use crate::parser::Rule;
use pest_ast::FromPest;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::function_argument))]
pub struct FunctionArgument {
    pub pointer_semantics: Option<PointerSemantics>,
    pub name: Ident,
    pub type_annotation: Option<ElpType>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::function_arguments))]
pub struct FunctionArguments {
    pub arguments: Vec<FunctionArgument>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::function_return_type))]
pub struct FunctionReturnType {
    pub type_annotations: Vec<ElpType>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::function_return_value))]
pub struct FunctionReturnValue {
    pub value: Box<CSTExpression>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::function_def))]
pub struct FunctionDef {
    pub name: VariableAccess,
    pub generics: Option<ElpTypeGeneric>,
    pub arguments: Option<FunctionArguments>,
    pub return_type: Option<FunctionReturnType>,
    pub block: Box<Block>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::fn_header_def))]
pub struct FunctionHeaderDef {
    pub pointer_semantics: Option<PointerSemantics>,
    pub name: VariableAccess,
    pub generics: Option<ElpTypeGeneric>,
    pub arguments: FunctionArguments,
    pub return_type: FunctionReturnType,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::function_call_name))]
pub enum FunctionCallName {
    VariableAccess(VariableAccess),
    ContextualVariableAccess(ContextualVariableAccess),
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::function_call))]
pub struct FunctionCall {
    pub name: FunctionCallName,
    pub generics: Option<ElpTypeGeneric>,
    pub arguments: Vec<CSTExpression>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cst::{
            elp_type::{ElpTypeParameter, ElpTypeValue},
            string::StringValue,
            variable_access::VariableAccessNames,
        },
        parser::ElpParser,
    };
    use from_pest::FromPest;
    use pest::Parser;
    use pretty_assertions::assert_eq;

    #[test]
    fn function_arguments_no_self() {
        let expression_str = "(name String, hello String)";
        let mut pairs = ElpParser::parse(Rule::function_arguments, expression_str).unwrap();
        let ast = FunctionArguments::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            FunctionArguments {
                arguments: vec![
                    FunctionArgument {
                        name: Ident {
                            value: "name".into()
                        },
                        pointer_semantics: None,
                        type_annotation: Some(ElpType {
                            mutability: None,
                            pointer_semantics: None,
                            value: ElpTypeValue::Parameter(ElpTypeParameter {
                                name: Ident {
                                    value: "String".into()
                                },
                                generics: vec![],
                            })
                        }),
                    },
                    FunctionArgument {
                        name: Ident {
                            value: "hello".into()
                        },
                        pointer_semantics: None,
                        type_annotation: Some(ElpType {
                            mutability: None,
                            pointer_semantics: None,
                            value: ElpTypeValue::Parameter(ElpTypeParameter {
                                name: Ident {
                                    value: "String".into()
                                },
                                generics: vec![],
                            })
                        }),
                    }
                ]
            }
        );
    }

    #[test]
    fn function_arguments_with_self() {
        let expression_str = "(self, name String, hello String)";
        let mut pairs = ElpParser::parse(Rule::function_arguments, expression_str).unwrap();
        let ast = FunctionArguments::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            FunctionArguments {
                arguments: vec![
                    FunctionArgument {
                        name: Ident {
                            value: "self".into()
                        },
                        pointer_semantics: None,
                        type_annotation: None,
                    },
                    FunctionArgument {
                        name: Ident {
                            value: "name".into()
                        },
                        pointer_semantics: None,
                        type_annotation: Some(ElpType {
                            mutability: None,
                            pointer_semantics: None,
                            value: ElpTypeValue::Parameter(ElpTypeParameter {
                                name: Ident {
                                    value: "String".into()
                                },
                                generics: vec![],
                            })
                        }),
                    },
                    FunctionArgument {
                        name: Ident {
                            value: "hello".into()
                        },
                        pointer_semantics: None,
                        type_annotation: Some(ElpType {
                            mutability: None,
                            pointer_semantics: None,
                            value: ElpTypeValue::Parameter(ElpTypeParameter {
                                name: Ident {
                                    value: "String".into()
                                },
                                generics: vec![],
                            })
                        }),
                    }
                ]
            }
        );
    }

    #[test]
    fn function_return_type() {
        let expression_str = "-> String";
        let mut pairs = ElpParser::parse(Rule::function_return_type, expression_str).unwrap();
        let ast = FunctionReturnType::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            FunctionReturnType {
                type_annotations: vec![ElpType {
                    mutability: None,
                    pointer_semantics: None,
                    value: ElpTypeValue::Parameter(ElpTypeParameter {
                        name: Ident {
                            value: "String".into()
                        },
                        generics: vec![],
                    })
                }]
            }
        );
    }

    #[test]
    fn function_return_value() {
        let expression_str = "return \"hello\"";
        let mut pairs = ElpParser::parse(Rule::function_return_value, expression_str).unwrap();
        let ast = FunctionReturnValue::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            FunctionReturnValue {
                value: Box::new(CSTExpression::String(Box::new(StringValue {
                    value: "hello".into()
                })))
            }
        );
    }

    #[test]
    fn simple_function_def() {
        let expression_str = "fn hello.name(name String) -> String { return \"hello {name}\" }";
        let mut pairs = ElpParser::parse(Rule::function_def, expression_str).unwrap();
        let ast = FunctionDef::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            FunctionDef {
                name: VariableAccess {
                    names: VariableAccessNames {
                        names: vec![
                            Ident {
                                value: "hello".into()
                            },
                            Ident {
                                value: "name".into()
                            }
                        ],
                    },
                    pointer_semantics: vec![],
                },
                generics: None,
                arguments: Some(FunctionArguments {
                    arguments: vec![FunctionArgument {
                        name: Ident {
                            value: "name".into()
                        },
                        pointer_semantics: None,
                        type_annotation: Some(ElpType {
                            mutability: None,
                            pointer_semantics: None,
                            value: ElpTypeValue::Parameter(ElpTypeParameter {
                                name: Ident {
                                    value: "String".into()
                                },
                                generics: vec![],
                            })
                        }),
                    }],
                }),
                return_type: Some(FunctionReturnType {
                    type_annotations: vec![ElpType {
                        mutability: None,
                        pointer_semantics: None,
                        value: ElpTypeValue::Parameter(ElpTypeParameter {
                            name: Ident {
                                value: "String".into()
                            },
                            generics: vec![],
                        })
                    }],
                }),
                block: Box::new(Block {
                    expressions: vec![CSTExpression::FunctionReturnValue(Box::new(
                        FunctionReturnValue {
                            value: Box::new(CSTExpression::String(Box::new(StringValue {
                                value: "hello {name}".into()
                            })))
                        }
                    ))]
                })
            }
        )
    }

    #[test]
    fn external_function_def() {
        let expression_str = "fn hello(name String) -> String";
        let mut pairs = ElpParser::parse(Rule::fn_header_def, expression_str).unwrap();
        let ast = FunctionHeaderDef::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            FunctionHeaderDef {
                pointer_semantics: None,
                name: VariableAccess {
                    names: VariableAccessNames {
                        names: vec![Ident {
                            value: "hello".into()
                        },],
                    },
                    pointer_semantics: vec![],
                },
                generics: None,
                arguments: FunctionArguments {
                    arguments: vec![FunctionArgument {
                        name: Ident {
                            value: "name".into()
                        },
                        pointer_semantics: None,
                        type_annotation: Some(ElpType {
                            mutability: None,
                            pointer_semantics: None,
                            value: ElpTypeValue::Parameter(ElpTypeParameter {
                                name: Ident {
                                    value: "String".into()
                                },
                                generics: vec![],
                            })
                        }),
                    }],
                },
                return_type: FunctionReturnType {
                    type_annotations: vec![ElpType {
                        mutability: None,
                        pointer_semantics: None,
                        value: ElpTypeValue::Parameter(ElpTypeParameter {
                            name: Ident {
                                value: "String".into()
                            },
                            generics: vec![],
                        })
                    }],
                },
            }
        )
    }

    #[test]
    fn function_call() {
        let expression_str = "hello.name()";
        let mut pairs = ElpParser::parse(Rule::function_call, expression_str).unwrap();
        let ast = FunctionCall::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            FunctionCall {
                name: FunctionCallName::VariableAccess(VariableAccess {
                    names: VariableAccessNames {
                        names: vec![
                            Ident {
                                value: "hello".into()
                            },
                            Ident {
                                value: "name".into()
                            },
                        ],
                    },
                    pointer_semantics: vec![],
                }),
                generics: None,
                arguments: Vec::new(),
            }
        )
    }
}
