use pest::Span;
use pest_ast::FromPest;

use crate::parser::Rule;

use super::{elp_type::CSTElpType, ident::CSTIdent, object::CSTObjectImplements, span_into_string};

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::enum_member))]
pub struct CSTEnumMember<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
    #[pest_ast(inner(with(span_into_string)))]
    pub name: String,
    pub params: Vec<CSTElpType<'a>>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::r#enum))]
pub struct CSTEnum<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
    pub name: CSTIdent<'a>,
    pub implements: Option<CSTObjectImplements<'a>>,
    pub members: Vec<CSTEnumMember<'a>>,
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
                span: pest::Span::new(expression_str, 0, 7).unwrap(),
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
                span: pest::Span::new(expression_str, 0, 10).unwrap(),
                name: "Member".into(),
                params: vec![CSTElpType {
                    span: pest::Span::new(expression_str, 8, 9).unwrap(),
                    mutability: None,
                    pointer_semantics: None,
                    value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                        span: pest::Span::new(expression_str, 8, 9).unwrap(),
                        name: CSTIdent {
                            span: pest::Span::new(expression_str, 8, 9).unwrap(),
                            value: "T".into()
                        },
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
                span: pest::Span::new(expression_str, 0, 23).unwrap(),
                name: CSTIdent {
                    span: pest::Span::new(expression_str, 5, 11).unwrap(),
                    value: "MyEnum".into()
                },
                implements: None,
                members: vec![CSTEnumMember {
                    span: pest::Span::new(expression_str, 14, 22).unwrap(),
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
                span: pest::Span::new(expression_str, 0, 46).unwrap(),
                name: CSTIdent {
                    span: pest::Span::new(expression_str, 5, 11).unwrap(),
                    value: "MyEnum".into()
                },
                implements: Some(CSTObjectImplements {
                    span: pest::Span::new(expression_str, 12, 35).unwrap(),
                    types: vec![CSTElpType {
                        span: pest::Span::new(expression_str, 23, 35).unwrap(),
                        mutability: None,
                        pointer_semantics: None,
                        value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                            span: pest::Span::new(expression_str, 23, 35).unwrap(),
                            name: CSTIdent {
                                span: pest::Span::new(expression_str, 23, 34).unwrap(),
                                value: "MyInterface".into()
                            },
                            generics: vec![]
                        })
                    }]
                }),
                members: vec![CSTEnumMember {
                    span: pest::Span::new(expression_str, 37, 45).unwrap(),
                    name: "MEMBER".into(),
                    params: vec![]
                }]
            }
        )
    }
}
