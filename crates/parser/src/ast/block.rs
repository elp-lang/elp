use pest::Span;

use crate::cst::block::CSTBlock;

use super::{expression::ASTExpression, traits::FromCST};

#[derive(Debug, PartialEq, Clone)]
pub struct ASTBlock<'a> {
    pub span: &'a Span<'a>,
    pub expressions: Vec<ASTExpression<'a>>,
}

impl<'a> FromCST<'a, CSTBlock<'a>> for ASTBlock<'a> {
    fn from_cst(cst: &'a CSTBlock) -> Self {
        ASTBlock {
            span: &cst.span,
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
                span: &pest::Span::new("", 0, 0).unwrap(),
                expressions: vec![]
            }
        )
    }
}
