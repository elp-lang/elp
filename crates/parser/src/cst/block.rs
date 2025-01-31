use crate::parser::Rule;
use pest::Span;
use pest_ast::FromPest;

use super::expression::CSTExpression;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::block))]
pub struct CSTBlock<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
    pub expressions: Vec<CSTExpression<'a>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cst::{
            elp_type::{CSTElpType, CSTElpTypeParameter},
            ident::CSTIdent,
            variable_declaration::CSTVariableDeclaration,
            CSTMutabilitySelector, Const,
        },
        parser::ElpParser,
    };
    use from_pest::FromPest;
    use pest::Parser;
    use pretty_assertions::assert_eq;

    #[test]
    fn blocks() {
        let expression_str = "{ const hello String }";
        let mut pairs = ElpParser::parse(Rule::block, expression_str).unwrap();
        let ast = CSTBlock::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTBlock {
                span: pest::Span::new(expression_str, 0, 22).unwrap(),
                expressions: vec![CSTExpression::VariableDeclaration(Box::new(
                    CSTVariableDeclaration {
                        span: pest::Span::new(expression_str, 2, 21).unwrap(),
                        name: CSTIdent {
                            span: pest::Span::new(expression_str, 8, 13).unwrap(),
                            value: "hello".into()
                        },
                        mutability: CSTMutabilitySelector::Immutable(Const {
                            span: Span::new(expression_str, 2, 7).unwrap()
                        }),
                        type_annotation: Some(Box::new(CSTElpType {
                            span: pest::Span::new(expression_str, 14, 21).unwrap(),
                            pointer_semantics: None,
                            mutability: None,
                            value: crate::cst::elp_type::CSTElpTypeValue::Parameter(
                                CSTElpTypeParameter {
                                    span: pest::Span::new(expression_str, 14, 21).unwrap(),
                                    name: CSTIdent {
                                        span: pest::Span::new(expression_str, 14, 20).unwrap(),
                                        value: "String".into()
                                    },
                                    generics: vec![]
                                }
                            )
                        })),
                    }
                ))]
            }
        )
    }
}
