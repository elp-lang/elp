use super::{
    block::Block, elp_type::ElpType, expression::Expression, variable_access::VariableAccess,
};
use crate::ast::ident::Ident;
use crate::parser::Rule;
use pest_ast::FromPest;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::function_argument))]
pub struct FunctionArgument {
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
    pub value: Box<Expression>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::function_def))]
pub struct FunctionDef {
    pub name: VariableAccess,
    pub arguments: Option<FunctionArguments>,
    pub return_type: Option<FunctionReturnType>,
    pub block: Box<Block>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ast::{string::StringValue, variable_access::VariableAccessNames},
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
                        type_annotation: Some(ElpType {
                            name: "String".into(),
                            type_parameters: vec![],
                        }),
                    },
                    FunctionArgument {
                        name: Ident {
                            value: "hello".into()
                        },
                        type_annotation: Some(ElpType {
                            name: "String".into(),
                            type_parameters: vec![],
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
                        type_annotation: None,
                    },
                    FunctionArgument {
                        name: Ident {
                            value: "name".into()
                        },
                        type_annotation: Some(ElpType {
                            name: "String".into(),
                            type_parameters: vec![],
                        }),
                    },
                    FunctionArgument {
                        name: Ident {
                            value: "hello".into()
                        },
                        type_annotation: Some(ElpType {
                            name: "String".into(),
                            type_parameters: vec![],
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
                    name: "String".into(),
                    type_parameters: vec![],
                }]
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
                arguments: Some(FunctionArguments {
                    arguments: vec![FunctionArgument {
                        name: Ident {
                            value: "name".into()
                        },
                        type_annotation: Some(ElpType {
                            name: "String".into(),
                            type_parameters: vec![],
                        }),
                    }],
                }),
                return_type: Some(FunctionReturnType {
                    type_annotations: vec![ElpType {
                        name: "String".into(),
                        type_parameters: vec![],
                    }],
                }),
                block: Box::new(Block {
                    expressions: vec![Expression::FunctionReturnValue(Box::new(
                        FunctionReturnValue {
                            value: Box::new(Expression::String(Box::new(StringValue {
                                value: "hello {name}".into()
                            })))
                        }
                    ))]
                })
            }
        )
    }
}
