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
    pub type_annotation: ElpType,
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

//#[cfg(test)]
//mod tests {
//    use super::*;
//    use crate::{ast::StringValue, parser::ElpParser};
//    use from_pest::FromPest;
//    use pest::Parser;
//    use pretty_assertions::assert_eq;
//
//    #[test]
//    fn simple_function_def() {
//        let expression_str = "fn hello.name(name String) -> String { return \"hello {name}\" }";
//        let mut pairs = ElpParser::parse(Rule::function_def, expression_str).unwrap();
//        let ast = FunctionDef::from_pest(&mut pairs).unwrap();
//
//        assert_eq!(
//            ast,
//            FunctionDef {
//                name: VariableAccess {
//                    variable_name: "hello".into(),
//                    pointer_semantics: None,
//                    member_chain: vec![IDENT {
//                        value: "name".into()
//                    }]
//                },
//                arguments: Some(FunctionArguments {
//                    arguments: vec![FunctionArgument {
//                        name: IDENT {
//                            value: "name".into()
//                        },
//                        type_annotation: Some(ElpType {
//                            name: "String".into(),
//                            type_parameters: None,
//                        }),
//                    }],
//                }),
//                return_type: Some(FunctionReturnType {
//                    type_annotation: ElpType {
//                        name: "String".into(),
//                        type_parameters: None,
//                    },
//                }),
//                block: Box::new(Block {
//                    expressions: vec![Expression::FunctionReturnValue(Box::new(
//                        FunctionReturnValue {
//                            value: Box::new(Expression::String(Box::new(StringValue {
//                                value: "hello {name}".into()
//                            })))
//                        }
//                    ))]
//                })
//            }
//        )
//    }
//}
