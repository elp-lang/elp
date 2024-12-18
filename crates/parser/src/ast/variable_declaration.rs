use super::{elp_type::ElpType, span_into_string};
use crate::parser::Rule;
use pest::Span;
use pest_ast::FromPest;

#[derive(Debug, PartialEq, Eq)]
pub enum VariableMutability {
    Mutable,
    Immutable,
}

fn span_into_mutability_selector(span: Span) -> VariableMutability {
    match span.as_str() {
        "var" => VariableMutability::Mutable,
        "const" => VariableMutability::Immutable,
        _ => panic!("Invalid variable declaration mutability: {}", span.as_str()),
    }
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::variable_declaration))]
pub struct VariableDeclaration {
    #[pest_ast(inner(with(span_into_mutability_selector)))]
    pub mutability: VariableMutability,

    #[pest_ast(inner(with(span_into_string)))]
    pub name: String,

    pub type_annotation: Option<Box<ElpType>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ElpParser;
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
                mutability: VariableMutability::Mutable,
                name: "hello".to_string(),
                type_annotation: Some(Box::new(ElpType {
                    name: "String".to_string(),
                    type_parameters: Some([].into()),
                })),
            }
        );
    }
}
