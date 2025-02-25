// An AST expression is different to an CST expression in the way that a CST expression is a wider set of possible expressions that compute further down to a known AST node. There are a lot of similarities between the two but it must be considered that the CST is an incredibly brief state of the code pipeline and the AST is the first "visible" part as once we have made a pass here we will execute all precomps to refine the AST from userland to satisfy some form of Homoiconicity before moving into type safety and memory safety/ownership.

use crate::cst::expression::CSTExpression;

use super::{
    block::ASTBlock, elp_type::ASTElpType, import::ASTImport, number::ASTNumber, object::ASTObject,
    r#enum::ASTEnum, traits::FromCST,
};

#[derive(Debug, PartialEq, Clone)]
pub enum ASTExpression<'a> {
    Block(Box<ASTBlock<'a>>),
    ElpType(Box<ASTElpType<'a>>),
    Enum(Box<ASTEnum<'a>>),
    Number(Box<ASTNumber<'a>>),
    //    Export(Box<Export>),
    //    FunctionDef(Box<FunctionDef>),
    //    FunctionHeaderDef(Box<FunctionHeaderDef>),
    //    FunctionReturnValue(Box<FunctionReturnValue>),
    //    Ident(Box<Ident>),
    Import(Box<ASTImport<'a>>),
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
            CSTExpression::Enum(r#enum) => ASTExpression::Enum(Box::new(ASTEnum::from_cst(r#enum))),
            CSTExpression::Number(num) => ASTExpression::Number(Box::new(ASTNumber::from_cst(num))),
            CSTExpression::Import(import) => {
                ASTExpression::Import(Box::new(ASTImport::from_cst(import)))
            }
            _ => panic!("Invalid CST expression: {:#?}", cst),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::elp_type::ASTMutability;

    use super::*;

    #[test]
    fn ast_expression_from_cst() {
        let block_cst_expression = CSTExpression::Block(Box::new(crate::cst::block::CSTBlock {
            span: pest::Span::new("", 0, 0).unwrap(),
            expressions: vec![],
        }));
        let block_ast_expression = ASTExpression::from_cst(&block_cst_expression);

        assert_eq!(
            block_ast_expression,
            ASTExpression::Block(Box::new(ASTBlock {
                span: &pest::Span::new("", 0, 0).unwrap(),
                expressions: vec![]
            }))
        );

        let elptype_cst_expression =
            CSTExpression::ElpType(Box::new(crate::cst::elp_type::CSTElpType {
                span: pest::Span::new("", 0, 0).unwrap(),
                mutability: None,
                pointer_semantics: None,
                value: crate::cst::elp_type::CSTElpTypeValue::Parameter(
                    crate::cst::elp_type::CSTElpTypeParameter {
                        span: pest::Span::new("", 0, 0).unwrap(),
                        name: crate::cst::ident::CSTIdent {
                            span: pest::Span::new("", 0, 0).unwrap(),
                            value: "test".into(),
                        },
                        generics: None,
                    },
                ),
            }));
        let elptype_ast_expression = ASTExpression::from_cst(&elptype_cst_expression);

        assert_eq!(
            elptype_ast_expression,
            ASTExpression::ElpType(Box::new(ASTElpType {
                span: &pest::Span::new("", 0, 0).unwrap(),
                name: "test".into(),
                mutability: ASTMutability::Immutable,
                pointer_semantics: None,
                generic_parameters: vec![],
                type_constraints: vec![],
            }))
        );

        let enum_cst_expression = CSTExpression::Enum(Box::new(crate::cst::r#enum::CSTEnum {
            span: pest::Span::new("", 0, 0).unwrap(),
            name: crate::cst::ident::CSTIdent {
                span: pest::Span::new("", 0, 0).unwrap(),
                value: "test".into(),
            },
            members: vec![],
            implements: None,
        }));
        let enum_ast_expression = ASTExpression::from_cst(&enum_cst_expression);

        assert_eq!(
            enum_ast_expression,
            ASTExpression::Enum(Box::new(ASTEnum {
                span: &pest::Span::new("", 0, 0).unwrap(),
                name: "test".into(),
                members: vec![],
                implements: vec![]
            }))
        );
    }
}
