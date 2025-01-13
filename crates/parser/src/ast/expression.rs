// An AST expression is different to an CST expression in the way that a CST expression is a wider set of possible expressions that compute further down to a known AST node. There are a lot of similarities between the two but it must be considered that the CST is an incredibly brief state of the code pipeline and the AST is the first "visible" part as once we have made a pass here we will execute all precomps to refine the AST from userland to satisfy some form of Homoiconicity.

use crate::cst::expression::CSTExpression;

use super::{block::Block, traits::FromCST};

#[derive(Debug, PartialEq, Eq)]
pub enum ASTExpression {
    Block(Box<Block>),
    //    ElpType(Box<ElpType>),
    //    Enum(Box<Enum>),
    //    Export(Box<Export>),
    //    FunctionDef(Box<FunctionDef>),
    //    FunctionHeaderDef(Box<FunctionHeaderDef>),
    //    FunctionReturnValue(Box<FunctionReturnValue>),
    //    Ident(Box<Ident>),
    //    Import(Box<Import>),
    //    Interface(Box<Interface>),
    //    Match(Box<MatchTree>),
    //    Number(Box<Number>),
    //    Object(Box<Object>),
    //    PointerSemantics(Box<PointerSemantics>),
    //    String(Box<StringValue>),
    //    ValueAssignment(Box<ValueAssignment>),
    //    VariableAccess(Box<VariableAccess>),
    //    VariableAssignment(Box<VariableAssignment>),
    //    VariableDeclaration(Box<VariableDeclaration>),
}

impl FromCST<CSTExpression> for ASTExpression {
    fn from_cst(cst: &CSTExpression) -> Self {
        match cst {
            CSTExpression::Block(block) => ASTExpression::Block(Box::new(Block::from_cst(block))),
            _ => panic!("Invalid CST expression: {:#?}", cst),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ast_expression_from_cst() {
        let cst_expression = CSTExpression::Block(Box::new(crate::cst::block::Block {
            expressions: vec![],
        }));
        let ast_expression = ASTExpression::from_cst(&cst_expression);

        assert_eq!(
            ast_expression,
            ASTExpression::Block(Box::new(Block {
                expressions: vec![]
            }))
        )
    }
}
