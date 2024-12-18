use super::{variable_access::VariableAccess, variable_declaration::VariableDeclaration};
use crate::parser::Rule;
use pest_ast::FromPest;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::variable_assignment_target))]
pub enum VariableAssignmentTarget {
    #[pest_ast(rule(Rule::variable_declaration))]
    VariableDeclaration(Box<VariableDeclaration>),

    #[pest_ast(rule(Rule::variable_access))]
    VariableAccess(Box<VariableAccess>),
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::variable_assignment))]
pub struct VariableAssignment {
    pub variable_declaration: VariableDeclaration,
}
