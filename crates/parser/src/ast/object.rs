use pest::Span;

use crate::cst::{
    object::{CSTObject, CSTObjectImplements, CSTObjectMember, CSTObjectMemberTags},
    CSTVisibilitySelector,
};

use super::{elp_type::ASTElpType, expression::ASTExpression, traits::FromCST};

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum ASTVisibility {
    Public,
    Private,
}

impl<'a> FromCST<'a, CSTVisibilitySelector<'a>> for ASTVisibility {
    fn from_cst(cst: &'a CSTVisibilitySelector) -> Self {
        match cst {
            CSTVisibilitySelector::Public(_) => ASTVisibility::Public,
            CSTVisibilitySelector::Private(_) => ASTVisibility::Private,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ASTObject<'a> {
    pub span: &'a Span<'a>,
    pub name: String,
    pub members: Vec<ASTObjectMember<'a>>,
    pub implements: Vec<ASTObjectImplements<'a>>,
}

impl<'a> FromCST<'a, CSTObject<'a>> for ASTObject<'a> {
    fn from_cst(cst: &'a CSTObject) -> Self {
        ASTObject {
            span: &cst.span,
            name: cst.name.value.clone(),
            members: cst.members.iter().map(ASTObjectMember::from_cst).collect(),
            implements: cst
                .implements
                .iter()
                .map(ASTObjectImplements::from_cst)
                .collect(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ASTObjectImplements<'a> {
    pub span: &'a Span<'a>,
    pub types: Vec<ASTElpType<'a>>,
}

impl<'a> FromCST<'a, CSTObjectImplements<'a>> for ASTObjectImplements<'a> {
    fn from_cst(cst: &'a CSTObjectImplements) -> Self {
        ASTObjectImplements {
            span: &cst.span,
            types: cst.types.iter().map(ASTElpType::from_cst).collect(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ASTObjectMember<'a> {
    pub span: &'a Span<'a>,
    pub name: String,
    pub type_annotation: Option<ASTElpType<'a>>,
    pub default_value: Option<ASTExpression<'a>>,
    pub visibility: Option<ASTVisibility>,
    pub tags: Vec<ASTObjectMemberTags<'a>>,
}

impl<'a> FromCST<'a, CSTObjectMember<'a>> for ASTObjectMember<'a> {
    fn from_cst(cst: &'a CSTObjectMember) -> Self {
        ASTObjectMember {
            span: &cst.span,
            name: cst.name.value.clone(),
            visibility: cst.visibility.as_ref().map(ASTVisibility::from_cst),
            type_annotation: cst.type_annotation.as_ref().map(ASTElpType::from_cst),
            default_value: cst
                .default_value
                .as_ref()
                .map(|default_value| ASTExpression::from_cst(&default_value.value)),
            tags: cst.tags.iter().map(ASTObjectMemberTags::from_cst).collect(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ASTObjectMemberTags<'a> {
    pub span: &'a Span<'a>,
    pub name: String,
    pub value: String,
}

impl<'a> FromCST<'a, CSTObjectMemberTags<'a>> for ASTObjectMemberTags<'a> {
    fn from_cst(cst: &'a CSTObjectMemberTags) -> Self {
        ASTObjectMemberTags {
            span: &cst.span,
            name: cst.name.value.clone(),
            value: cst.contents.value.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::elp_type::ASTMutability,
        cst::{
            elp_type::{CSTElpType, CSTElpTypeParameter, CSTElpTypeValue},
            ident::CSTIdent,
        },
    };

    use super::*;

    #[test]
    fn object_from_cst() {
        let expression_str = "object Test {.name String}";
        let cst = CSTObject {
            span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
            name: CSTIdent {
                span: pest::Span::new(expression_str, 7, 11).unwrap(),
                value: "Test".into(),
            },
            implements: None,
            members: vec![CSTObjectMember {
                span: pest::Span::new(expression_str, 13, 25).unwrap(),
                visibility: None,
                name: CSTIdent {
                    span: pest::Span::new(expression_str, 14, 18).unwrap(),
                    value: "name".into(),
                },
                type_annotation: Some(CSTElpType {
                    span: pest::Span::new(expression_str, 19, 25).unwrap(),
                    mutability: None,
                    pointer_semantics: None,
                    value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                        span: pest::Span::new(expression_str, 19, 25).unwrap(),
                        name: CSTIdent {
                            span: pest::Span::new(expression_str, 19, 25).unwrap(),
                            value: "String".into(),
                        },
                        generics: None,
                    }),
                }),
                default_value: None,
                tags: vec![],
            }],
        };

        let ast = ASTObject::from_cst(&cst);

        assert_eq!(
            ast,
            ASTObject {
                span: &cst.span,
                name: "Test".into(),
                members: vec![ASTObjectMember {
                    span: &cst.members[0].span,
                    visibility: None,
                    name: "name".into(),
                    type_annotation: Some(ASTElpType {
                        span: &cst.members[0].type_annotation.as_ref().unwrap().span,
                        mutability: ASTMutability::Immutable,
                        pointer_semantics: None,
                        name: "String".into(),
                        type_constraints: vec![],
                        generic_parameters: vec![]
                    }),
                    default_value: None,
                    tags: vec![],
                }],
                implements: vec![],
            }
        )
    }
}
