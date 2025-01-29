use super::{
    elp_type::{CSTElpType, CSTElpTypeGeneric},
    function::CSTFunctionHeaderDef,
    ident::CSTIdent,
};
use crate::parser::Rule;
use pest::Span;
use pest_ast::FromPest;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::interface_member_key_value))]
pub struct CSTInterfaceMemberKeyValue<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
    pub name: CSTIdent<'a>,
    pub type_annotation: Option<CSTElpType<'a>>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::interface_member))]
pub enum CSTInterfaceMember<'a> {
    Field(CSTInterfaceMemberKeyValue<'a>),
    Method(Box<CSTFunctionHeaderDef<'a>>),
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::interface_def))]
pub struct CSTInterface<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
    pub name: CSTIdent<'a>,
    pub generics: Option<CSTElpTypeGeneric<'a>>,
    pub members: Vec<CSTInterfaceMember<'a>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cst::{
            elp_type::{
                CSTElpTypeGenericConstraint, CSTElpTypeGenericParam, CSTElpTypeParameter,
                CSTElpTypeValue,
            },
            function::{CSTFunctionArgument, CSTFunctionArguments, CSTFunctionReturnType},
            variable_access::{CSTVariableAccess, CSTVariableAccessNames},
        },
        parser::ElpParser,
    };
    use from_pest::FromPest;
    use pest::{Parser, Span};
    use pretty_assertions::assert_eq;

    #[test]
    fn basic_interface_member() {
        let expression_str = ".name String";
        let mut pairs = ElpParser::parse(Rule::interface_member, expression_str).unwrap();
        let ast = CSTInterfaceMember::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTInterfaceMember::Field(CSTInterfaceMemberKeyValue {
                span: Span::new(expression_str, 0, 12).unwrap(),
                name: CSTIdent {
                    span: Span::new(expression_str, 1, 5).unwrap(),
                    value: "name".into()
                },
                type_annotation: Some(CSTElpType {
                    span: Span::new(expression_str, 6, 12).unwrap(),
                    mutability: None,
                    pointer_semantics: None,
                    value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                        span: Span::new(expression_str, 6, 12).unwrap(),
                        name: CSTIdent {
                            span: Span::new(expression_str, 6, 12).unwrap(),
                            value: "String".into()
                        },
                        generics: vec![]
                    })
                }),
            })
        );
    }

    #[test]
    fn basic_interface() {
        let expression_str = "interface Test {
  .name String
}";
        let mut pairs = ElpParser::parse(Rule::interface_def, expression_str).unwrap();
        let ast = CSTInterface::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTInterface {
                span: Span::new(expression_str, 0, expression_str.len()).unwrap(),
                name: CSTIdent {
                    span: Span::new(expression_str, 10, 14).unwrap(),
                    value: "Test".into()
                },
                generics: None,
                members: vec![CSTInterfaceMember::Field(CSTInterfaceMemberKeyValue {
                    span: Span::new(expression_str, 19, 32).unwrap(),
                    name: CSTIdent {
                        span: Span::new(expression_str, 20, 24).unwrap(),
                        value: "name".into()
                    },
                    type_annotation: Some(CSTElpType {
                        span: Span::new(expression_str, 25, 32).unwrap(),
                        mutability: None,
                        pointer_semantics: None,
                        value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                            span: Span::new(expression_str, 25, 32).unwrap(),
                            name: CSTIdent {
                                span: Span::new(expression_str, 25, 31).unwrap(),
                                value: "String".into()
                            },
                            generics: vec![]
                        })
                    }),
                })],
            }
        );
    }

    #[test]
    fn complex_interface() {
        let expression_str = "interface Into<Out, ErrorType: Error> {
            fn into<O>(self) -> Either<Out, ErrorType>
        }";
        let mut pairs = ElpParser::parse(Rule::interface_def, expression_str).unwrap();
        let ast = CSTInterface::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTInterface {
                span: Span::new(expression_str, 0, expression_str.len()).unwrap(),
                name: CSTIdent {
                    span: Span::new(expression_str, 10, 14).unwrap(),
                    value: "Into".into()
                },
                generics: Some(CSTElpTypeGeneric {
                    span: Span::new(expression_str, 14, 37).unwrap(),
                    params: vec![
                        CSTElpTypeGenericParam {
                            span: Span::new(expression_str, 15, 18).unwrap(),
                            elp_type: CSTElpType {
                                span: Span::new(expression_str, 15, 18).unwrap(),
                                mutability: None,
                                pointer_semantics: None,
                                value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                    span: Span::new(expression_str, 15, 18).unwrap(),
                                    name: CSTIdent {
                                        span: Span::new(expression_str, 15, 18).unwrap(),
                                        value: "Out".into()
                                    },
                                    generics: vec![]
                                })
                            },
                            type_constraint: None
                        },
                        CSTElpTypeGenericParam {
                            span: Span::new(expression_str, 20, 36).unwrap(),
                            elp_type: CSTElpType {
                                span: Span::new(expression_str, 20, 29).unwrap(),
                                mutability: None,
                                pointer_semantics: None,
                                value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                    span: Span::new(expression_str, 20, 29).unwrap(),
                                    name: CSTIdent {
                                        span: Span::new(expression_str, 20, 29).unwrap(),
                                        value: "ErrorType".into()
                                    },
                                    generics: vec![]
                                })
                            },
                            type_constraint: Some(CSTElpTypeGenericConstraint {
                                span: Span::new(expression_str, 29, 36).unwrap(),
                                constraints: vec![CSTElpType {
                                    span: Span::new(expression_str, 31, 36).unwrap(),
                                    mutability: None,
                                    pointer_semantics: None,
                                    value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                        span: Span::new(expression_str, 31, 36).unwrap(),
                                        name: CSTIdent {
                                            span: Span::new(expression_str, 31, 36).unwrap(),
                                            value: "Error".into()
                                        },
                                        generics: vec![]
                                    })
                                },]
                            })
                        }
                    ]
                }),
                members: vec![CSTInterfaceMember::Method(Box::new(CSTFunctionHeaderDef {
                    span: Span::new(expression_str, 52, 103).unwrap(),
                    pointer_semantics: None,
                    name: CSTVariableAccess {
                        span: Span::new(expression_str, 55, 59).unwrap(),
                        pointer_semantics: vec![],
                        names: CSTVariableAccessNames {
                            span: Span::new(expression_str, 55, 59).unwrap(),
                            names: vec![CSTIdent {
                                span: Span::new(expression_str, 55, 59).unwrap(),
                                value: "into".into()
                            }],
                        },
                    },
                    generics: Some(CSTElpTypeGeneric {
                        span: Span::new(expression_str, 59, 62).unwrap(),
                        params: vec![CSTElpTypeGenericParam {
                            span: Span::new(expression_str, 60, 61).unwrap(),
                            elp_type: CSTElpType {
                                span: Span::new(expression_str, 60, 61).unwrap(),
                                mutability: None,
                                pointer_semantics: None,
                                value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                    span: Span::new(expression_str, 60, 61).unwrap(),
                                    name: CSTIdent {
                                        span: Span::new(expression_str, 60, 61).unwrap(),
                                        value: "O".into()
                                    },
                                    generics: vec![]
                                })
                            },
                            type_constraint: None
                        },],
                    }),
                    arguments: CSTFunctionArguments {
                        span: Span::new(expression_str, 62, 68).unwrap(),
                        arguments: vec![CSTFunctionArgument {
                            span: Span::new(expression_str, 63, 67).unwrap(),
                            name: CSTIdent {
                                span: Span::new(expression_str, 63, 67).unwrap(),
                                value: "self".into()
                            },
                            pointer_semantics: None,
                            type_annotation: None,
                        }]
                    },
                    return_type: CSTFunctionReturnType {
                        span: Span::new(expression_str, 69, 103).unwrap(),
                        type_annotations: vec![CSTElpType {
                            span: Span::new(expression_str, 72, 94).unwrap(),
                            mutability: None,
                            pointer_semantics: None,
                            value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                span: Span::new(expression_str, 72, 94).unwrap(),
                                name: CSTIdent {
                                    span: Span::new(expression_str, 72, 78).unwrap(),
                                    value: "Either".into()
                                },
                                generics: vec![CSTElpTypeGeneric {
                                    span: Span::new(expression_str, 78, 94).unwrap(),
                                    params: vec![
                                        CSTElpTypeGenericParam {
                                            span: Span::new(expression_str, 79, 82,).unwrap(),
                                            elp_type: CSTElpType {
                                                span: Span::new(expression_str, 79, 82,).unwrap(),
                                                mutability: None,
                                                pointer_semantics: None,
                                                value: CSTElpTypeValue::Parameter(
                                                    CSTElpTypeParameter {
                                                        span: Span::new(expression_str, 79, 82,)
                                                            .unwrap(),
                                                        name: CSTIdent {
                                                            span:
                                                                Span::new(expression_str, 79, 82,)
                                                                    .unwrap(),
                                                            value: "Out".into()
                                                        },
                                                        generics: vec![]
                                                    }
                                                )
                                            },
                                            type_constraint: None
                                        },
                                        CSTElpTypeGenericParam {
                                            span: Span::new(expression_str, 84, 93,).unwrap(),
                                            elp_type: CSTElpType {
                                                span: Span::new(expression_str, 84, 93).unwrap(),
                                                mutability: None,
                                                pointer_semantics: None,
                                                value: CSTElpTypeValue::Parameter(
                                                    CSTElpTypeParameter {
                                                        span: Span::new(expression_str, 84, 93)
                                                            .unwrap(),
                                                        name: CSTIdent {
                                                            span: Span::new(expression_str, 84, 93)
                                                                .unwrap(),
                                                            value: "ErrorType".into()
                                                        },
                                                        generics: vec![]
                                                    }
                                                )
                                            },
                                            type_constraint: None,
                                        }
                                    ]
                                },],
                            })
                        }]
                    }
                }))]
            }
        );
    }
}
