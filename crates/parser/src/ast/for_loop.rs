use crate::cst::for_loop::CSTForLoop;

use super::{block::ASTBlock, expression::ASTExpression, traits::FromCST};

#[derive(Debug, PartialEq, Clone)]
pub struct ASTForLoop<'a> {
    pub span: &'a pest::Span<'a>,
    pub declaration_expression: ASTExpression<'a>,
    pub in_expression: ASTExpression<'a>,
    pub body: ASTBlock<'a>,
}

impl<'a> FromCST<'a, CSTForLoop<'a>> for ASTForLoop<'a> {
    fn from_cst(cst: &'a CSTForLoop<'a>) -> Self {
        Self {
            span: &cst.span,
            body: ASTBlock::from_cst(&cst.body),
            in_expression: ASTExpression::from_cst(&cst.in_expression),
            declaration_expression: ASTExpression::from_cst(&cst.declaration_expression),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::variable_access::{ASTVariableAccess, ASTVariableAccessNames},
        cst::{
            block::CSTBlock,
            expression::CSTExpression,
            function::{CSTFunctionCall, CSTFunctionCallName},
            ident::CSTIdent,
            variable_access::{CSTVariableAccess, CSTVariableAccessNames},
        },
    };

    use super::*;

    #[test]
    fn ast_for_loop() {
        let expression_str = "for thing in thingies { print(thing) }";
        let cst = CSTForLoop {
            span: pest::Span::new(expression_str, 0, 38).unwrap(),
            declaration_expression: CSTExpression::VariableAccess(Box::new(CSTVariableAccess {
                span: pest::Span::new(expression_str, 4, 10).unwrap(),
                names: CSTVariableAccessNames {
                    span: pest::Span::new(expression_str, 4, 10).unwrap(),
                    names: vec![CSTIdent {
                        span: pest::Span::new(expression_str, 4, 9).unwrap(),
                        value: "thing".into(),
                    }],
                },
                pointer_semantics: vec![],
            })),
            in_expression: CSTExpression::VariableAccess(Box::new(CSTVariableAccess {
                span: pest::Span::new(expression_str, 13, 22).unwrap(),
                names: CSTVariableAccessNames {
                    span: pest::Span::new(expression_str, 13, 22).unwrap(),
                    names: vec![CSTIdent {
                        span: pest::Span::new(expression_str, 13, 21).unwrap(),
                        value: "thingies".into(),
                    }],
                },
                pointer_semantics: vec![],
            })),
            body: CSTBlock {
                span: pest::Span::new(expression_str, 22, 38).unwrap(),
                expressions: vec![CSTExpression::FunctionCall(Box::new(CSTFunctionCall {
                    span: pest::Span::new(expression_str, 24, 36).unwrap(),
                    name: CSTFunctionCallName::VariableAccess(CSTVariableAccess {
                        span: pest::Span::new(expression_str, 24, 29).unwrap(),
                        pointer_semantics: vec![],
                        names: CSTVariableAccessNames {
                            span: pest::Span::new(expression_str, 24, 29).unwrap(),
                            names: vec![CSTIdent {
                                span: pest::Span::new(expression_str, 24, 29).unwrap(),
                                value: "print".into(),
                            }],
                        },
                    }),
                    generics: None,
                    arguments: vec![CSTExpression::VariableAccess(Box::new(CSTVariableAccess {
                        span: pest::Span::new(expression_str, 30, 35).unwrap(),
                        names: CSTVariableAccessNames {
                            span: pest::Span::new(expression_str, 30, 35).unwrap(),
                            names: vec![CSTIdent {
                                span: pest::Span::new(expression_str, 30, 35).unwrap(),
                                value: "thing".into(),
                            }],
                        },
                        pointer_semantics: vec![],
                    }))],
                }))],
            },
        };

        let ast = ASTForLoop::from_cst(&cst);

        assert_eq!(
            ast,
            ASTForLoop {
                span: &pest::Span::new(expression_str, 0, 38).unwrap(),
                declaration_expression: ASTExpression::VariableAccess(Box::new(
                    ASTVariableAccess {
                        span: &pest::Span::new(expression_str, 4, 10).unwrap(),
                        names: ASTVariableAccessNames {
                            span: &pest::Span::new(expression_str, 4, 10).unwrap(),
                            names: vec!["thing".into()],
                        },
                        pointer_semantics: vec![],
                    }
                )),
                in_expression: ASTExpression::VariableAccess(Box::new(ASTVariableAccess {
                    span: &pest::Span::new(expression_str, 13, 22).unwrap(),
                    names: ASTVariableAccessNames {
                        span: &pest::Span::new(expression_str, 13, 22).unwrap(),
                        names: vec!["thingies".into()],
                    },
                    pointer_semantics: vec![],
                })),
                body: ASTBlock {
                    span: &pest::Span::new(expression_str, 22, 38).unwrap(),
                    expressions: vec![ASTExpression::FunctionCall(Box::new(ASTFunctionCall {
                        span: &pest::Span::new(expression_str, 24, 36).unwrap(),
                        name: ASTFunctionCallName::VariableAccess(ASTVariableAccess {
                            span: &pest::Span::new(expression_str, 24, 29).unwrap(),
                            pointer_semantics: vec![],
                            names: ASTVariableAccessNames {
                                span: &pest::Span::new(expression_str, 24, 29).unwrap(),
                                names: vec!["print".into()],
                            },
                        }),
                        generics: None,
                        arguments: vec![CSTExpression::VariableAccess(Box::new(
                            ASTVariableAccess {
                                span: &pest::Span::new(expression_str, 30, 35).unwrap(),
                                names: ASTVariableAccessNames {
                                    span: &pest::Span::new(expression_str, 30, 35).unwrap(),
                                    names: vec!["thing".into()],
                                },
                                pointer_semantics: vec![],
                            }
                        ))],
                    }))],
                },
            }
        )
    }
}
