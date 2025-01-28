use crate::parser::Rule;
use pest::Span;
use pest_ast::FromPest;

use super::expression::CSTExpression;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::block))]
pub struct CSTBlock<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
    pub expressions: Vec<CSTExpression>,
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
                span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                expressions: vec![CSTExpression::VariableDeclaration(Box::new(
                    CSTVariableDeclaration {
                        name: "hello".into(),
                        mutability: CSTMutabilitySelector::Immutable(Const),
                        type_annotation: Some(Box::new(CSTElpType {
                            pointer_semantics: None,
                            mutability: None,
                            value: crate::cst::elp_type::CSTElpTypeValue::Parameter(
                                CSTElpTypeParameter {
                                    name: CSTIdent {
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
