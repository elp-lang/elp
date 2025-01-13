use super::{elp_type::ElpType, span_into_string, MutabilitySelector};
use crate::parser::Rule;
use pest_ast::FromPest;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::variable_declaration))]
pub struct VariableDeclaration {
    pub mutability: MutabilitySelector,

    #[pest_ast(inner(with(span_into_string)))]
    pub name: String,

    pub type_annotation: Option<Box<ElpType>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cst::{
            elp_type::{ElpTypeParameter, ElpTypeValue},
            ident::Ident,
            MutabilitySelector, Var,
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
        let ast = VariableDeclaration::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            VariableDeclaration {
                mutability: MutabilitySelector::Mutable(Var),
                name: "hello".to_string(),
                type_annotation: Some(Box::new(ElpType {
                    mutability: None,
                    pointer_semantics: None,
                    value: ElpTypeValue::Parameter(ElpTypeParameter {
                        name: Ident {
                            value: "String".into()
                        },
                        generics: vec![],
                    })
                })),
            }
        );
    }
}
