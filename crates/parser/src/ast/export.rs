use super::expression::Expression;
use crate::parser::Rule;
use pest_ast::FromPest;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::export))]
pub struct Export {
    pub expression: Expression,
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        ast::variable_declaration::{VariableDeclaration, VariableMutability},
        parser::ElpParser,
    };
    use from_pest::FromPest;
    use pest::Parser;
    use pretty_assertions::assert_eq;

    #[test]
    fn parse_export_expression() {
        let expression_str = "export const hello";
        let mut pairs = ElpParser::parse(Rule::expression, expression_str).unwrap();
        let ast = Expression::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            Expression::Export(Box::new(Export {
                expression: Expression::VariableDeclaration(Box::new(VariableDeclaration {
                    mutability: VariableMutability::Immutable,
                    name: "hello".into(),
                    type_annotation: None
                }))
            }))
        )
    }
}
