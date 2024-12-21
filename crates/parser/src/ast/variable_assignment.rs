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
    pub variable_assignment_target: VariableAssignmentTarget,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ast::{expression::Expression, Eoi, Module},
        parser::ElpParser,
    };
    use from_pest::FromPest;
    use pest::Parser;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_pointer_semantics() {
        let ref_expression_str = "const hello = \"world\"";
        let mut ref_pairs = ElpParser::parse(Rule::pointer_semantics, ref_expression_str).unwrap();
        let ref_ast = VariableAssignment::from_pest(&mut ref_pairs).unwrap();

        assert_eq!(ref_ast, VariableAssignment {});
    }
}
