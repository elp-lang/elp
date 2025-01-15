use super::{
    value_assignment::CSTValueAssignment, variable_access::CSTVariableAccess,
    variable_declaration::CSTVariableDeclaration,
};
use crate::parser::Rule;
use pest_ast::FromPest;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::variable_assignment_target))]
pub enum CSTVariableAssignmentTarget {
    VariableDeclaration(CSTVariableDeclaration),
    VariableAccess(CSTVariableAccess),
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::variable_assignment))]
pub struct CSTVariableAssignment {
    pub variable_assignment_target: CSTVariableAssignmentTarget,
    pub value_assignment: CSTValueAssignment,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cst::{
            expression::CSTExpression,
            string::CSTString,
            value_assignment::{CSTEquals, CSTOperand},
            CSTMutabilitySelector, Const,
        },
        parser::ElpParser,
    };
    use from_pest::FromPest;
    use pest::Parser;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_variable_assignments() {
        let expression_str = "const hello = \"world\"";
        let mut pairs = ElpParser::parse(Rule::variable_assignment, expression_str).unwrap();
        let ast = CSTVariableAssignment::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTVariableAssignment {
                variable_assignment_target: CSTVariableAssignmentTarget::VariableDeclaration(
                    CSTVariableDeclaration {
                        mutability: CSTMutabilitySelector::Immutable(Const),
                        name: "hello".into(),
                        type_annotation: None,
                    }
                ),
                value_assignment: CSTValueAssignment {
                    operand: CSTOperand::Equals(CSTEquals {}),
                    value: Box::new(CSTExpression::String(Box::new(CSTString {
                        value: "world".into(),
                    }))),
                },
            }
        );
    }
}
