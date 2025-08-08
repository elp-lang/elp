use lexer::span::CodeSpan;

use super::expression::CSTExpression;

#[derive(Debug, PartialEq, Clone)]
pub struct CSTBlock<'a> {
    pub span: CodeSpan,
    pub expressions: Vec<CSTExpression<'a>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cst::{
            CSTMutabilitySelector, Const,
            elp_type::{CSTElpType, CSTElpTypeParameter},
            ident::CSTIdent,
            variable_declaration::CSTVariableDeclaration,
        },
        parser::ElpParser,
    };
    use pretty_assertions::assert_eq;

    #[test]
    fn blocks() {
        let expression_str = "{ const hello String }";
        let mut pairs = ElpParser::parse(expression_str);
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
                                    generics: None
                                }
                            )
                        })),
                    }
                ))]
            }
        )
    }
}
