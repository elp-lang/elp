use crate::cst::block::CSTBlock;

use super::{expression::ASTExpression, traits::FromCST};

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct ASTBlock {
    pub expressions: Vec<ASTExpression>,
}

impl FromCST<CSTBlock<'_>> for ASTBlock {
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
    use crate::cst;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn block_from_cst() {
        let cst_block = cst::block::CSTBlock {
            span: pest::Span::new("", 0, 0).unwrap(),
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
