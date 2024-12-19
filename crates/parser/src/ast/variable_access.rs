use crate::parser::Rule;
use pest::Span;
use pest_ast::FromPest;

use super::IDENT;

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
        println!("{:#?}", pairs);
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
                }
            }
        )
    }
}
