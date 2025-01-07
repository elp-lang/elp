use super::{
    elp_type::{ElpType, ElpTypeGeneric},
    function::ExternalFunctionDef,
    ident::Ident,
};
use crate::parser::Rule;
use pest_ast::FromPest;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::interface_member_key_value))]
pub struct InterfaceMemberKeyValue {
    pub name: Ident,
    pub type_annotation: Option<ElpType>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::interface_member))]
pub enum InterfaceMember {
    Field(InterfaceMemberKeyValue),
    Method(ExternalFunctionDef),
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::interface_def))]
pub struct Interface {
    pub name: Ident,
    pub generics: Option<ElpTypeGeneric>,
    pub members: Vec<InterfaceMember>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cst::{
            elp_type::{ElpTypeGenericConstraint, ElpTypeGenericParam},
            function::{FunctionArgument, FunctionArguments, FunctionReturnType},
            variable_access::{VariableAccess, VariableAccessNames},
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
        let ast = InterfaceMember::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            InterfaceMember::Field(InterfaceMemberKeyValue {
                name: Ident {
                    value: "name".into()
                },
                type_annotation: Some(ElpType {
                    mutability: None,
                    name: Ident {
                        value: "String".into()
                    },
                    generics: vec![]
                }),
            })
        );
    }

    #[test]
    fn basic_interface() {
        let expression_str = "interface Test {.name String}";
        let mut pairs = ElpParser::parse(Rule::interface_def, expression_str).unwrap();
        let ast = Interface::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            Interface {
                name: Ident {
                    value: "Test".into()
                },
                generics: None,
                members: vec![InterfaceMember::Field(InterfaceMemberKeyValue {
                    name: Ident {
                        value: "name".into()
                    },
                    type_annotation: Some(ElpType {
                        mutability: None,
                        name: Ident {
                            value: "String".into()
                        },
                        generics: vec![]
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
        let ast = Interface::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            Interface {
                name: Ident {
                    value: "Into".into()
                },
                generics: Some(ElpTypeGeneric {
                    params: vec![
                        ElpTypeGenericParam {
                            elp_type: ElpType {
                                mutability: None,
                                name: Ident {
                                    value: "Out".into()
                                },
                                generics: vec![]
                            },
                            type_constraint: None
                        },
                        ElpTypeGenericParam {
                            elp_type: ElpType {
                                mutability: None,
                                name: Ident {
                                    value: "ErrorType".into()
                                },
                                generics: vec![]
                            },
                            type_constraint: Some(ElpTypeGenericConstraint {
                                constraints: vec![ElpType {
                                    mutability: None,
                                    name: Ident {
                                        value: "Error".into()
                                    },
                                    generics: vec![]
                                },]
                            })
                        }
                    ]
                }),
                members: vec![InterfaceMember::Method(ExternalFunctionDef {
                    name: VariableAccess {
                        pointer_semantics: vec![],
                        names: VariableAccessNames {
                            names: vec![Ident {
                                value: "into".into()
                            }],
                        },
                    },
                    generics: Some(ElpTypeGeneric {
                        params: vec![ElpTypeGenericParam {
                            elp_type: ElpType {
                                mutability: None,
                                name: Ident { value: "O".into() },
                                generics: vec![]
                            },
                            type_constraint: None
                        },],
                    }),
                    arguments: FunctionArguments {
                        arguments: vec![FunctionArgument {
                            name: Ident {
                                value: "self".into()
                            },
                            type_annotation: None,
                        }]
                    },
                    return_type: FunctionReturnType {
                        type_annotations: vec![ElpType {
                            mutability: None,
                            name: Ident {
                                value: "Either".into()
                            },
                            generics: vec![ElpTypeGeneric {
                                params: vec![
                                    ElpTypeGenericParam {
                                        elp_type: ElpType {
                                            mutability: None,
                                            name: Ident {
                                                value: "Out".into()
                                            },
                                            generics: vec![]
                                        },
                                        type_constraint: None
                                    },
                                    ElpTypeGenericParam {
                                        elp_type: ElpType {
                                            mutability: None,
                                            name: Ident {
                                                value: "ErrorType".into()
                                            },
                                            generics: vec![]
                                        },
                                        type_constraint: None,
                                    }
                                ]
                            },],
                        }]
                    }
                })]
            }
        );
    }
}
