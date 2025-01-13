use crate::cst::block::Block as CSTBlock;

use super::{expression::ASTExpression, traits::FromCST};

#[derive(Debug, PartialEq, Eq)]
pub struct Block {
    pub expressions: Vec<ASTExpression>,
}

impl FromCST<CSTBlock> for Block {
    fn from_cst(cst: &CSTBlock) -> Self {
        Block {
            expressions: cst
                .expressions
                .iter()
                .map(ASTExpression::from_cst)
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_from_cst() {
        let cst_block = crate::cst::block::Block {
            expressions: vec![],
        };
        let ast_block = Block::from_cst(&cst_block);

        assert_eq!(
            ast_block,
            Block {
                expressions: vec![]
            }
        )
    }
}
