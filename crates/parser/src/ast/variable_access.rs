use crate::parser::Rule;
use pest::Span;
use pest_ast::FromPest;

use super::IDENT;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::pointer_semantics))]
pub enum PointerSemantics {
    Pointer,
    Reference,
}

fn span_into_pointer_semantics(span: Span) -> Option<PointerSemantics> {
    match span.as_str() {
        "*" => Some(PointerSemantics::Pointer),
        "&" => Some(PointerSemantics::Reference),
        _ => None,
    }
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::variable_access))]
pub struct VariableAccess {
    #[pest_ast(inner(with(span_into_pointer_semantics)))]
    pub pointer_semantics: Option<PointerSemantics>,

    // A member chain is something like this:
    //   `foo.bar.baz` becomes `vec![IDENT { value: "foo".into() }, IDENT { value: "bar".into() }, IDENT { value: "baz".into() }]`
    pub names: Vec<IDENT>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ElpParser;
    use from_pest::FromPest;
    use pest::Parser;
    use pretty_assertions::assert_eq;

    #[test]
    fn variable_access() {
        let expression_str = "hello.world.my.name.is.dave";
        let mut pairs = ElpParser::parse(Rule::variable_access, expression_str).unwrap();
        let ast = VariableAccess::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            VariableAccess {
                names: vec![
                    IDENT {
                        value: "hello".into()
                    },
                    IDENT {
                        value: "world".into()
                    },
                    IDENT { value: "my".into() },
                    IDENT {
                        value: "name".into()
                    },
                    IDENT { value: "is".into() },
                    IDENT {
                        value: "dave".into()
                    },
                ],
                pointer_semantics: None,
            }
        )
    }
}
