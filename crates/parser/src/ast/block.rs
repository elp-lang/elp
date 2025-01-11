use crate::cst::expression::CSTExpression;

use super::{expression::ASTExpression, traits::FromCST};

pub struct Block {
    pub expressions: Vec<ASTExpression>,
}

impl FromCST for Block {
    fn from_cst(cst: &CSTExpression) -> Self {
        match cst {
            CSTExpression::Block(block) => Block {
                expressions: block
                    .expressions
                    .iter()
                    .map(ASTExpression::from_cst)
                    .collect(),
            },
            _ => panic!("Expected a block"),
        }
    }
}
