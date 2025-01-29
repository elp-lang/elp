use pest::Span;
use pest_ast::FromPest;

use crate::parser::Rule;

use super::{block::CSTBlock, expression::CSTExpression};

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::for_loop))]
pub struct CSTForLoop<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
    pub declaration_expression: CSTExpression<'a>,
    pub in_expression: CSTExpression<'a>,
    pub body: CSTBlock<'a>,
}

#[cfg(test)]
mod tests {
    use crate::{
        cst::{
            function::{CSTFunctionCall, CSTFunctionCallName},
            ident::CSTIdent,
            variable_access::{CSTVariableAccess, CSTVariableAccessNames},
        },
        parser::ElpParser,
    };

    use super::*;
    use from_pest::FromPest;
    use pest::Parser;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_for_loop() {
        let expression_str = "for thing in thingies { print(thing) }";
        let mut pairs = ElpParser::parse(Rule::for_loop, expression_str).unwrap();
        let ast = CSTForLoop::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTForLoop {
                span: pest::Span::new(expression_str, 0, 38).unwrap(),
                declaration_expression: CSTExpression::VariableAccess(Box::new(
                    CSTVariableAccess {
                        span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                        names: CSTVariableAccessNames {
                            span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                            names: vec![CSTIdent {
                                span: pest::Span::new(expression_str, 0, expression_str.len())
                                    .unwrap(),
                                value: "thing".into()
                            }],
                        },
                        pointer_semantics: vec![],
                    }
                )),
                in_expression: CSTExpression::VariableAccess(Box::new(CSTVariableAccess {
                    span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                    names: CSTVariableAccessNames {
                        span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                        names: vec![CSTIdent {
                            span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                            value: "thingies".into()
                        }],
                    },
                    pointer_semantics: vec![],
                })),
                body: CSTBlock {
                    span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                    expressions: vec![CSTExpression::FunctionCall(Box::new(CSTFunctionCall {
                        span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                        name: CSTFunctionCallName::VariableAccess(CSTVariableAccess {
                            span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                            pointer_semantics: vec![],
                            names: CSTVariableAccessNames {
                                span: pest::Span::new(expression_str, 0, expression_str.len())
                                    .unwrap(),
                                names: vec![CSTIdent {
                                    span: pest::Span::new(expression_str, 0, expression_str.len())
                                        .unwrap(),
                                    value: "print".into()
                                }],
                            },
                        }),
                        generics: None,
                        arguments: vec![CSTExpression::VariableAccess(Box::new(
                            CSTVariableAccess {
                                span: pest::Span::new(expression_str, 0, expression_str.len())
                                    .unwrap(),
                                names: CSTVariableAccessNames {
                                    span: pest::Span::new(expression_str, 0, expression_str.len())
                                        .unwrap(),
                                    names: vec![CSTIdent {
                                        span: pest::Span::new(
                                            expression_str,
                                            0,
                                            expression_str.len()
                                        )
                                        .unwrap(),
                                        value: "thing".into()
                                    }],
                                },
                                pointer_semantics: vec![],
                            }
                        ))],
                    }))]
                }
            }
        )
    }
}
