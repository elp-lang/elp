use super::{elp_type::CSTElpType, span_into_string, CSTMutabilitySelector};
use crate::parser::Rule;
use pest_ast::FromPest;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::variable_declaration))]
pub struct CSTVariableDeclaration {
    pub mutability: CSTMutabilitySelector,

    #[pest_ast(inner(with(span_into_string)))]
    pub name: String,

    pub type_annotation: Option<Box<CSTElpType>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cst::{
            elp_type::{CSTElpTypeParameter, CSTElpTypeValue},
            ident::CSTIdent,
            CSTMutabilitySelector, Var,
        },
        parser::ElpParser,
    };
    use from_pest::FromPest;
    use pest::Parser;
    use pretty_assertions::assert_eq;

    #[test]
    fn variable_declaration() {
        let expression_str = "var hello String";
        let mut pairs = ElpParser::parse(Rule::variable_declaration, expression_str).unwrap();
        let ast = CSTVariableDeclaration::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTVariableDeclaration {
                mutability: CSTMutabilitySelector::Mutable(Var),
                name: "hello".to_string(),
                type_annotation: Some(Box::new(CSTElpType {
                    mutability: None,
                    pointer_semantics: None,
                    value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                        name: CSTIdent {
                            value: "String".into()
                        },
                        generics: vec![],
                    })
                })),
            }
        );
    }
}
