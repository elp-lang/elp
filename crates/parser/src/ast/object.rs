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
            elp_type::{
                CSTElpType, CSTElpTypeGeneric, CSTElpTypeGenericParam, CSTElpTypeParameter,
                CSTElpTypeValue,
            },
            expression::CSTExpression,
            ident::CSTIdent,
            number_value::CSTNumber,
            object::CSTObjectMemberDefaultValue,
            string::CSTString,
            CSTPrivateVisibility, CSTPublicVisibility,
        },
    };

    use super::*;

    use pretty_assertions::assert_eq;

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

    #[test]
    fn ast_complex_object() {
        let expression_str = "object Test implements Into<JSON> {
            public     .name String `json:\"name\"`,
            private    .age Int     `json:\"age\"`,
            .friends   Vec<Friend>  `json:\"friends\"`,
            .studentId = 123        `json:\"studentId\"`
    }";
        let cst = CSTObject {
            span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
            name: CSTIdent {
                span: pest::Span::new(expression_str, 7, 11).unwrap(),
                value: "Test".into(),
            },
            implements: Some(CSTObjectImplements {
                span: pest::Span::new(expression_str, 12, 34).unwrap(),
                types: vec![CSTElpType {
                    span: pest::Span::new(expression_str, 23, 33).unwrap(),
                    mutability: None,
                    pointer_semantics: None,
                    value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                        span: pest::Span::new(expression_str, 23, 33).unwrap(),
                        name: CSTIdent {
                            span: pest::Span::new(expression_str, 23, 27).unwrap(),
                            value: "Into".into(),
                        },
                        generics: Some(CSTElpTypeGeneric {
                            span: pest::Span::new(expression_str, 27, 33).unwrap(),
                            params: vec![CSTElpTypeGenericParam {
                                span: pest::Span::new(expression_str, 28, 32).unwrap(),
                                elp_type: CSTElpType {
                                    span: pest::Span::new(expression_str, 28, 32).unwrap(),
                                    mutability: None,
                                    pointer_semantics: None,
                                    value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                        span: pest::Span::new(expression_str, 28, 32).unwrap(),
                                        name: CSTIdent {
                                            span: pest::Span::new(expression_str, 28, 32).unwrap(),
                                            value: "JSON".into(),
                                        },
                                        generics: None,
                                    }),
                                },
                                type_constraints: None,
                            }],
                        }),
                    }),
                }],
            }),
            members: vec![
                CSTObjectMember {
                    span: pest::Span::new(expression_str, 48, 85).unwrap(),
                    visibility: Some(CSTVisibilitySelector::Public(CSTPublicVisibility {
                        span: pest::Span::new(expression_str, 48, 54).unwrap(),
                    })),
                    name: CSTIdent {
                        span: pest::Span::new(expression_str, 60, 64).unwrap(),
                        value: "name".into(),
                    },
                    type_annotation: Some(CSTElpType {
                        span: pest::Span::new(expression_str, 65, 72).unwrap(),
                        mutability: None,
                        pointer_semantics: None,
                        value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                            span: pest::Span::new(expression_str, 65, 72).unwrap(),
                            name: CSTIdent {
                                span: pest::Span::new(expression_str, 65, 71).unwrap(),
                                value: "String".into(),
                            },
                            generics: None,
                        }),
                    }),
                    default_value: None,
                    tags: vec![CSTObjectMemberTags {
                        span: pest::Span::new(expression_str, 72, 85).unwrap(),
                        name: CSTIdent {
                            span: pest::Span::new(expression_str, 73, 77).unwrap(),
                            value: "json".into(),
                        },
                        contents: CSTString {
                            span: pest::Span::new(expression_str, 78, 84).unwrap(),
                            value: "name".into(),
                        },
                    }],
                },
                CSTObjectMember {
                    span: pest::Span::new(expression_str, 99, 135).unwrap(),
                    visibility: Some(CSTVisibilitySelector::Private(CSTPrivateVisibility {
                        span: pest::Span::new(expression_str, 99, 106).unwrap(),
                    })),
                    name: CSTIdent {
                        span: pest::Span::new(expression_str, 111, 114).unwrap(),
                        value: "age".into(),
                    },
                    type_annotation: Some(CSTElpType {
                        span: pest::Span::new(expression_str, 115, 123).unwrap(),
                        mutability: None,
                        pointer_semantics: None,
                        value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                            span: pest::Span::new(expression_str, 115, 123).unwrap(),
                            name: CSTIdent {
                                span: pest::Span::new(expression_str, 115, 118).unwrap(),
                                value: "Int".into(),
                            },
                            generics: None,
                        }),
                    }),
                    default_value: None,
                    tags: vec![CSTObjectMemberTags {
                        span: pest::Span::new(expression_str, 123, 135).unwrap(),
                        name: CSTIdent {
                            span: pest::Span::new(expression_str, 124, 128).unwrap(),
                            value: "json".into(),
                        },
                        contents: CSTString {
                            span: pest::Span::new(expression_str, 129, 134).unwrap(),
                            value: "age".into(),
                        },
                    }],
                },
                CSTObjectMember {
                    span: pest::Span::new(expression_str, 149, 189).unwrap(),
                    visibility: None,
                    name: CSTIdent {
                        span: pest::Span::new(expression_str, 150, 157).unwrap(),
                        value: "friends".into(),
                    },
                    type_annotation: Some(CSTElpType {
                        span: pest::Span::new(expression_str, 160, 171).unwrap(),
                        mutability: None,
                        pointer_semantics: None,
                        value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                            span: pest::Span::new(expression_str, 160, 171).unwrap(),
                            name: CSTIdent {
                                span: pest::Span::new(expression_str, 160, 163).unwrap(),
                                value: "Vec".into(),
                            },
                            generics: Some(CSTElpTypeGeneric {
                                span: pest::Span::new(expression_str, 163, 171).unwrap(),
                                params: vec![CSTElpTypeGenericParam {
                                    span: pest::Span::new(expression_str, 164, 170).unwrap(),
                                    elp_type: CSTElpType {
                                        span: pest::Span::new(expression_str, 164, 170).unwrap(),
                                        mutability: None,
                                        pointer_semantics: None,
                                        value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                            span: pest::Span::new(expression_str, 164, 170)
                                                .unwrap(),
                                            name: CSTIdent {
                                                span: pest::Span::new(expression_str, 164, 170)
                                                    .unwrap(),
                                                value: "Friend".into(),
                                            },
                                            generics: None,
                                        }),
                                    },
                                    type_constraints: None,
                                }],
                            }),
                        }),
                    }),
                    default_value: None,
                    tags: vec![CSTObjectMemberTags {
                        span: pest::Span::new(expression_str, 173, 189).unwrap(),
                        name: CSTIdent {
                            span: pest::Span::new(expression_str, 174, 178).unwrap(),
                            value: "json".into(),
                        },
                        contents: CSTString {
                            span: pest::Span::new(expression_str, 179, 188).unwrap(),
                            value: "friends".into(),
                        },
                    }],
                },
                CSTObjectMember {
                    span: pest::Span::new(expression_str, 203, 245).unwrap(),
                    visibility: None,
                    name: CSTIdent {
                        span: pest::Span::new(expression_str, 204, 213).unwrap(),
                        value: "studentId".into(),
                    },
                    type_annotation: None,
                    default_value: Some(CSTObjectMemberDefaultValue {
                        span: pest::Span::new(expression_str, 214, 219).unwrap(),
                        value: CSTExpression::Number(Box::new(CSTNumber {
                            span: pest::Span::new(expression_str, 216, 219).unwrap(),
                            value: "123".into(),
                        })),
                    }),
                    tags: vec![CSTObjectMemberTags {
                        span: pest::Span::new(expression_str, 227, 245).unwrap(),
                        name: CSTIdent {
                            span: pest::Span::new(expression_str, 228, 232).unwrap(),
                            value: "json".into(),
                        },
                        contents: CSTString {
                            span: pest::Span::new(expression_str, 233, 244).unwrap(),
                            value: "studentId".into(),
                        },
                    }],
                },
            ],
        };

        let ast = ASTObject::from_cst(&cst);

        assert_eq!(
            ast,
            ASTObject {
                span: &pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                name: "Test".into(),
                implements: vec![ASTObjectImplements {
                    span: &pest::Span::new(expression_str, 23, 30).unwrap(),
                    types: vec![ASTElpType {
                        span: &pest::Span::new(expression_str, 33, 41).unwrap(),
                        mutability: ASTMutability::Immutable,
                        pointer_semantics: None,
                        name: "JSON".into(),
                        type_constraints: vec![],
                        generic_parameters: vec![ASTElpType {
                            span: &pest::Span::new(expression_str, 42, 48).unwrap(),
                            name: "JSON".into(),
                            pointer_semantics: None,
                            mutability: ASTMutability::Immutable,
                            generic_parameters: vec![],
                            type_constraints: vec![],
                        }],
                    }],
                }],
                members: vec![
                    ASTObjectMember {
                        span: &pest::Span::new(expression_str, 23, 65).unwrap(),
                        visibility: Some(ASTVisibility::Public),
                        name: "name".into(),
                        type_annotation: Some(ASTElpType {
                            span: &pest::Span::new(expression_str, 33, 41).unwrap(),
                            mutability: ASTMutability::Immutable,
                            pointer_semantics: None,
                            name: "String".into(),
                            type_constraints: vec![],
                            generic_parameters: vec![],
                        }),
                        default_value: None,
                        tags: vec![ASTObjectMemberTags {
                            span: &pest::Span::new(expression_str, 42, 65).unwrap(),
                            name: "json".into(),
                            value: "name".into(),
                        }],
                    },
                    ASTObjectMember {
                        span: &pest::Span::new(expression_str, 42, 65).unwrap(),
                        name: "age".into(),
                        visibility: Some(ASTVisibility::Private),
                        type_annotation: Some(ASTElpType {
                            span: &pest::Span::new(expression_str, 53, 61).unwrap(),
                            mutability: ASTMutability::Immutable,
                            pointer_semantics: None,
                            name: "Int".into(),
                            type_constraints: vec![],
                            generic_parameters: vec![],
                        }),
                        default_value: None,
                        tags: vec![ASTObjectMemberTags {
                            span: &pest::Span::new(expression_str, 62, 65).unwrap(),
                            name: "json".into(),
                            value: "age".into(),
                        }]
                    },
                    ASTObjectMember {
                        span: &pest::Span::new(expression_str, 149, 189).unwrap(),
                        visibility: Some(ASTVisibility::Private),
                        name: "friends".into(),
                        type_annotation: Some(ASTElpType {
                            span: &pest::Span::new(expression_str, 160, 171).unwrap(),
                            mutability: ASTMutability::Immutable,
                            pointer_semantics: None,
                            name: "Vec".into(),
                            type_constraints: vec![],
                            generic_parameters: vec![ASTElpType {
                                span: &pest::Span::new(expression_str, 164, 170).unwrap(),
                                name: "Friend".into(),
                                pointer_semantics: None,
                                mutability: ASTMutability::Immutable,
                                generic_parameters: vec![],
                                type_constraints: vec![],
                            }],
                        }),
                        default_value: None,
                        tags: vec![ASTObjectMemberTags {
                            span: &pest::Span::new(expression_str, 173, 189).unwrap(),
                            name: "json".into(),
                            value: "friends".into(),
                        }]
                    },
                    ASTObjectMember {
                        span: &pest::Span::new(expression_str, 203, 245).unwrap(),
                        visibility: None,
                        name: "studentId".into(),
                        type_annotation: None,
                        default_value: None,
                        tags: vec![ASTObjectMemberTags {
                            span: &pest::Span::new(expression_str, 227, 245).unwrap(),
                            name: "json".into(),
                            value: "studentId".into(),
                        }]
                    }
                ],
            }
        )
    }
}
