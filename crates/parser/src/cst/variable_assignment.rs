use super::{
    value_assignment::CSTValueAssignment, variable_access::CSTVariableAccess,
    variable_declaration::CSTVariableDeclaration,
};
use crate::parser::Rule;
use pest::Span;
use pest_ast::FromPest;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::variable_assignment_target))]
pub enum CSTVariableAssignmentTarget<'a> {
    VariableDeclaration(CSTVariableDeclaration<'a>),
    VariableAccess(CSTVariableAccess<'a>),
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::variable_assignment))]
pub struct CSTVariableAssignment<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
    pub variable_assignment_target: CSTVariableAssignmentTarget<'a>,
    pub value_assignment: CSTValueAssignment<'a>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cst::{
            expression::CSTExpression,
            ident::CSTIdent,
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
                span: pest::Span::new(expression_str, 0, 21).unwrap(),
                variable_assignment_target: CSTVariableAssignmentTarget::VariableDeclaration(
                    CSTVariableDeclaration {
                        span: pest::Span::new(expression_str, 0, 12).unwrap(),
                        mutability: CSTMutabilitySelector::Immutable(Const {
                            span: Span::new(expression_str, 0, 5).unwrap(),
                        }),
                        name: CSTIdent {
                            span: pest::Span::new(expression_str, 6, 11).unwrap(),
                            value: "hello".into()
                        },
                        type_annotation: None,
                    }
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
            }
        );
    }
}
