use crate::parser::Rule;
use pest_ast::FromPest;

use super::{block::CSTBlock, expression::CSTExpression};

#[derive(Debug, FromPest, PartialEq, Eq, Clone)]
#[pest_ast(rule(Rule::OR))]
pub struct CSTOrLogicConditional<'a> {
    #[pest_ast(outer())]
    pub span: pest::Span<'a>,
}

#[derive(Debug, FromPest, PartialEq, Eq, Clone)]
#[pest_ast(rule(Rule::NOT))]
pub struct CSTNotLogicConditional<'a> {
    #[pest_ast(outer())]
    pub span: pest::Span<'a>,
}

#[derive(Debug, FromPest, PartialEq, Eq, Clone)]
#[pest_ast(rule(Rule::IS))]
pub struct CSTIsLogicConditional<'a> {
    #[pest_ast(outer())]
    pub span: pest::Span<'a>,
}

#[derive(Debug, FromPest, PartialEq, Eq, Clone)]
#[pest_ast(rule(Rule::AND))]
pub struct CSTAndLogicConditional<'a> {
    #[pest_ast(outer())]
    pub span: pest::Span<'a>,
}

#[derive(Debug, FromPest, PartialEq, Eq, Clone)]
#[pest_ast(rule(Rule::logic_conditionals))]
pub enum CSTLogicConditional<'a> {
    Or(CSTOrLogicConditional<'a>),
    Not(CSTNotLogicConditional<'a>),
    Is(CSTIsLogicConditional<'a>),
    And(CSTAndLogicConditional<'a>),
}

#[derive(Debug, FromPest, PartialEq, Eq, Clone)]
#[pest_ast(rule(Rule::if_tree))]
pub struct CSTIfTree<'a> {
    #[pest_ast(outer())]
    pub span: pest::Span<'a>,
    pub expressions: Vec<CSTExpression<'a>>,
    pub block: CSTBlock<'a>,
    pub elseif_trees: Vec<CSTElseIfBranch<'a>>,
    pub else_block: Option<CSTBlock<'a>>,
}

#[derive(Debug, FromPest, PartialEq, Eq, Clone)]
#[pest_ast(rule(Rule::elseif_tree))]
pub struct CSTElseIfBranch<'a> {
    #[pest_ast(outer())]
    pub span: pest::Span<'a>,
    pub expressions: Vec<CSTExpression<'a>>,
    pub block: CSTBlock<'a>,
}

#[cfg(test)]
mod tests {
    use from_pest::FromPest;
    use pest::Parser;
    use pretty_assertions::assert_eq;

    use crate::{
        cst::{
            ident::CSTIdent,
            number_value::CSTNumber,
            value_assignment::{CSTEqualityEqual, CSTOperand},
            variable_access::{CSTVariableAccess, CSTVariableAccessNames},
        },
        parser::ElpParser,
    };

    use super::*;

    #[test]
    fn if_tree_parsing_to_cst() {
        let if_tree_str = r#"if x is 1 {
				return 1
			} elseif y == 2 {
				return 2
			} else {
				return 0
			}"#;

        let mut pairs = ElpParser::parse(Rule::if_tree, if_tree_str).unwrap();
        let cst = CSTIfTree::from_pest(&mut pairs).unwrap();

        assert_eq!(
            cst,
            CSTIfTree {
                span: pest::Span::new(if_tree_str, 0, if_tree_str.len()).unwrap(),
                expressions: vec![
                    CSTExpression::VariableAccess(Box::new(CSTVariableAccess {
                        span: pest::Span::new(if_tree_str, 1, 1).unwrap(),
                        pointer_semantics: vec![],
                        names: CSTVariableAccessNames {
                            span: pest::Span::new(if_tree_str, 1, 1).unwrap(),
                            names: vec![CSTIdent {
                                span: pest::Span::new(if_tree_str, 1, 2).unwrap(),
                                value: "x".into(),
                            }],
                        }
                    })),
                    CSTExpression::Operand(Box::new(CSTOperand::EqualityEqual(CSTEqualityEqual {
                        span: pest::Span::new(if_tree_str, 1, 1).unwrap(),
                    }))),
                    CSTExpression::Number(Box::new(CSTNumber {
                        span: pest::Span::new(if_tree_str, 3, 4).unwrap(),
                        value: "1".into(),
                    })),
                ],
                block: CSTBlock {
                    span: pest::Span::new(if_tree_str, 27, if_tree_str.len()).unwrap(),
                    expressions: vec![CSTExpression::Number(Box::new(CSTNumber {
                        span: pest::Span::new(if_tree_str, 31, 32).unwrap(),
                        value: 1.to_string(),
                    }))]
                },
                elseif_trees: vec![CSTElseIfBranch {
                    span: pest::Span::new(if_tree_str, 48, if_tree_str.len()).unwrap(),
                    expressions: vec![
                        CSTExpression::VariableAccess(Box::new(CSTVariableAccess {
                            span: pest::Span::new(if_tree_str, 1, 1).unwrap(),
                            pointer_semantics: vec![],
                            names: CSTVariableAccessNames {
                                span: pest::Span::new(if_tree_str, 1, 1).unwrap(),
                                names: vec![CSTIdent {
                                    span: pest::Span::new(if_tree_str, 1, 2).unwrap(),
                                    value: "x".into(),
                                }],
                            }
                        })),
                        CSTExpression::Operand(Box::new(CSTOperand::EqualityEqual(
                            CSTEqualityEqual {
                                span: pest::Span::new(if_tree_str, 1, 1).unwrap(),
                            }
                        ))),
                        CSTExpression::Number(Box::new(CSTNumber {
                            span: pest::Span::new(if_tree_str, 3, 4).unwrap(),
                            value: "2".into(),
                        })),
                    ],
                    block: CSTBlock {
                        span: pest::Span::new(if_tree_str, 27, if_tree_str.len()).unwrap(),
                        expressions: vec![CSTExpression::Number(Box::new(CSTNumber {
                            span: pest::Span::new(if_tree_str, 31, 32).unwrap(),
                            value: 2.to_string(),
                        }))]
                    },
                }],
                else_block: Some(CSTBlock {
                    span: pest::Span::new(if_tree_str, 64, if_tree_str.len()).unwrap(),
                    expressions: vec![CSTExpression::Number(Box::new(CSTNumber {
                        span: pest::Span::new(if_tree_str, 68, 69).unwrap(),
                        value: "0".into(),
                    }))]
                })
            }
        )
    }
}
