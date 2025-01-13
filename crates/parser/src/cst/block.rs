use crate::parser::Rule;
use pest_ast::FromPest;

use super::expression::CSTExpression;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::block))]
pub struct Block {
    pub expressions: Vec<CSTExpression>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cst::{
            elp_type::{ElpType, ElpTypeParameter},
            ident::Ident,
            variable_declaration::VariableDeclaration,
            Const, MutabilitySelector,
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
        let ast = Block::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            Block {
                expressions: vec![CSTExpression::VariableDeclaration(Box::new(
                    VariableDeclaration {
                        name: "hello".into(),
                        mutability: MutabilitySelector::Immutable(Const),
                        type_annotation: Some(Box::new(ElpType {
                            pointer_semantics: None,
                            mutability: None,
                            value: crate::cst::elp_type::ElpTypeValue::Parameter(
                                ElpTypeParameter {
                                    name: Ident {
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
