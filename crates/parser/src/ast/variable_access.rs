use crate::ast::ident::Ident;
use crate::parser::Rule;
use pest::Span;
use pest_ast::FromPest;

fn span_into_pointer_semantics(span: Span) -> PointerSemanticsType {
    match span.as_str() {
        "*" => PointerSemanticsType::Pointer,
        _ => PointerSemanticsType::Reference,
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum PointerSemanticsType {
    Pointer,
    Reference,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::pointer_semantics))]
pub struct PointerSemantics {
    #[pest_ast(inner(with(span_into_pointer_semantics)))]
    pub semantics: PointerSemanticsType,
    pub variable_access: VariableAccess,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::variable_access))]
pub struct VariableAccess {
    pub names: Vec<Ident>,
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
        println!("{:#?}", pairs);
        let ast = VariableAccess::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            VariableAccess {
                names: vec![
                    Ident {
                        value: "hello".into()
                    },
                    Ident {
                        value: "world".into()
                    },
                    Ident { value: "my".into() },
                    Ident {
                        value: "name".into()
                    },
                    Ident { value: "is".into() },
                    Ident {
                        value: "dave".into()
                    },
                ],
            }
        )
    }

    #[test]
    fn variable_access_reference() {
        let expression_str = "&hello.world.my.name.is.dave";
        let mut pairs = ElpParser::parse(Rule::pointer_semantics, expression_str).unwrap();
        println!("{:#?}", pairs);
        let ast = PointerSemantics::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            PointerSemantics {
                semantics: PointerSemanticsType::Reference,
                variable_access: VariableAccess {
                    names: vec![
                        Ident {
                            value: "hello".into()
                        },
                        Ident {
                            value: "world".into()
                        },
                        Ident { value: "my".into() },
                        Ident {
                            value: "name".into()
                        },
                        Ident { value: "is".into() },
                        Ident {
                            value: "dave".into()
                        },
                    ],
                }
            }
        )
    }
}
