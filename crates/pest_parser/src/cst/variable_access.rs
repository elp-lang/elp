use crate::cst::ident::CSTIdent;
use crate::parser::Rule;
use pest::Span;
use pest_ast::FromPest;

// Not a fan of having anonymous structs for these rules to fit into the enum
// but it is what it is.
#[derive(Debug, FromPest, PartialEq, Eq, Clone)]
#[pest_ast(rule(Rule::POINTER))]
pub struct CSTPointer<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
}

#[derive(Debug, FromPest, PartialEq, Eq, Clone)]
#[pest_ast(rule(Rule::REFERENCE))]
pub struct CSTReference<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
}

#[derive(Debug, FromPest, PartialEq, Eq, Clone)]
#[pest_ast(rule(Rule::pointer_semantics))]
pub enum CSTPointerSemantics<'a> {
    Pointer(CSTPointer<'a>),
    Reference(CSTReference<'a>),
}

#[derive(Debug, FromPest, PartialEq, Eq, Clone)]
#[pest_ast(rule(Rule::variable_access))]
pub struct CSTVariableAccess<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
    pub pointer_semantics: Vec<CSTPointerSemantics<'a>>,
    pub names: CSTVariableAccessNames<'a>,
}

#[derive(Debug, FromPest, PartialEq, Eq, Clone)]
#[pest_ast(rule(Rule::variable_access_names))]
pub struct CSTVariableAccessNames<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
    pub names: Vec<CSTIdent<'a>>,
}

#[derive(Debug, FromPest, PartialEq, Eq, Clone)]
#[pest_ast(rule(Rule::contextual_variable_access))]
pub struct CSTContextualVariableAccess<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
    pub name: CSTIdent<'a>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cst::variable_access::CSTVariableAccess;
    use crate::parser::ElpParser;
    use from_pest::FromPest;
    use pest::Parser;

    use pretty_assertions::assert_eq;

    #[test]
    fn test_pointer_semantics() {
        let ref_expression_str = "&";
        let mut ref_pairs = ElpParser::parse(Rule::pointer_semantics, ref_expression_str).unwrap();
        let ref_ast = CSTPointerSemantics::from_pest(&mut ref_pairs).unwrap();

        assert_eq!(
            ref_ast,
            CSTPointerSemantics::Reference(CSTReference {
                span: pest::Span::new(ref_expression_str, 0, 1).unwrap(),
            })
        );

        let ptr_expression_str = "*";
        let mut ptr_pairs = ElpParser::parse(Rule::pointer_semantics, ptr_expression_str).unwrap();
        let ptr_ast = CSTPointerSemantics::from_pest(&mut ptr_pairs).unwrap();

        assert_eq!(
            ptr_ast,
            CSTPointerSemantics::Pointer(CSTPointer {
                span: pest::Span::new(ptr_expression_str, 0, 1).unwrap(),
            })
        );
    }

    #[test]
    fn variable_access() {
        let expression_str = "hello.world.my.name.is.dave";
        let mut pairs = ElpParser::parse(Rule::variable_access, expression_str).unwrap();
        let ast = CSTVariableAccess::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTVariableAccess {
                span: pest::Span::new(expression_str, 0, 27).unwrap(),
                pointer_semantics: vec![],
                names: CSTVariableAccessNames {
                    span: pest::Span::new(expression_str, 0, 27).unwrap(),
                    names: vec![
                        CSTIdent {
                            span: pest::Span::new(expression_str, 0, 5).unwrap(),
                            value: "hello".into()
                        },
                        CSTIdent {
                            span: pest::Span::new(expression_str, 6, 11).unwrap(),
                            value: "world".into()
                        },
                        CSTIdent {
                            span: pest::Span::new(expression_str, 12, 14).unwrap(),
                            value: "my".into()
                        },
                        CSTIdent {
                            span: pest::Span::new(expression_str, 15, 19).unwrap(),
                            value: "name".into()
                        },
                        CSTIdent {
                            span: pest::Span::new(expression_str, 20, 22).unwrap(),
                            value: "is".into()
                        },
                        CSTIdent {
                            span: pest::Span::new(expression_str, 23, 27).unwrap(),
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
        let reference_ast = CSTVariableAccess::from_pest(&mut pairs_reference).unwrap();

        assert_eq!(
            reference_ast,
            CSTVariableAccess {
                span: pest::Span::new(expression_str_reference, 0, 28).unwrap(),
                pointer_semantics: vec![CSTPointerSemantics::Reference(CSTReference {
                    span: pest::Span::new(expression_str_reference, 0, 1).unwrap(),
                })],
                names: CSTVariableAccessNames {
                    span: pest::Span::new(expression_str_reference, 1, 28).unwrap(),
                    names: vec![
                        CSTIdent {
                            span: pest::Span::new(expression_str_reference, 1, 6).unwrap(),
                            value: "hello".into()
                        },
                        CSTIdent {
                            span: pest::Span::new(expression_str_reference, 7, 12).unwrap(),
                            value: "world".into()
                        },
                        CSTIdent {
                            span: pest::Span::new(expression_str_reference, 13, 15).unwrap(),
                            value: "my".into()
                        },
                        CSTIdent {
                            span: pest::Span::new(expression_str_reference, 16, 20).unwrap(),
                            value: "name".into()
                        },
                        CSTIdent {
                            span: pest::Span::new(expression_str_reference, 21, 23).unwrap(),
                            value: "is".into()
                        },
                        CSTIdent {
                            span: pest::Span::new(expression_str_reference, 24, 28).unwrap(),
                            value: "dave".into()
                        },
                    ],
                }
            }
        );

        let expression_str_pointer = "*hello.world.my.name.is.dave";
        let mut pairs_pointer =
            ElpParser::parse(Rule::variable_access, expression_str_pointer).unwrap();
        let pointer_ast = CSTVariableAccess::from_pest(&mut pairs_pointer).unwrap();

        assert_eq!(
            pointer_ast,
            CSTVariableAccess {
                span: pest::Span::new(expression_str_pointer, 0, 28).unwrap(),
                pointer_semantics: vec![CSTPointerSemantics::Pointer(CSTPointer {
                    span: pest::Span::new(expression_str_pointer, 0, 1).unwrap()
                })],
                names: CSTVariableAccessNames {
                    span: pest::Span::new(expression_str_pointer, 1, 28).unwrap(),
                    names: vec![
                        CSTIdent {
                            span: pest::Span::new(expression_str_pointer, 1, 6).unwrap(),
                            value: "hello".into()
                        },
                        CSTIdent {
                            span: pest::Span::new(expression_str_pointer, 7, 12).unwrap(),
                            value: "world".into()
                        },
                        CSTIdent {
                            span: pest::Span::new(expression_str_pointer, 13, 15).unwrap(),
                            value: "my".into()
                        },
                        CSTIdent {
                            span: pest::Span::new(expression_str_pointer, 16, 20).unwrap(),
                            value: "name".into()
                        },
                        CSTIdent {
                            span: pest::Span::new(expression_str_pointer, 21, 23).unwrap(),
                            value: "is".into()
                        },
                        CSTIdent {
                            span: pest::Span::new(expression_str_pointer, 24, 28).unwrap(),
                            value: "dave".into()
                        },
                    ],
                }
            }
        )
    }

    #[test]
    fn contextual_variable_access() {
        let expression_str_pointer = ".CONTEXTUAL";
        let mut pairs_pointer =
            ElpParser::parse(Rule::contextual_variable_access, expression_str_pointer).unwrap();
        let pointer_ast = CSTContextualVariableAccess::from_pest(&mut pairs_pointer).unwrap();

        assert_eq!(
            pointer_ast,
            CSTContextualVariableAccess {
                span: pest::Span::new(expression_str_pointer, 0, 11).unwrap(),
                name: CSTIdent {
                    span: pest::Span::new(expression_str_pointer, 1, 11).unwrap(),
                    value: "CONTEXTUAL".into()
                },
            }
        )
    }
}
