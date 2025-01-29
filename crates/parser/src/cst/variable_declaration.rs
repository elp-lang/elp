use super::{elp_type::CSTElpType, ident::CSTIdent, CSTMutabilitySelector};
use crate::parser::Rule;
use pest::Span;
use pest_ast::FromPest;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::variable_declaration))]
pub struct CSTVariableDeclaration<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
    pub mutability: CSTMutabilitySelector,
    pub name: CSTIdent<'a>,
    pub type_annotation: Option<Box<CSTElpType<'a>>>,
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

    #[test]
    fn variable_declaration() {
        let expression_str = "var hello String";
        let mut pairs = ElpParser::parse(Rule::variable_declaration, expression_str).unwrap();
        let ast = CSTVariableDeclaration::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTVariableDeclaration {
                span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                mutability: CSTMutabilitySelector::Mutable(Var),
                name: CSTIdent {
                    span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                    value: "hello".to_string(),
                },
                type_annotation: Some(Box::new(CSTElpType {
                    span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                    mutability: None,
                    pointer_semantics: None,
                    value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                        span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                        name: CSTIdent {
                            span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                            value: "String".into()
                        },
                        generics: vec![],
                    })
                })),
            }
        );
    }
}
