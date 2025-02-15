// An AST expression is different to an CST expression in the way that a CST expression is a wider set of possible expressions that compute further down to a known AST node. There are a lot of similarities between the two but it must be considered that the CST is an incredibly brief state of the code pipeline and the AST is the first "visible" part as once we have made a pass here we will execute all precomps to refine the AST from userland to satisfy some form of Homoiconicity.

use crate::cst::expression::CSTExpression;

use super::{
    block::ASTBlock, elp_type::ASTElpType, object::ASTObject, r#enum::ASTEnum, traits::FromCST,
};

#[derive(Debug, PartialEq, Clone)]
pub enum ASTExpression<'a> {
    Block(Box<ASTBlock<'a>>),
    ElpType(Box<ASTElpType<'a>>),
    Enum(Box<ASTEnum<'a>>),
    //    Export(Box<Export>),
    //    FunctionDef(Box<FunctionDef>),
    //    FunctionHeaderDef(Box<FunctionHeaderDef>),
    //    FunctionReturnValue(Box<FunctionReturnValue>),
    //    Ident(Box<Ident>),
    //    Import(Box<Import>),
    //    Interface(Box<Interface>),
    //    Match(Box<MatchTree>),
    //    Number(Box<Number>),
    Object(Box<ASTObject<'a>>),
    //    PointerSemantics(Box<PointerSemantics>),
    //    String(Box<StringValue>),
    //    ValueAssignment(Box<ValueAssignment>),
    //    VariableAccess(Box<VariableAccess>),
    //    VariableAssignment(Box<VariableAssignment>),
    //    VariableDeclaration(Box<VariableDeclaration>),
}

impl<'a> FromCST<'a, CSTExpression<'a>> for ASTExpression<'a> {
    fn from_cst(cst: &'a CSTExpression) -> Self {
        match cst {
            CSTExpression::Block(block) => {
                ASTExpression::Block(Box::new(ASTBlock::from_cst(block)))
            }
            CSTExpression::ElpType(boxed_elp_type) => {
                ASTExpression::ElpType(Box::new(ASTElpType::from_cst(&**boxed_elp_type)))
            }
            CSTExpression::Object(object) => {
                ASTExpression::Object(Box::new(ASTObject::from_cst(object)))
            }
            _ => panic!("Invalid CST expression: {:#?}", cst),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ast_expression_from_cst() {
        let cst_expression = CSTExpression::Block(Box::new(crate::cst::block::CSTBlock {
            span: pest::Span::new("", 0, 0).unwrap(),
            expressions: vec![],
        }));
        let ast_expression = ASTExpression::from_cst(&cst_expression);

        assert_eq!(
            ast_expression,
            ASTExpression::Block(Box::new(ASTBlock {
                span: &pest::Span::new("", 0, 0).unwrap(),
                expressions: vec![]
            }))
        )
    }
}
