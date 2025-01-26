use crate::cst::ident::CSTIdent;
use crate::parser::Rule;
use pest_ast::FromPest;

// Not a fan of having anonymous structs for these rules to fit into the enum
// but it is what it is.
#[derive(Debug, FromPest, PartialEq, Eq, Clone)]
#[pest_ast(rule(Rule::POINTER))]
pub struct CSTPointer;

#[derive(Debug, FromPest, PartialEq, Eq, Clone)]
#[pest_ast(rule(Rule::REFERENCE))]
pub struct CSTReference;

#[derive(Debug, FromPest, PartialEq, Eq, Clone)]
#[pest_ast(rule(Rule::pointer_semantics))]
pub enum CSTPointerSemantics {
    Pointer(CSTPointer),
    Reference(CSTReference),
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::variable_access))]
pub struct CSTVariableAccess {
    pub pointer_semantics: Vec<CSTPointerSemantics>,
    pub names: CSTVariableAccessNames,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::variable_access_names))]
pub struct CSTVariableAccessNames {
    pub names: Vec<CSTIdent>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::contextual_variable_access))]
pub struct CSTContextualVariableAccess {
    pub name: CSTIdent,
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

        assert_eq!(ref_ast, CSTPointerSemantics::Reference(CSTReference {}));

        let ptr_expression_str = "*";
        let mut ptr_pairs = ElpParser::parse(Rule::pointer_semantics, ptr_expression_str).unwrap();
        let ptr_ast = CSTPointerSemantics::from_pest(&mut ptr_pairs).unwrap();

        assert_eq!(ptr_ast, CSTPointerSemantics::Pointer(CSTPointer {}));
    }

    #[test]
    fn variable_access() {
        let expression_str = "hello.world.my.name.is.dave";
        let mut pairs = ElpParser::parse(Rule::variable_access, expression_str).unwrap();
        let ast = CSTVariableAccess::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTVariableAccess {
                pointer_semantics: vec![],
                names: CSTVariableAccessNames {
                    names: vec![
                        CSTIdent {
                            value: "hello".into()
                        },
                        CSTIdent {
                            value: "world".into()
                        },
                        CSTIdent { value: "my".into() },
                        CSTIdent {
                            value: "name".into()
                        },
                        CSTIdent { value: "is".into() },
                        CSTIdent {
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
                pointer_semantics: vec![CSTPointerSemantics::Reference(CSTReference {})],
                names: CSTVariableAccessNames {
                    names: vec![
                        CSTIdent {
                            value: "hello".into()
                        },
                        CSTIdent {
                            value: "world".into()
                        },
                        CSTIdent { value: "my".into() },
                        CSTIdent {
                            value: "name".into()
                        },
                        CSTIdent { value: "is".into() },
                        CSTIdent {
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
                pointer_semantics: vec![CSTPointerSemantics::Pointer(CSTPointer {})],
                names: CSTVariableAccessNames {
                    names: vec![
                        CSTIdent {
                            value: "hello".into()
                        },
                        CSTIdent {
                            value: "world".into()
                        },
                        CSTIdent { value: "my".into() },
                        CSTIdent {
                            value: "name".into()
                        },
                        CSTIdent { value: "is".into() },
                        CSTIdent {
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
                name: CSTIdent {
                    value: "CONTEXTUAL".into()
                },
            }
        )
    }
}
