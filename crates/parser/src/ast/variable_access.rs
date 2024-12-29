use crate::ast::ident::Ident;
use crate::parser::Rule;
use pest_ast::FromPest;

// Not a fan of having anonymous structs for these rules to fit into the enum
// but it is what it is.
#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::POINTER))]
pub struct Pointer;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::REFERENCE))]
pub struct Reference;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::pointer_semantics))]
pub enum PointerSemantics {
    Pointer(Pointer),
    Reference(Reference),
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::variable_access))]
pub struct VariableAccess {
    pub pointer_semantics: Vec<PointerSemantics>,
    pub names: VariableAccessNames,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::variable_access_names))]
pub struct VariableAccessNames {
    pub names: Vec<Ident>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::variable_access::VariableAccess;
    use crate::parser::ElpParser;
    use from_pest::FromPest;
    use pest::Parser;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_pointer_semantics() {
        let ref_expression_str = "&";
        let mut ref_pairs = ElpParser::parse(Rule::pointer_semantics, ref_expression_str).unwrap();
        let ref_ast = PointerSemantics::from_pest(&mut ref_pairs).unwrap();

        assert_eq!(ref_ast, PointerSemantics::Reference(Reference {}));

        let ptr_expression_str = "*";
        let mut ptr_pairs = ElpParser::parse(Rule::pointer_semantics, ptr_expression_str).unwrap();
        let ptr_ast = PointerSemantics::from_pest(&mut ptr_pairs).unwrap();

        assert_eq!(ptr_ast, PointerSemantics::Pointer(Pointer {}));
    }

    #[test]
    fn variable_access() {
        let expression_str = "hello.world.my.name.is.dave";
        let mut pairs = ElpParser::parse(Rule::variable_access, expression_str).unwrap();
        let ast = VariableAccess::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            VariableAccess {
                pointer_semantics: vec![],
                names: VariableAccessNames {
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

    #[test]
    fn variable_access_with_pointer_semantics() {
        let expression_str_reference = "&hello.world.my.name.is.dave";
        let mut pairs_reference =
            ElpParser::parse(Rule::variable_access, expression_str_reference).unwrap();
        let reference_ast = VariableAccess::from_pest(&mut pairs_reference).unwrap();

        assert_eq!(
            reference_ast,
            VariableAccess {
                pointer_semantics: vec![PointerSemantics::Reference(Reference {})],
                names: VariableAccessNames {
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
        );

        let expression_str_pointer = "*hello.world.my.name.is.dave";
        let mut pairs_pointer =
            ElpParser::parse(Rule::variable_access, expression_str_pointer).unwrap();
        let pointer_ast = VariableAccess::from_pest(&mut pairs_pointer).unwrap();

        assert_eq!(
            pointer_ast,
            VariableAccess {
                pointer_semantics: vec![PointerSemantics::Pointer(Pointer {})],
                names: VariableAccessNames {
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
