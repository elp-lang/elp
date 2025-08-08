use pest_ast::FromPest;

use crate::parser::Rule;

use super::{
    block::CSTBlock,
    elp_type::CSTElpTypeGeneric,
    function::{CSTFunctionArguments, CSTFunctionReturnType},
    variable_access::CSTVariableAccess,
};

#[derive(Debug, FromPest, PartialEq, Eq, Clone)]
#[pest_ast(rule(Rule::component_def))]
pub struct CSTComponentDef<'a> {
    #[pest_ast(outer())]
    pub span: pest::Span<'a>,
    pub name: CSTVariableAccess<'a>,
    pub generics: Option<CSTElpTypeGeneric<'a>>,
    pub arguments: Option<CSTFunctionArguments<'a>>,
    pub return_type: Option<CSTFunctionReturnType<'a>>,
    pub block: Box<CSTBlock<'a>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cst::{
            elp_type::{CSTElpType, CSTElpTypeParameter, CSTElpTypeValue},
            expression::CSTExpression,
            function::{CSTFunctionArgument, CSTFunctionReturnValue},
            ident::CSTIdent,
            string::CSTString,
            variable_access::CSTVariableAccessNames,
        },
        parser::ElpParser,
    };
    use from_pest::FromPest;
    use pest::Parser;
    use pretty_assertions::assert_eq;

    #[test]
    fn simple_component_def() {
        let expression_str =
            "component hello.name(name String) -> String { return \"hello {name}\" }";
        let mut pairs = ElpParser::parse(Rule::component_def, expression_str).unwrap();
        let ast = CSTComponentDef::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTComponentDef {
                span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                name: CSTVariableAccess {
                    span: pest::Span::new(expression_str, 10, 20).unwrap(),
                    names: CSTVariableAccessNames {
                        span: pest::Span::new(expression_str, 10, 20).unwrap(),
                        names: vec![
                            CSTIdent {
                                span: pest::Span::new(expression_str, 10, 15).unwrap(),
                                value: "hello".into()
                            },
                            CSTIdent {
                                span: pest::Span::new(expression_str, 16, 20).unwrap(),
                                value: "name".into()
                            }
                        ],
                    },
                    pointer_semantics: vec![],
                },
                generics: None,
                arguments: Some(CSTFunctionArguments {
                    span: pest::Span::new(expression_str, 20, 33).unwrap(),
                    arguments: vec![CSTFunctionArgument {
                        span: pest::Span::new(expression_str, 21, 32).unwrap(),
                        name: CSTIdent {
                            span: pest::Span::new(expression_str, 21, 25).unwrap(),
                            value: "name".into()
                        },
                        pointer_semantics: None,
                        type_annotation: Some(CSTElpType {
                            span: pest::Span::new(expression_str, 26, 32).unwrap(),
                            mutability: None,
                            pointer_semantics: None,
                            value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                span: pest::Span::new(expression_str, 26, 32).unwrap(),
                                name: CSTIdent {
                                    span: pest::Span::new(expression_str, 26, 32).unwrap(),
                                    value: "String".into()
                                },
                                generics: None
                            })
                        }),
                    }],
                }),
                return_type: Some(CSTFunctionReturnType {
                    span: pest::Span::new(expression_str, 34, 44).unwrap(),
                    type_annotations: vec![CSTElpType {
                        span: pest::Span::new(expression_str, 37, 44).unwrap(),
                        mutability: None,
                        pointer_semantics: None,
                        value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                            span: pest::Span::new(expression_str, 37, 44).unwrap(),
                            name: CSTIdent {
                                span: pest::Span::new(expression_str, 37, 43).unwrap(),
                                value: "String".into()
                            },
                            generics: None
                        })
                    }],
                }),
                block: Box::new(CSTBlock {
                    span: pest::Span::new(expression_str, 44, expression_str.len()).unwrap(),
                    expressions: vec![CSTExpression::FunctionReturnValue(Box::new(
                        CSTFunctionReturnValue {
                            span: pest::Span::new(expression_str, 46, 67).unwrap(),
                            value: Box::new(CSTExpression::String(Box::new(CSTString {
                                span: pest::Span::new(expression_str, 53, 67).unwrap(),
                                value: "hello {name}".into()
                            })))
                        }
                    ))]
                })
            }
        )
    }
}
