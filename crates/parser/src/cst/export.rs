use super::expression::CSTExpression;
use crate::parser::Rule;
use pest::Span;
use pest_ast::FromPest;

#[derive(Debug, FromPest, PartialEq, Eq, Clone)]
#[pest_ast(rule(Rule::export))]
pub struct CSTExport<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
    pub expression: CSTExpression<'a>,
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        cst::{
            ident::CSTIdent, variable_declaration::CSTVariableDeclaration, CSTMutabilitySelector,
            Const,
        },
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
                span: pest::Span::new(expression_str, 0, 18).unwrap(),
                expression: CSTExpression::VariableDeclaration(Box::new(CSTVariableDeclaration {
                    span: pest::Span::new(expression_str, 7, 18).unwrap(),
                    mutability: CSTMutabilitySelector::Immutable(Const {
                        span: Span::new(expression_str, 7, 12).unwrap(),
                    }),
                    name: CSTIdent {
                        span: pest::Span::new(expression_str, 13, 18).unwrap(),
                        value: "hello".into(),
                    },
                    type_annotation: None
                }))
            }
        )
    }
}
