use super::elp_type::CSTElpTypeGeneric;
use super::variable_access::{CSTContextualVariableAccess, CSTPointerSemantics};
use super::{
    block::CSTBlock, elp_type::CSTElpType, expression::CSTExpression,
    variable_access::CSTVariableAccess,
};
use crate::cst::ident::CSTIdent;
use crate::parser::Rule;
use pest::Span;
use pest_ast::FromPest;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::function_argument))]
pub struct CSTFunctionArgument<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
    pub pointer_semantics: Option<CSTPointerSemantics<'a>>,
    pub name: CSTIdent<'a>,
    pub type_annotation: Option<CSTElpType<'a>>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::function_arguments))]
pub struct CSTFunctionArguments<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
    pub arguments: Vec<CSTFunctionArgument<'a>>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::function_return_type))]
pub struct CSTFunctionReturnType<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
    pub type_annotations: Vec<CSTElpType<'a>>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::function_return_value))]
pub struct CSTFunctionReturnValue<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
    pub value: Box<CSTExpression<'a>>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::function_def))]
pub struct CSTFunctionDef<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
    pub name: CSTVariableAccess<'a>,
    pub generics: Option<CSTElpTypeGeneric<'a>>,
    pub arguments: Option<CSTFunctionArguments<'a>>,
    pub return_type: Option<CSTFunctionReturnType<'a>>,
    pub block: Box<CSTBlock<'a>>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::fn_header_def))]
pub struct CSTFunctionHeaderDef<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
    pub pointer_semantics: Option<CSTPointerSemantics<'a>>,
    pub name: CSTVariableAccess<'a>,
    pub generics: Option<CSTElpTypeGeneric<'a>>,
    pub arguments: CSTFunctionArguments<'a>,
    pub return_type: CSTFunctionReturnType<'a>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::function_call_name))]
pub enum CSTFunctionCallName<'a> {
    VariableAccess(CSTVariableAccess<'a>),
    ContextualVariableAccess(CSTContextualVariableAccess<'a>),
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::function_call))]
pub struct CSTFunctionCall<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
    pub name: CSTFunctionCallName<'a>,
    pub generics: Option<CSTElpTypeGeneric<'a>>,
    pub arguments: Vec<CSTExpression<'a>>,
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
                span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                arguments: vec![
                    CSTFunctionArgument {
                        span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                        name: CSTIdent {
                            span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                            value: "name".into()
                        },
                        pointer_semantics: None,
                        type_annotation: Some(CSTElpType {
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
                                generics: vec![],
                            })
                        }),
                    },
                    CSTFunctionArgument {
                        span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                        name: CSTIdent {
                            span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                            value: "hello".into()
                        },
                        pointer_semantics: None,
                        type_annotation: Some(CSTElpType {
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
                span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                arguments: vec![
                    CSTFunctionArgument {
                        span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                        name: CSTIdent {
                            span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                            value: "self".into()
                        },
                        pointer_semantics: None,
                        type_annotation: None,
                    },
                    CSTFunctionArgument {
                        span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                        name: CSTIdent {
                            span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                            value: "name".into()
                        },
                        pointer_semantics: None,
                        type_annotation: Some(CSTElpType {
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
                                generics: vec![],
                            })
                        }),
                    },
                    CSTFunctionArgument {
                        span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                        name: CSTIdent {
                            span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                            value: "hello".into()
                        },
                        pointer_semantics: None,
                        type_annotation: Some(CSTElpType {
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
                span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                type_annotations: vec![CSTElpType {
                    span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                    mutability: None,
                    pointer_semantics: None,
                    value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                        span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                        name: CSTIdent {
                            span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
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
                span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                value: Box::new(CSTExpression::String(Box::new(CSTString {
                    span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
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
                span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                name: CSTVariableAccess {
                    span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                    names: CSTVariableAccessNames {
                        span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                        names: vec![
                            CSTIdent {
                                span: pest::Span::new(expression_str, 0, expression_str.len())
                                    .unwrap(),
                                value: "hello".into()
                            },
                            CSTIdent {
                                span: pest::Span::new(expression_str, 0, expression_str.len())
                                    .unwrap(),
                                value: "name".into()
                            }
                        ],
                    },
                    pointer_semantics: vec![],
                },
                generics: None,
                arguments: Some(CSTFunctionArguments {
                    span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                    arguments: vec![CSTFunctionArgument {
                        span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                        name: CSTIdent {
                            span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                            value: "name".into()
                        },
                        pointer_semantics: None,
                        type_annotation: Some(CSTElpType {
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
                                generics: vec![],
                            })
                        }),
                    }],
                }),
                return_type: Some(CSTFunctionReturnType {
                    span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                    type_annotations: vec![CSTElpType {
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
                            generics: vec![],
                        })
                    }],
                }),
                block: Box::new(CSTBlock {
                    span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                    expressions: vec![CSTExpression::FunctionReturnValue(Box::new(
                        CSTFunctionReturnValue {
                            span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                            value: Box::new(CSTExpression::String(Box::new(CSTString {
                                span: pest::Span::new(expression_str, 0, expression_str.len())
                                    .unwrap(),
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
                span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                pointer_semantics: None,
                name: CSTVariableAccess {
                    span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                    names: CSTVariableAccessNames {
                        span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                        names: vec![CSTIdent {
                            span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                            value: "hello".into()
                        },],
                    },
                    pointer_semantics: vec![],
                },
                generics: None,
                arguments: CSTFunctionArguments {
                    span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                    arguments: vec![CSTFunctionArgument {
                        span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                        name: CSTIdent {
                            span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                            value: "name".into()
                        },
                        pointer_semantics: None,
                        type_annotation: Some(CSTElpType {
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
                                generics: vec![],
                            })
                        }),
                    }],
                },
                return_type: CSTFunctionReturnType {
                    span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                    type_annotations: vec![CSTElpType {
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
                span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                name: CSTFunctionCallName::VariableAccess(CSTVariableAccess {
                    span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                    names: CSTVariableAccessNames {
                        span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                        names: vec![
                            CSTIdent {
                                span: pest::Span::new(expression_str, 0, expression_str.len())
                                    .unwrap(),
                                value: "hello".into()
                            },
                            CSTIdent {
                                span: pest::Span::new(expression_str, 0, expression_str.len())
                                    .unwrap(),
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
