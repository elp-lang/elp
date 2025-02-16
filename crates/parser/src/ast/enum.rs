use pest::Span;

use crate::cst::r#enum::{CSTEnum, CSTEnumMember};

use super::{elp_type::ASTElpType, traits::FromCST};

#[derive(Debug, PartialEq, Clone)]
pub struct ASTEnum<'a> {
    pub span: &'a Span<'a>,
    pub name: String,
    pub members: Vec<ASTEnumMember<'a>>,
    pub implements: Vec<ASTElpType<'a>>,
}

impl<'a> FromCST<'a, CSTEnum<'a>> for ASTEnum<'a> {
    fn from_cst(cst: &'a CSTEnum) -> Self {
        Self {
            span: &cst.span,
            name: cst.name.value.clone(),
            members: cst.members.iter().map(ASTEnumMember::from_cst).collect(),
            implements: match &cst.implements {
                Some(implements) => implements.types.iter().map(ASTElpType::from_cst).collect(),
                None => vec![],
            },
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ASTEnumMember<'a> {
    pub span: &'a Span<'a>,
    pub name: String,
    pub parameters: Vec<ASTElpType<'a>>,
}

impl<'a> FromCST<'a, CSTEnumMember<'a>> for ASTEnumMember<'a> {
    fn from_cst(cst: &'a CSTEnumMember) -> Self {
        Self {
            span: &cst.span,
            name: cst.name.value.clone(),
            parameters: cst.params.iter().map(ASTElpType::from_cst).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::cst::ident::CSTIdent;

    use super::*;

    #[test]
    fn basic_enum() {
        let expression_str = "enum MyEnum { .MEMBER }";
        let cst_enum = CSTEnum {
            span: pest::Span::new(expression_str, 0, 23).unwrap(),
            name: CSTIdent {
                span: pest::Span::new(expression_str, 5, 11).unwrap(),
                value: "MyEnum".into(),
            },
            implements: None,
            members: vec![CSTEnumMember {
                span: pest::Span::new(expression_str, 14, 22).unwrap(),
                name: CSTIdent {
                    span: pest::Span::new(expression_str, 15, 21).unwrap(),
                    value: "MEMBER".into(),
                },
                params: vec![],
            }],
        };

        let ast_enum = ASTEnum::from_cst(&cst_enum);

        assert_eq!(
            ast_enum,
            ASTEnum {
                span: &pest::Span::new(expression_str, 0, 23).unwrap(),
                name: "MyEnum".to_string(),
                members: vec![ASTEnumMember {
                    span: &pest::Span::new(expression_str, 14, 22).unwrap(),
                    name: "MEMBER".to_string(),
                    parameters: vec![]
                }],
                implements: vec![],
            }
        )
    }
}
