use crate::parser::Rule;
use pest_ast::FromPest;

use super::{block::CSTBlock, expression::CSTExpression};

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::match_tree))]
pub struct CSTMatchTree {
    pub match_expression: CSTExpression,
    pub match_arms: Vec<CSTMatchTreeArm>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::match_arm_subject))]
pub enum CSTMatchArmSubject {
    Expression(CSTExpression),
    MatchRange(CSTMatchRange),
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::match_range))]
pub struct CSTMatchRange {
    pub range_start: Option<Box<CSTExpression>>,
    pub range_end: Option<Box<CSTExpression>>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::match_arm_body))]
pub enum CSTMatchBody {
    Expression(Box<CSTExpression>),
    Block(Box<CSTBlock>),
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::match_arm))]
pub struct CSTMatchTreeArm {
    pub subject: CSTMatchArmSubject,
    pub body: CSTMatchBody,
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
                subject: CSTMatchArmSubject::Expression(CSTExpression::VariableAccess(Box::new(
                    CSTVariableAccess {
                        pointer_semantics: vec![],
                        names: CSTVariableAccessNames {
                            names: vec![CSTIdent {
                                value: "expr".into()
                            }],
                        },
                    }
                ))),
                body: CSTMatchBody::Block(Box::new(CSTBlock {
                    expressions: vec![CSTExpression::String(Box::new(CSTString {
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
                    pointer_semantics: vec![],
                    names: CSTVariableAccessNames {
                        names: vec![CSTIdent {
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
                range_start: Some(Box::new(CSTExpression::Number(Box::new(CSTNumber {
                    value: "1".into()
                })))),
                range_end: Some(Box::new(CSTExpression::Number(Box::new(CSTNumber {
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
                match_expression: CSTExpression::VariableAccess(Box::new(CSTVariableAccess {
                    pointer_semantics: vec![],
                    names: CSTVariableAccessNames {
                        names: vec![CSTIdent {
                            value: "expr".into()
                        }],
                    }
                })),
                match_arms: vec![
                    CSTMatchTreeArm {
                        subject: CSTMatchArmSubject::Expression(CSTExpression::VariableAccess(
                            Box::new(CSTVariableAccess {
                                pointer_semantics: vec![],
                                names: CSTVariableAccessNames {
                                    names: vec![CSTIdent {
                                        value: "expr".into()
                                    }],
                                },
                            })
                        )),
                        body: CSTMatchBody::Expression(Box::new(CSTExpression::String(Box::new(
                            CSTString {
                                value: "Hello, World!".into()
                            }
                        ))))
                    },
                    CSTMatchTreeArm {
                        subject: CSTMatchArmSubject::Expression(CSTExpression::VariableAccess(
                            Box::new(CSTVariableAccess {
                                pointer_semantics: vec![],
                                names: CSTVariableAccessNames {
                                    names: vec![CSTIdent { value: "_".into() }],
                                },
                            })
                        )),
                        body: CSTMatchBody::Block(Box::new(CSTBlock {
                            expressions: vec![CSTExpression::FunctionReturnValue(Box::new(
                                CSTFunctionReturnValue {
                                    value: Box::new(CSTExpression::String(Box::new(CSTString {
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
