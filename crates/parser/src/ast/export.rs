use crate::cst::export::CSTExport;

use super::{expression::ASTExpression, traits::FromCST};

#[derive(Debug, PartialEq, Clone)]
pub struct ASTExport<'a> {
    pub span: &'a pest::Span<'a>,
    pub value: ASTExpression<'a>,
}

impl<'a> FromCST<'a, CSTExport<'a>> for ASTExport<'a> {
    fn from_cst(cst: &'a CSTExport<'a>) -> Self {
        Self {
            span: &cst.span,
            value: ASTExpression::from_cst(&cst.expression),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::cst::{
        export::CSTExport, expression::CSTExpression, ident::CSTIdent,
        variable_declaration::CSTVariableDeclaration, Const,
    };

    use super::*;

    #[test]
    fn export_ast() {
        let expression_str = "export const a string";
        let cst = CSTExport {
            span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
            expression: CSTExpression::VariableDeclaration(Box::new(CSTVariableDeclaration {
                span: pest::Span::new(expression_str, 7, 20).unwrap(),
                name: CSTIdent {
                    span: pest::Span::new(expression_str, 15, 16).unwrap(),
                    value: "a".into(),
                },
                mutability: crate::cst::CSTMutabilitySelector::Immutable(Const {
                    span: pest::Span::new(expression_str, 8, 12).unwrap(),
                }),
                type_annotation: None,
            })),
        };

        let ast = ASTExport::from_cst(&cst);

        assert_eq!(
            ast,
            ASTExport {
                span: &pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                value: ASTExpression::Variable(Box::new(ASTVariable {
                    span: pest::Span::new(expression_str, 7, 20).unwrap(),
                    name: CSTIdent {
                        span: pest::Span::new(expression_str, 15, 16).unwrap(),
                        value: "a".into(),
                    },
                })),
            }
        );
    }
}
