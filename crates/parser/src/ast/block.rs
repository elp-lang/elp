use crate::cst::block::CSTBlock;

use super::{expression::ASTExpression, traits::FromCST};

#[derive(Debug, PartialEq, PartialOrd)]
pub struct ASTBlock {
    pub expressions: Vec<ASTExpression>,
}

impl FromCST<CSTBlock> for ASTBlock {
    fn from_cst(cst: &CSTBlock) -> Self {
        ASTBlock {
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
        let cst_block = crate::cst::block::CSTBlock {
            expressions: vec![],
        };
        let ast_block = ASTBlock::from_cst(&cst_block);

        assert_eq!(
            ast_block,
            ASTBlock {
                expressions: vec![]
            }
        )
    }
}
