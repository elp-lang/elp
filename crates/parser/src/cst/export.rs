use super::expression::CSTExpression;
use crate::parser::Rule;
use pest_ast::FromPest;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::export))]
pub struct CSTExport {
    pub expression: CSTExpression,
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        cst::{variable_declaration::CSTVariableDeclaration, CSTMutabilitySelector, Const},
        parser::ElpParser,
    };
    use from_pest::FromPest;
    use pest::Parser;
    use pretty_assertions::assert_eq;

    #[test]
    fn parse_export_expression() {
        let expression_str = "export const hello";
        let mut pairs = ElpParser::parse(Rule::export, expression_str).unwrap();
        let ast = CSTExport::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTExport {
                expression: CSTExpression::VariableDeclaration(Box::new(CSTVariableDeclaration {
                    mutability: CSTMutabilitySelector::Immutable(Const),
                    name: "hello".into(),
                    type_annotation: None
                }))
            }
        )
    }
}
