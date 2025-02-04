use crate::parser::Rule;
use pest::Span;
use pest_ast::FromPest;

use super::{block::CSTBlock, expression::CSTExpression};

#[derive(Debug, FromPest, PartialEq, Eq, Clone)]
#[pest_ast(rule(Rule::match_tree))]
pub struct CSTMatchTree<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
    pub match_expression: CSTExpression<'a>,
    pub match_arms: Vec<CSTMatchTreeArm<'a>>,
}

#[derive(Debug, FromPest, PartialEq, Eq, Clone)]
#[pest_ast(rule(Rule::match_arm_subject))]
pub enum CSTMatchArmSubject<'a> {
    Expression(CSTExpression<'a>),
    MatchRange(CSTMatchRange<'a>),
}

#[derive(Debug, FromPest, PartialEq, Eq, Clone)]
#[pest_ast(rule(Rule::match_range))]
pub struct CSTMatchRange<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
    pub range_start: Option<Box<CSTExpression<'a>>>,
    pub range_end: Option<Box<CSTExpression<'a>>>,
}

#[derive(Debug, FromPest, PartialEq, Eq, Clone)]
#[pest_ast(rule(Rule::match_arm_body))]
pub enum CSTMatchBody<'a> {
    Expression(Box<CSTExpression<'a>>),
    Block(Box<CSTBlock<'a>>),
}

#[derive(Debug, FromPest, PartialEq, Eq, Clone)]
#[pest_ast(rule(Rule::match_arm))]
pub struct CSTMatchTreeArm<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
    pub subject: CSTMatchArmSubject<'a>,
    pub body: CSTMatchBody<'a>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cst::{
            function::CSTFunctionReturnValue,
            ident::CSTIdent,
            number_value::CSTNumber,
            string::CSTString,
            variable_access::{CSTVariableAccess, CSTVariableAccessNames},
        },
        parser::ElpParser,
    };
    use from_pest::FromPest;
    use pest::Parser;
    use pretty_assertions::assert_eq;

    #[test]
    fn match_tree_arm() {
        let expression_str = "expr -> {
\"Hello, World!\"
}";
        let mut pairs = ElpParser::parse(Rule::match_arm, expression_str).unwrap();
        let ast = CSTMatchTreeArm::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTMatchTreeArm {
                span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                subject: CSTMatchArmSubject::Expression(CSTExpression::VariableAccess(Box::new(
                    CSTVariableAccess {
                        span: pest::Span::new(expression_str, 0, 5).unwrap(),
                        pointer_semantics: vec![],
                        names: CSTVariableAccessNames {
                            span: pest::Span::new(expression_str, 0, 5).unwrap(),
                            names: vec![CSTIdent {
                                span: pest::Span::new(expression_str, 0, 4).unwrap(),
                                value: "expr".into()
                            }],
                        },
                    }
                ))),
                body: CSTMatchBody::Block(Box::new(CSTBlock {
                    span: pest::Span::new(expression_str, 8, expression_str.len()).unwrap(),
                    expressions: vec![CSTExpression::String(Box::new(CSTString {
                        span: pest::Span::new(expression_str, 10, 25).unwrap(),
                        value: "Hello, World!".into()
                    }))]
                }))
            }
        )
    }

    #[test]
    fn match_arm_subjects() {
        let expression_ident = "expr";
        let mut pairs = ElpParser::parse(Rule::match_arm_subject, expression_ident).unwrap();
        let ast = CSTMatchArmSubject::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTMatchArmSubject::Expression(CSTExpression::VariableAccess(Box::new(
                CSTVariableAccess {
                    span: pest::Span::new(expression_ident, 0, expression_ident.len()).unwrap(),
                    pointer_semantics: vec![],
                    names: CSTVariableAccessNames {
                        span: pest::Span::new(expression_ident, 0, expression_ident.len()).unwrap(),
                        names: vec![CSTIdent {
                            span: pest::Span::new(expression_ident, 0, expression_ident.len())
                                .unwrap(),
                            value: "expr".into()
                        }],
                    },
                }
            )))
        );

        let expression_range = "1..10";
        let mut pairs = ElpParser::parse(Rule::match_arm_subject, expression_range).unwrap();
        let ast = CSTMatchArmSubject::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTMatchArmSubject::MatchRange(CSTMatchRange {
                span: pest::Span::new(expression_range, 0, expression_range.len()).unwrap(),
                range_start: Some(Box::new(CSTExpression::Number(Box::new(CSTNumber {
                    span: pest::Span::new(expression_range, 0, 1).unwrap(),
                    value: "1".into()
                })))),
                range_end: Some(Box::new(CSTExpression::Number(Box::new(CSTNumber {
                    span: pest::Span::new(expression_range, 3, expression_range.len()).unwrap(),
                    value: "10".into()
                })))),
            })
        )
    }

    #[test]
    fn match_tree() {
        let expression_str = "match expr {
            expr -> \"Hello, World!\",
            _ -> {
                return \"Default value\"
            }
        }";
        let mut pairs = ElpParser::parse(Rule::match_tree, expression_str).unwrap();
        let ast = CSTMatchTree::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTMatchTree {
                span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                match_expression: CSTExpression::VariableAccess(Box::new(CSTVariableAccess {
                    span: pest::Span::new(expression_str, 6, 11).unwrap(),
                    pointer_semantics: vec![],
                    names: CSTVariableAccessNames {
                        span: pest::Span::new(expression_str, 6, 11).unwrap(),
                        names: vec![CSTIdent {
                            span: pest::Span::new(expression_str, 6, 10).unwrap(),
                            value: "expr".into()
                        }],
                    }
                })),
                match_arms: vec![
                    CSTMatchTreeArm {
                        span: pest::Span::new(expression_str, 25, 49).unwrap(),
                        subject: CSTMatchArmSubject::Expression(CSTExpression::VariableAccess(
                            Box::new(CSTVariableAccess {
                                span: pest::Span::new(expression_str, 25, 30).unwrap(),
                                pointer_semantics: vec![],
                                names: CSTVariableAccessNames {
                                    span: pest::Span::new(expression_str, 25, 30).unwrap(),
                                    names: vec![CSTIdent {
                                        span: pest::Span::new(expression_str, 25, 29).unwrap(),
                                        value: "expr".into()
                                    }],
                                },
                            })
                        )),
                        body: CSTMatchBody::Expression(Box::new(CSTExpression::String(Box::new(
                            CSTString {
                                span: pest::Span::new(expression_str, 33, 48).unwrap(),
                                value: "Hello, World!".into()
                            }
                        ))))
                    },
                    CSTMatchTreeArm {
                        span: pest::Span::new(expression_str, 62, 130).unwrap(),
                        subject: CSTMatchArmSubject::Expression(CSTExpression::VariableAccess(
                            Box::new(CSTVariableAccess {
                                span: pest::Span::new(expression_str, 62, 64).unwrap(),
                                pointer_semantics: vec![],
                                names: CSTVariableAccessNames {
                                    span: pest::Span::new(expression_str, 62, 64).unwrap(),
                                    names: vec![CSTIdent {
                                        span: pest::Span::new(expression_str, 62, 63).unwrap(),
                                        value: "_".into()
                                    }],
                                },
                            })
                        )),
                        body: CSTMatchBody::Block(Box::new(CSTBlock {
                            span: pest::Span::new(expression_str, 67, 121).unwrap(),
                            expressions: vec![CSTExpression::FunctionReturnValue(Box::new(
                                CSTFunctionReturnValue {
                                    span: pest::Span::new(expression_str, 85, 107).unwrap(),
                                    value: Box::new(CSTExpression::String(Box::new(CSTString {
                                        span: pest::Span::new(expression_str, 92, 107).unwrap(),
                                        value: "Default value".into()
                                    })))
                                }
                            ))]
                        }))
                    }
                ]
            }
        )
    }
}
