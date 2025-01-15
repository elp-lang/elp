use super::{
    elp_type::{CSTElpType, CSTElpTypeGeneric},
    function::CSTFunctionHeaderDef,
    ident::CSTIdent,
};
use crate::parser::Rule;
use pest_ast::FromPest;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::interface_member_key_value))]
pub struct CSTInterfaceMemberKeyValue {
    pub name: CSTIdent,
    pub type_annotation: Option<CSTElpType>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::interface_member))]
pub enum CSTInterfaceMember {
    Field(CSTInterfaceMemberKeyValue),
    Method(CSTFunctionHeaderDef),
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::interface_def))]
pub struct CSTInterface {
    pub name: CSTIdent,
    pub generics: Option<CSTElpTypeGeneric>,
    pub members: Vec<CSTInterfaceMember>,
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
    use pest::Parser;
    use pretty_assertions::assert_eq;

    #[test]
    fn basic_interface_member() {
        let expression_str = ".name String";
        let mut pairs = ElpParser::parse(Rule::interface_member, expression_str).unwrap();
        let ast = CSTInterfaceMember::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTInterfaceMember::Field(CSTInterfaceMemberKeyValue {
                name: CSTIdent {
                    value: "name".into()
                },
                type_annotation: Some(CSTElpType {
                    mutability: None,
                    pointer_semantics: None,
                    value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                        name: CSTIdent {
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
        let expression_str = "interface Test {.name String}";
        let mut pairs = ElpParser::parse(Rule::interface_def, expression_str).unwrap();
        let ast = CSTInterface::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTInterface {
                name: CSTIdent {
                    value: "Test".into()
                },
                generics: None,
                members: vec![CSTInterfaceMember::Field(CSTInterfaceMemberKeyValue {
                    name: CSTIdent {
                        value: "name".into()
                    },
                    type_annotation: Some(CSTElpType {
                        mutability: None,
                        pointer_semantics: None,
                        value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                            name: CSTIdent {
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
                name: CSTIdent {
                    value: "Into".into()
                },
                generics: Some(CSTElpTypeGeneric {
                    params: vec![
                        CSTElpTypeGenericParam {
                            elp_type: CSTElpType {
                                mutability: None,
                                pointer_semantics: None,
                                value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                    name: CSTIdent {
                                        value: "Out".into()
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
                                        value: "ErrorType".into()
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
                                            value: "Error".into()
                                        },
                                        generics: vec![]
                                    })
                                },]
                            })
                        }
                    ]
                }),
                members: vec![CSTInterfaceMember::Method(CSTFunctionHeaderDef {
                    pointer_semantics: None,
                    name: CSTVariableAccess {
                        pointer_semantics: vec![],
                        names: CSTVariableAccessNames {
                            names: vec![CSTIdent {
                                value: "into".into()
                            }],
                        },
                    },
                    generics: Some(CSTElpTypeGeneric {
                        params: vec![CSTElpTypeGenericParam {
                            elp_type: CSTElpType {
                                mutability: None,
                                pointer_semantics: None,
                                value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                    name: CSTIdent { value: "O".into() },
                                    generics: vec![]
                                })
                            },
                            type_constraint: None
                        },],
                    }),
                    arguments: CSTFunctionArguments {
                        arguments: vec![CSTFunctionArgument {
                            name: CSTIdent {
                                value: "self".into()
                            },
                            pointer_semantics: None,
                            type_annotation: None,
                        }]
                    },
                    return_type: CSTFunctionReturnType {
                        type_annotations: vec![CSTElpType {
                            mutability: None,
                            pointer_semantics: None,
                            value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                name: CSTIdent {
                                    value: "Either".into()
                                },
                                generics: vec![CSTElpTypeGeneric {
                                    params: vec![
                                        CSTElpTypeGenericParam {
                                            elp_type: CSTElpType {
                                                mutability: None,
                                                pointer_semantics: None,
                                                value: CSTElpTypeValue::Parameter(
                                                    CSTElpTypeParameter {
                                                        name: CSTIdent {
                                                            value: "Out".into()
                                                        },
                                                        generics: vec![]
                                                    }
                                                )
                                            },
                                            type_constraint: None
                                        },
                                        CSTElpTypeGenericParam {
                                            elp_type: CSTElpType {
                                                mutability: None,
                                                pointer_semantics: None,
                                                value: CSTElpTypeValue::Parameter(
                                                    CSTElpTypeParameter {
                                                        name: CSTIdent {
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
                })]
            }
        );
    }
}
