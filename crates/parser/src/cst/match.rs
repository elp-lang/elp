use crate::parser::Rule;
use pest_ast::FromPest;

use super::{block::Block, expression::CSTExpression};

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::match_tree))]
pub struct MatchTree {
    pub match_expression: CSTExpression,
    pub match_arms: Vec<MatchTreeArm>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::match_arm_subject))]
pub enum MatchArmSubject {
    Expression(CSTExpression),
    MatchRange(MatchRange),
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::match_range))]
pub struct MatchRange {
    pub range_start: Option<Box<CSTExpression>>,
    pub range_end: Option<Box<CSTExpression>>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::match_arm_body))]
pub enum MatchBody {
    Expression(Box<CSTExpression>),
    Block(Box<Block>),
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::match_arm))]
pub struct MatchTreeArm {
    pub subject: MatchArmSubject,
    pub body: MatchBody,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cst::{
            function::FunctionReturnValue,
            ident::Ident,
            number_value::Number,
            string::StringValue,
            variable_access::{VariableAccess, VariableAccessNames},
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
        let ast = MatchTreeArm::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            MatchTreeArm {
                subject: MatchArmSubject::Expression(CSTExpression::VariableAccess(Box::new(
                    VariableAccess {
                        pointer_semantics: vec![],
                        names: VariableAccessNames {
                            names: vec![Ident {
                                value: "expr".into()
                            }],
                        },
                    }
                ))),
                body: MatchBody::Block(Box::new(Block {
                    expressions: vec![CSTExpression::String(Box::new(StringValue {
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
        let ast = MatchArmSubject::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            MatchArmSubject::Expression(CSTExpression::VariableAccess(Box::new(VariableAccess {
                pointer_semantics: vec![],
                names: VariableAccessNames {
                    names: vec![Ident {
                        value: "expr".into()
                    }],
                },
            })))
        );

        let expression_range = "1..10";
        let mut pairs = ElpParser::parse(Rule::match_arm_subject, expression_range).unwrap();
        let ast = MatchArmSubject::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            MatchArmSubject::MatchRange(MatchRange {
                range_start: Some(Box::new(CSTExpression::Number(Box::new(Number {
                    value: "1".into()
                })))),
                range_end: Some(Box::new(CSTExpression::Number(Box::new(Number {
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
        let ast = MatchTree::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            MatchTree {
                match_expression: CSTExpression::VariableAccess(Box::new(VariableAccess {
                    pointer_semantics: vec![],
                    names: VariableAccessNames {
                        names: vec![Ident {
                            value: "expr".into()
                        }],
                    }
                })),
                match_arms: vec![
                    MatchTreeArm {
                        subject: MatchArmSubject::Expression(CSTExpression::VariableAccess(
                            Box::new(VariableAccess {
                                pointer_semantics: vec![],
                                names: VariableAccessNames {
                                    names: vec![Ident {
                                        value: "expr".into()
                                    }],
                                },
                            })
                        )),
                        body: MatchBody::Expression(Box::new(CSTExpression::String(Box::new(
                            StringValue {
                                value: "Hello, World!".into()
                            }
                        ))))
                    },
                    MatchTreeArm {
                        subject: MatchArmSubject::Expression(CSTExpression::VariableAccess(
                            Box::new(VariableAccess {
                                pointer_semantics: vec![],
                                names: VariableAccessNames {
                                    names: vec![Ident { value: "_".into() }],
                                },
                            })
                        )),
                        body: MatchBody::Block(Box::new(Block {
                            expressions: vec![CSTExpression::FunctionReturnValue(Box::new(
                                FunctionReturnValue {
                                    value: Box::new(CSTExpression::String(Box::new(StringValue {
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
