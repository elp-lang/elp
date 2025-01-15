use super::elp_type::CSTElpTypeGeneric;
use super::variable_access::{CSTContextualVariableAccess, CSTPointerSemantics};
use super::{
    block::CSTBlock, elp_type::CSTElpType, expression::CSTExpression,
    variable_access::CSTVariableAccess,
};
use crate::cst::ident::CSTIdent;
use crate::parser::Rule;
use pest_ast::FromPest;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::function_argument))]
pub struct CSTFunctionArgument {
    pub pointer_semantics: Option<CSTPointerSemantics>,
    pub name: CSTIdent,
    pub type_annotation: Option<CSTElpType>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::function_arguments))]
pub struct CSTFunctionArguments {
    pub arguments: Vec<CSTFunctionArgument>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::function_return_type))]
pub struct CSTFunctionReturnType {
    pub type_annotations: Vec<CSTElpType>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::function_return_value))]
pub struct CSTFunctionReturnValue {
    pub value: Box<CSTExpression>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::function_def))]
pub struct CSTFunctionDef {
    pub name: CSTVariableAccess,
    pub generics: Option<CSTElpTypeGeneric>,
    pub arguments: Option<CSTFunctionArguments>,
    pub return_type: Option<CSTFunctionReturnType>,
    pub block: Box<CSTBlock>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::fn_header_def))]
pub struct CSTFunctionHeaderDef {
    pub pointer_semantics: Option<CSTPointerSemantics>,
    pub name: CSTVariableAccess,
    pub generics: Option<CSTElpTypeGeneric>,
    pub arguments: CSTFunctionArguments,
    pub return_type: CSTFunctionReturnType,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::function_call_name))]
pub enum CSTFunctionCallName {
    VariableAccess(CSTVariableAccess),
    ContextualVariableAccess(CSTContextualVariableAccess),
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::function_call))]
pub struct CSTFunctionCall {
    pub name: CSTFunctionCallName,
    pub generics: Option<CSTElpTypeGeneric>,
    pub arguments: Vec<CSTExpression>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cst::{
            elp_type::{CSTElpTypeParameter, CSTElpTypeValue},
            string::CSTString,
            variable_access::CSTVariableAccessNames,
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
        let ast = CSTFunctionArguments::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTFunctionArguments {
                arguments: vec![
                    CSTFunctionArgument {
                        name: CSTIdent {
                            value: "name".into()
                        },
                        pointer_semantics: None,
                        type_annotation: Some(CSTElpType {
                            mutability: None,
                            pointer_semantics: None,
                            value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                name: CSTIdent {
                                    value: "String".into()
                                },
                                generics: vec![],
                            })
                        }),
                    },
                    CSTFunctionArgument {
                        name: CSTIdent {
                            value: "hello".into()
                        },
                        pointer_semantics: None,
                        type_annotation: Some(CSTElpType {
                            mutability: None,
                            pointer_semantics: None,
                            value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                name: CSTIdent {
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
        let ast = CSTFunctionArguments::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTFunctionArguments {
                arguments: vec![
                    CSTFunctionArgument {
                        name: CSTIdent {
                            value: "self".into()
                        },
                        pointer_semantics: None,
                        type_annotation: None,
                    },
                    CSTFunctionArgument {
                        name: CSTIdent {
                            value: "name".into()
                        },
                        pointer_semantics: None,
                        type_annotation: Some(CSTElpType {
                            mutability: None,
                            pointer_semantics: None,
                            value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                name: CSTIdent {
                                    value: "String".into()
                                },
                                generics: vec![],
                            })
                        }),
                    },
                    CSTFunctionArgument {
                        name: CSTIdent {
                            value: "hello".into()
                        },
                        pointer_semantics: None,
                        type_annotation: Some(CSTElpType {
                            mutability: None,
                            pointer_semantics: None,
                            value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                name: CSTIdent {
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
        let ast = CSTFunctionReturnType::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTFunctionReturnType {
                type_annotations: vec![CSTElpType {
                    mutability: None,
                    pointer_semantics: None,
                    value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                        name: CSTIdent {
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
        let ast = CSTFunctionReturnValue::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTFunctionReturnValue {
                value: Box::new(CSTExpression::String(Box::new(CSTString {
                    value: "hello".into()
                })))
            }
        );
    }

    #[test]
    fn simple_function_def() {
        let expression_str = "fn hello.name(name String) -> String { return \"hello {name}\" }";
        let mut pairs = ElpParser::parse(Rule::function_def, expression_str).unwrap();
        let ast = CSTFunctionDef::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTFunctionDef {
                name: CSTVariableAccess {
                    names: CSTVariableAccessNames {
                        names: vec![
                            CSTIdent {
                                value: "hello".into()
                            },
                            CSTIdent {
                                value: "name".into()
                            }
                        ],
                    },
                    pointer_semantics: vec![],
                },
                generics: None,
                arguments: Some(CSTFunctionArguments {
                    arguments: vec![CSTFunctionArgument {
                        name: CSTIdent {
                            value: "name".into()
                        },
                        pointer_semantics: None,
                        type_annotation: Some(CSTElpType {
                            mutability: None,
                            pointer_semantics: None,
                            value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                name: CSTIdent {
                                    value: "String".into()
                                },
                                generics: vec![],
                            })
                        }),
                    }],
                }),
                return_type: Some(CSTFunctionReturnType {
                    type_annotations: vec![CSTElpType {
                        mutability: None,
                        pointer_semantics: None,
                        value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                            name: CSTIdent {
                                value: "String".into()
                            },
                            generics: vec![],
                        })
                    }],
                }),
                block: Box::new(CSTBlock {
                    expressions: vec![CSTExpression::FunctionReturnValue(Box::new(
                        CSTFunctionReturnValue {
                            value: Box::new(CSTExpression::String(Box::new(CSTString {
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
        let ast = CSTFunctionHeaderDef::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTFunctionHeaderDef {
                pointer_semantics: None,
                name: CSTVariableAccess {
                    names: CSTVariableAccessNames {
                        names: vec![CSTIdent {
                            value: "hello".into()
                        },],
                    },
                    pointer_semantics: vec![],
                },
                generics: None,
                arguments: CSTFunctionArguments {
                    arguments: vec![CSTFunctionArgument {
                        name: CSTIdent {
                            value: "name".into()
                        },
                        pointer_semantics: None,
                        type_annotation: Some(CSTElpType {
                            mutability: None,
                            pointer_semantics: None,
                            value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                name: CSTIdent {
                                    value: "String".into()
                                },
                                generics: vec![],
                            })
                        }),
                    }],
                },
                return_type: CSTFunctionReturnType {
                    type_annotations: vec![CSTElpType {
                        mutability: None,
                        pointer_semantics: None,
                        value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                            name: CSTIdent {
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
        let ast = CSTFunctionCall::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTFunctionCall {
                name: CSTFunctionCallName::VariableAccess(CSTVariableAccess {
                    names: CSTVariableAccessNames {
                        names: vec![
                            CSTIdent {
                                value: "hello".into()
                            },
                            CSTIdent {
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
