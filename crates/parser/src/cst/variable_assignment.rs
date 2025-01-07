use super::{
    value_assignment::ValueAssignment, variable_access::VariableAccess,
    variable_declaration::VariableDeclaration,
};
use crate::parser::Rule;
use pest_ast::FromPest;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::variable_assignment_target))]
pub enum VariableAssignmentTarget {
    VariableDeclaration(VariableDeclaration),
    VariableAccess(VariableAccess),
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::variable_assignment))]
pub struct VariableAssignment {
    pub variable_assignment_target: VariableAssignmentTarget,
    pub value_assignment: ValueAssignment,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cst::{
            expression::Expression,
            string::StringValue,
            value_assignment::{Equals, Operand},
            Const, MutabilitySelector,
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
        let ast = VariableAssignment::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            VariableAssignment {
                variable_assignment_target: VariableAssignmentTarget::VariableDeclaration(
                    VariableDeclaration {
                        mutability: MutabilitySelector::Immutable(Const),
                        name: "hello".into(),
                        type_annotation: None,
                    }
                ),
                value_assignment: ValueAssignment {
                    operand: Operand::Equals(Equals {}),
                    value: Box::new(Expression::String(Box::new(StringValue {
                        value: "world".into(),
                    }))),
                },
            }
        );
    }
}
