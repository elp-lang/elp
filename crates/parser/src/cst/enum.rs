use pest_ast::FromPest;

use crate::parser::Rule;

use super::{elp_type::CSTElpType, ident::CSTIdent, object::CSTObjectImplements, span_into_string};

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::enum_member))]
pub struct CSTEnumMember {
    #[pest_ast(inner(with(span_into_string)))]
    pub name: String,
    pub params: Vec<CSTElpType>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::r#enum))]
pub struct CSTEnum {
    pub name: CSTIdent,
    pub implements: Option<CSTObjectImplements>,
    pub members: Vec<CSTEnumMember>,
}

#[cfg(test)]
mod tests {
    use crate::{
        cst::{
            elp_type::{CSTElpTypeParameter, CSTElpTypeValue},
            ident::CSTIdent,
        },
        parser::ElpParser,
    };

    use super::*;
    use from_pest::FromPest;
    use pest::Parser;
    use pretty_assertions::assert_eq;

    #[test]
    fn enum_member() {
        let expression_str = ".MEMBER";
        let mut pairs = ElpParser::parse(Rule::enum_member, expression_str).unwrap();
        let ast = CSTEnumMember::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTEnumMember {
                name: "MEMBER".into(),
                params: vec![]
            }
        )
    }

    #[test]
    fn enum_member_with_params() {
        let expression_str = ".Member(T)";
        let mut pairs = ElpParser::parse(Rule::enum_member, expression_str).unwrap();
        let ast = CSTEnumMember::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTEnumMember {
                name: "Member".into(),
                params: vec![CSTElpType {
                    mutability: None,
                    pointer_semantics: None,
                    value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                        name: CSTIdent { value: "T".into() },
                        generics: vec![]
                    })
                }]
            }
        )
    }

    #[test]
    fn enum_def() {
        let expression_str = "enum MyEnum { .MEMBER }";
        let mut pairs = ElpParser::parse(Rule::r#enum, expression_str).unwrap();
        let ast = CSTEnum::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTEnum {
                name: CSTIdent {
                    value: "MyEnum".into()
                },
                implements: None,
                members: vec![CSTEnumMember {
                    name: "MEMBER".into(),
                    params: vec![]
                }]
            }
        )
    }

    #[test]
    fn enum_implements() {
        let expression_str = "enum MyEnum implements MyInterface { .MEMBER }";
        let mut pairs = ElpParser::parse(Rule::r#enum, expression_str).unwrap();
        let ast = CSTEnum::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTEnum {
                name: CSTIdent {
                    value: "MyEnum".into()
                },
                implements: Some(CSTObjectImplements {
                    types: vec![CSTElpType {
                        mutability: None,
                        pointer_semantics: None,
                        value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                            name: CSTIdent {
                                value: "MyInterface".into()
                            },
                            generics: vec![]
                        })
                    }]
                }),
                members: vec![CSTEnumMember {
                    name: "MEMBER".into(),
                    params: vec![]
                }]
            }
        )
    }
}
