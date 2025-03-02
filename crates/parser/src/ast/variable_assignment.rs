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
    use super::*;
}
