use pest::Span;

use crate::cst::variable_declaration::CSTVariableDeclaration;

use super::{
    elp_type::{ASTElpType, ASTMutability},
    traits::FromCST,
};

#[derive(Debug, PartialEq, Clone)]
pub struct ASTVariableDeclaration<'a> {
    pub span: &'a Span<'a>,
    pub mutability: ASTMutability,
    pub name: String,
    pub type_annotation: Option<Box<ASTElpType<'a>>>,
}

impl<'a> FromCST<'a, CSTVariableDeclaration<'a>> for ASTVariableDeclaration<'a> {
    fn from_cst(cst: &'a CSTVariableDeclaration<'a>) -> Self {
        Self {
            span: &cst.span,
            mutability: ASTMutability::from_cst(&cst.mutability),
            name: cst.name.value.clone(),
            type_annotation: cst
                .type_annotation
                .as_ref()
                .map(|boxed| ASTElpType::from_cst(&**boxed))
                .map(Box::new),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::cst::{
        elp_type::{CSTElpType, CSTElpTypeParameter, CSTElpTypeValue},
        ident::CSTIdent,
        CSTMutabilitySelector, Var,
    };

    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn variable_declaration_from_cst() {
        let expression_str = "var hello String";
        let cst = CSTVariableDeclaration {
            span: pest::Span::new(expression_str, 0, 16).unwrap(),
            mutability: CSTMutabilitySelector::Mutable(Var {
                span: Span::new(expression_str, 0, 3).unwrap(),
            }),
            name: CSTIdent {
                span: pest::Span::new(expression_str, 4, 9).unwrap(),
                value: "hello".to_string(),
            },
            type_annotation: Some(Box::new(CSTElpType {
                span: pest::Span::new(expression_str, 10, 16).unwrap(),
                mutability: None,
                pointer_semantics: None,
                value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                    span: pest::Span::new(expression_str, 10, 16).unwrap(),
                    name: CSTIdent {
                        span: pest::Span::new(expression_str, 10, 16).unwrap(),
                        value: "String".into(),
                    },
                    generics: None,
                }),
            })),
        };

        let ast = ASTVariableDeclaration::from_cst(&cst);

        assert_eq!(
            ast,
            ASTVariableDeclaration {
                span: &pest::Span::new(expression_str, 0, 16).unwrap(),
                mutability: ASTMutability::Mutable,
                name: "hello".to_string(),
                type_annotation: Some(Box::new(ASTElpType {
                    span: &pest::Span::new(expression_str, 10, 16).unwrap(),
                    mutability: ASTMutability::Immutable,
                    pointer_semantics: None,
                    name: "String".into(),
                    generic_parameters: vec![],
                    type_constraints: vec![],
                }))
            }
        )
    }
}
