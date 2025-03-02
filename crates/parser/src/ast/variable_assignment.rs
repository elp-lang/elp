use pest::Span;

use crate::cst::variable_assignment::{CSTVariableAssignment, CSTVariableAssignmentTarget};

use super::{
    traits::FromCST, value_assignment::ASTValueAssignment, variable_access::ASTVariableAccess,
    variable_declaration::ASTVariableDeclaration,
};

#[derive(Debug, PartialEq, Clone)]
pub enum ASTVariableAssignmentTarget<'a> {
    VariableDeclaration(ASTVariableDeclaration<'a>),
    VariableAccess(ASTVariableAccess<'a>),
}

impl<'a> FromCST<'a, CSTVariableAssignmentTarget<'a>> for ASTVariableAssignmentTarget<'a> {
    fn from_cst(cst: &'a CSTVariableAssignmentTarget<'a>) -> Self {
        match cst {
            CSTVariableAssignmentTarget::VariableDeclaration(declaration) => {
                ASTVariableAssignmentTarget::VariableDeclaration(ASTVariableDeclaration::from_cst(
                    declaration,
                ))
            }
            CSTVariableAssignmentTarget::VariableAccess(access) => {
                ASTVariableAssignmentTarget::VariableAccess(ASTVariableAccess::from_cst(access))
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ASTVariableAssignment<'a> {
    pub span: &'a Span<'a>,
    pub variable_assignment_target: ASTVariableAssignmentTarget<'a>,
    pub value_assignment: ASTValueAssignment<'a>,
}

impl<'a> FromCST<'a, CSTVariableAssignment<'a>> for ASTVariableAssignment<'a> {
    fn from_cst(cst: &'a CSTVariableAssignment<'a>) -> Self {
        Self {
            span: &cst.span,
            variable_assignment_target: ASTVariableAssignmentTarget::from_cst(
                &cst.variable_assignment_target,
            ),
            value_assignment: ASTValueAssignment::from_cst(&cst.value_assignment),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{
            elp_type::ASTMutability, expression::ASTExpression, string::ASTString,
            value_assignment::ASTOperand,
        },
        cst::{
            expression::CSTExpression,
            ident::CSTIdent,
            string::CSTString,
            value_assignment::{CSTEquals, CSTOperand, CSTValueAssignment},
            variable_declaration::CSTVariableDeclaration,
            CSTMutabilitySelector, Const,
        },
    };

    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn variable_assignment_from_cst() {
        let expression_str = "const hello = \"world\"";
        let cst = CSTVariableAssignment {
            span: pest::Span::new(expression_str, 0, 21).unwrap(),
            variable_assignment_target: CSTVariableAssignmentTarget::VariableDeclaration(
                CSTVariableDeclaration {
                    span: pest::Span::new(expression_str, 0, 12).unwrap(),
                    mutability: CSTMutabilitySelector::Immutable(Const {
                        span: Span::new(expression_str, 0, 5).unwrap(),
                    }),
                    name: CSTIdent {
                        span: pest::Span::new(expression_str, 6, 11).unwrap(),
                        value: "hello".into(),
                    },
                    type_annotation: None,
                },
            ),
            value_assignment: CSTValueAssignment {
                span: pest::Span::new(expression_str, 12, 21).unwrap(),
                operand: CSTOperand::Equals(CSTEquals {
                    span: pest::Span::new(expression_str, 12, 13).unwrap(),
                }),
                value: Box::new(CSTExpression::String(Box::new(CSTString {
                    span: pest::Span::new(expression_str, 14, 21).unwrap(),
                    value: "world".into(),
                }))),
            },
        };

        let ast = ASTVariableAssignment::from_cst(&cst);

        assert_eq!(
            ast,
            ASTVariableAssignment {
                span: &pest::Span::new(expression_str, 0, 21).unwrap(),
                variable_assignment_target: ASTVariableAssignmentTarget::VariableDeclaration(
                    ASTVariableDeclaration {
                        span: &pest::Span::new(expression_str, 0, 12).unwrap(),
                        mutability: ASTMutability::Immutable,
                        name: "hello".into(),
                        type_annotation: None,
                    }
                ),
                value_assignment: ASTValueAssignment {
                    span: &pest::Span::new(expression_str, 12, 21).unwrap(),
                    operand: ASTOperand::Equals,
                    value: Box::new(ASTExpression::String(Box::new(ASTString {
                        span: &pest::Span::new(expression_str, 14, 21).unwrap(),
                        value: "world".into(),
                    })))
                }
            }
        )
    }
}
