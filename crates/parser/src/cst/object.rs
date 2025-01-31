use super::{
    elp_type::CSTElpType, expression::CSTExpression, ident::CSTIdent, string::CSTString,
    VisibilitySelector,
};
use crate::parser::Rule;
use pest::Span;
use pest_ast::FromPest;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::object_implements))]
pub struct CSTObjectImplements<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
    pub types: Vec<CSTElpType<'a>>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::object_key_default_value))]
pub struct CSTObjectMemberDefaultValue<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
    pub value: CSTExpression<'a>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::object_key_tags))]
pub struct CSTObjectMemberTags<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
    pub name: CSTIdent<'a>,
    pub contents: CSTString<'a>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::object_member))]
pub struct CSTObjectMember<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
    pub visibility: Option<VisibilitySelector<'a>>,
    pub name: CSTIdent<'a>,
    pub type_annotation: Option<CSTElpType<'a>>,
    pub default_value: Option<CSTObjectMemberDefaultValue<'a>>,
    pub tags: Vec<CSTObjectMemberTags<'a>>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::object_def))]
pub struct CSTObject<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
    pub name: CSTIdent<'a>,
    pub implements: Option<CSTObjectImplements<'a>>,
    pub members: Vec<CSTObjectMember<'a>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cst::{
            elp_type::{
                CSTElpTypeGeneric, CSTElpTypeGenericParam, CSTElpTypeParameter, CSTElpTypeValue,
            },
            number_value::CSTNumber,
            PrivateVisibility, PublicVisibility,
        },
        parser::ElpParser,
    };
    use from_pest::FromPest;
    use pest::Parser;
    use pretty_assertions::assert_eq;

    #[test]
    fn object_implements() {
        let expression_str_basic = "implements String";
        let mut pairs_basic =
            ElpParser::parse(Rule::object_implements, expression_str_basic).unwrap();
        let ast_basic = CSTObjectImplements::from_pest(&mut pairs_basic).unwrap();

        assert_eq!(
            ast_basic,
            CSTObjectImplements {
                span: pest::Span::new(expression_str_basic, 0, 17).unwrap(),
                types: vec![CSTElpType {
                    span: pest::Span::new(expression_str_basic, 11, 17).unwrap(),
                    mutability: None,
                    pointer_semantics: None,
                    value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                        span: pest::Span::new(expression_str_basic, 11, 17).unwrap(),
                        name: CSTIdent {
                            span: pest::Span::new(expression_str_basic, 11, 17).unwrap(),
                            value: "String".into()
                        },
                        generics: vec![],
                    })
                }]
            }
        );

        let expression_str_multiple = "implements String, Number, Into<JSON>";
        let mut pairs_multiple =
            ElpParser::parse(Rule::object_implements, expression_str_multiple).unwrap();
        let ast_multiple = CSTObjectImplements::from_pest(&mut pairs_multiple).unwrap();

        assert_eq!(
            ast_multiple,
            CSTObjectImplements {
                span: pest::Span::new(expression_str_multiple, 0, 37).unwrap(),
                types: vec![
                    CSTElpType {
                        span: pest::Span::new(expression_str_multiple, 11, 17).unwrap(),
                        mutability: None,
                        pointer_semantics: None,
                        value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                            span: pest::Span::new(expression_str_multiple, 11, 17).unwrap(),
                            name: CSTIdent {
                                span: pest::Span::new(expression_str_multiple, 11, 17).unwrap(),
                                value: "String".into()
                            },
                            generics: vec![],
                        })
                    },
                    CSTElpType {
                        span: pest::Span::new(expression_str_multiple, 19, 25).unwrap(),
                        mutability: None,
                        pointer_semantics: None,
                        value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                            span: pest::Span::new(expression_str_multiple, 19, 25).unwrap(),
                            name: CSTIdent {
                                span: pest::Span::new(expression_str_multiple, 19, 25).unwrap(),
                                value: "Number".into()
                            },
                            generics: vec![],
                        })
                    },
                    CSTElpType {
                        span: pest::Span::new(expression_str_multiple, 27, 37).unwrap(),
                        mutability: None,
                        pointer_semantics: None,
                        value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                            span: pest::Span::new(expression_str_multiple, 27, 37).unwrap(),
                            name: CSTIdent {
                                span: pest::Span::new(expression_str_multiple, 27, 31).unwrap(),
                                value: "Into".into()
                            },
                            generics: vec![CSTElpTypeGeneric {
                                span: pest::Span::new(expression_str_multiple, 31, 37).unwrap(),
                                params: vec![CSTElpTypeGenericParam {
                                    span: pest::Span::new(expression_str_multiple, 32, 36).unwrap(),
                                    elp_type: CSTElpType {
                                        span: pest::Span::new(expression_str_multiple, 32, 36)
                                            .unwrap(),
                                        mutability: None,
                                        pointer_semantics: None,
                                        value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                            span: pest::Span::new(expression_str_multiple, 32, 36)
                                                .unwrap(),
                                            name: CSTIdent {
                                                span: pest::Span::new(
                                                    expression_str_multiple,
                                                    32,
                                                    36
                                                )
                                                .unwrap(),
                                                value: "JSON".into()
                                            },
                                            generics: vec![]
                                        })
                                    },
                                    type_constraint: None
                                }]
                            }]
                        })
                    }
                ]
            }
        );
    }

    #[test]
    fn object_member_visibility() {
        let expression_str_private = "private";
        let mut private_pairs =
            ElpParser::parse(Rule::visibility_selector, expression_str_private).unwrap();
        let private_ast = VisibilitySelector::from_pest(&mut private_pairs).unwrap();

        assert_eq!(
            private_ast,
            VisibilitySelector::Private(PrivateVisibility {
                span: pest::Span::new(expression_str_private, 0, 7).unwrap()
            })
        );

        let expression_str_public = "public";
        let mut public_pairs =
            ElpParser::parse(Rule::visibility_selector, expression_str_public).unwrap();
        let public_ast = VisibilitySelector::from_pest(&mut public_pairs).unwrap();

        assert_eq!(
            public_ast,
            VisibilitySelector::Public(PublicVisibility {
                span: pest::Span::new(expression_str_public, 0, 6).unwrap()
            })
        );
    }

    #[test]
    fn object_member_tags() {
        let expression_str = "`name: \"example\"`";
        let mut pairs = ElpParser::parse(Rule::object_key_tags, expression_str).unwrap();
        let ast = CSTObjectMemberTags::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTObjectMemberTags {
                span: pest::Span::new(expression_str, 0, 17).unwrap(),
                name: CSTIdent {
                    span: pest::Span::new(expression_str, 1, 5).unwrap(),
                    value: "name".into()
                },
                contents: CSTString {
                    span: pest::Span::new(expression_str, 7, 16).unwrap(),
                    value: "example".into()
                }
            }
        );
    }

    #[test]
    fn basic_object_member() {
        let expression_str = ".name String";
        let mut pairs = ElpParser::parse(Rule::object_member, expression_str).unwrap();
        let ast = CSTObjectMember::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTObjectMember {
                span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                visibility: None,
                name: CSTIdent {
                    span: pest::Span::new(expression_str, 1, 5).unwrap(),
                    value: "name".into()
                },
                type_annotation: Some(CSTElpType {
                    span: pest::Span::new(expression_str, 6, 12).unwrap(),
                    mutability: None,
                    pointer_semantics: None,
                    value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                        span: pest::Span::new(expression_str, 6, expression_str.len()).unwrap(),
                        name: CSTIdent {
                            span: pest::Span::new(expression_str, 6, expression_str.len()).unwrap(),
                            value: "String".into()
                        },
                        generics: vec![]
                    })
                }),
                default_value: None,
                tags: vec![]
            }
        );
    }

    #[test]
    fn private_object_member() {
        let expression_str = "private .name String";
        let mut pairs = ElpParser::parse(Rule::object_member, expression_str).unwrap();
        let ast = CSTObjectMember::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTObjectMember {
                span: pest::Span::new(expression_str, 0, 20).unwrap(),
                visibility: Some(VisibilitySelector::Private(PrivateVisibility {
                    span: pest::Span::new(expression_str, 0, 14).unwrap()
                })),
                name: CSTIdent {
                    span: pest::Span::new(expression_str, 9, 13).unwrap(),
                    value: "name".into()
                },
                type_annotation: Some(CSTElpType {
                    span: pest::Span::new(expression_str, 14, 20).unwrap(),
                    mutability: None,
                    pointer_semantics: None,
                    value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                        span: pest::Span::new(expression_str, 14, 20).unwrap(),
                        name: CSTIdent {
                            span: pest::Span::new(expression_str, 14, 20).unwrap(),
                            value: "String".into()
                        },
                        generics: vec![]
                    })
                }),
                default_value: None,
                tags: vec![]
            }
        );
    }

    #[test]
    fn public_object_member() {
        let expression_str = "public .name String";
        let mut pairs = ElpParser::parse(Rule::object_member, expression_str).unwrap();
        let ast = CSTObjectMember::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTObjectMember {
                span: pest::Span::new(expression_str, 0, 19).unwrap(),
                visibility: Some(VisibilitySelector::Public(PublicVisibility {
                    span: pest::Span::new(expression_str, 0, 13).unwrap()
                })),
                name: CSTIdent {
                    span: pest::Span::new(expression_str, 8, 12).unwrap(),
                    value: "name".into()
                },
                type_annotation: Some(CSTElpType {
                    span: pest::Span::new(expression_str, 13, 19).unwrap(),
                    mutability: None,
                    pointer_semantics: None,
                    value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                        span: pest::Span::new(expression_str, 13, 19).unwrap(),
                        name: CSTIdent {
                            span: pest::Span::new(expression_str, 13, 19).unwrap(),
                            value: "String".into()
                        },
                        generics: vec![]
                    })
                }),
                default_value: None,
                tags: vec![]
            }
        );
    }

    #[test]
    fn tagged_object_member() {
        let expression_str = ".name String `name: \"example\"`";
        let mut pairs = ElpParser::parse(Rule::object_member, expression_str).unwrap();
        let ast = CSTObjectMember::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTObjectMember {
                span: pest::Span::new(expression_str, 0, 30).unwrap(),
                visibility: None,
                name: CSTIdent {
                    span: pest::Span::new(expression_str, 1, 5).unwrap(),
                    value: "name".into()
                },
                type_annotation: Some(CSTElpType {
                    span: pest::Span::new(expression_str, 6, 13).unwrap(),
                    mutability: None,
                    pointer_semantics: None,
                    value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                        span: pest::Span::new(expression_str, 6, 13).unwrap(),
                        name: CSTIdent {
                            span: pest::Span::new(expression_str, 6, 12).unwrap(),
                            value: "String".into()
                        },
                        generics: vec![]
                    })
                }),
                default_value: None,
                tags: vec![CSTObjectMemberTags {
                    span: pest::Span::new(expression_str, 13, 30).unwrap(),
                    name: CSTIdent {
                        span: pest::Span::new(expression_str, 14, 18).unwrap(),
                        value: "name".into()
                    },
                    contents: CSTString {
                        span: pest::Span::new(expression_str, 20, 29).unwrap(),
                        value: "example".into()
                    }
                }]
            }
        );
    }

    #[test]
    fn object_member_default_value() {
        let expression_str = ".name String = \"example\"";
        let mut pairs = ElpParser::parse(Rule::object_member, expression_str).unwrap();
        let ast = CSTObjectMember::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTObjectMember {
                span: pest::Span::new(expression_str, 0, 24).unwrap(),
                visibility: None,
                name: CSTIdent {
                    span: pest::Span::new(expression_str, 1, 5).unwrap(),
                    value: "name".into()
                },
                type_annotation: Some(CSTElpType {
                    span: pest::Span::new(expression_str, 6, 13).unwrap(),
                    mutability: None,
                    pointer_semantics: None,
                    value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                        span: pest::Span::new(expression_str, 6, 13).unwrap(),
                        name: CSTIdent {
                            span: pest::Span::new(expression_str, 6, 12).unwrap(),
                            value: "String".into()
                        },
                        generics: vec![]
                    })
                }),
                default_value: Some(CSTObjectMemberDefaultValue {
                    span: pest::Span::new(expression_str, 13, 24).unwrap(),
                    value: CSTExpression::String(Box::new(CSTString {
                        span: pest::Span::new(expression_str, 15, 24).unwrap(),
                        value: "example".into()
                    }))
                }),
                tags: vec![]
            }
        );
    }

    #[test]
    fn tagged_object_member_with_default_value() {
        let expression_str = ".name String = \"example_default\" `name: \"example\"`";
        let mut pairs = ElpParser::parse(Rule::object_member, expression_str).unwrap();
        let ast = CSTObjectMember::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTObjectMember {
                span: pest::Span::new(expression_str, 0, 50).unwrap(),
                visibility: None,
                name: CSTIdent {
                    span: pest::Span::new(expression_str, 1, 5).unwrap(),
                    value: "name".into()
                },
                type_annotation: Some(CSTElpType {
                    span: pest::Span::new(expression_str, 6, 13).unwrap(),
                    mutability: None,
                    pointer_semantics: None,
                    value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                        span: pest::Span::new(expression_str, 6, 13).unwrap(),
                        name: CSTIdent {
                            span: pest::Span::new(expression_str, 6, 12).unwrap(),
                            value: "String".into()
                        },
                        generics: vec![]
                    })
                }),
                default_value: Some(CSTObjectMemberDefaultValue {
                    span: pest::Span::new(expression_str, 13, 32).unwrap(),
                    value: CSTExpression::String(Box::new(CSTString {
                        span: pest::Span::new(expression_str, 15, 32).unwrap(),
                        value: "example_default".into()
                    }))
                }),
                tags: vec![CSTObjectMemberTags {
                    span: pest::Span::new(expression_str, 33, 50).unwrap(),
                    name: CSTIdent {
                        span: pest::Span::new(expression_str, 34, 38).unwrap(),
                        value: "name".into()
                    },
                    contents: CSTString {
                        span: pest::Span::new(expression_str, 40, 49).unwrap(),
                        value: "example".into()
                    }
                }]
            }
        );
    }

    #[test]
    fn basic_object() {
        let expression_str = "object Test {.name String}";
        let mut pairs = ElpParser::parse(Rule::object_def, expression_str).unwrap();
        let ast = CSTObject::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTObject {
                span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                name: CSTIdent {
                    span: pest::Span::new(expression_str, 7, 11).unwrap(),
                    value: "Test".into()
                },
                implements: None,
                members: vec![CSTObjectMember {
                    span: pest::Span::new(expression_str, 13, 25).unwrap(),
                    visibility: None,
                    name: CSTIdent {
                        span: pest::Span::new(expression_str, 14, 18).unwrap(),
                        value: "name".into()
                    },
                    type_annotation: Some(CSTElpType {
                        span: pest::Span::new(expression_str, 19, 25).unwrap(),
                        mutability: None,
                        pointer_semantics: None,
                        value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                            span: pest::Span::new(expression_str, 19, 25).unwrap(),
                            name: CSTIdent {
                                span: pest::Span::new(expression_str, 19, 25).unwrap(),
                                value: "String".into()
                            },
                            generics: vec![]
                        })
                    }),
                    default_value: None,
                    tags: vec![]
                }],
            }
        );
    }

    #[test]
    fn object_with_implements() {
        let expression_str = "object Test implements MyInterface {.name String}";
        let mut pairs = ElpParser::parse(Rule::object_def, expression_str).unwrap();
        let ast = CSTObject::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTObject {
                span: pest::Span::new(expression_str, 0, 49).unwrap(),
                name: CSTIdent {
                    span: pest::Span::new(expression_str, 7, 11).unwrap(),
                    value: "Test".into()
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
                members: vec![CSTObjectMember {
                    span: pest::Span::new(expression_str, 36, 48).unwrap(),
                    visibility: None,
                    name: CSTIdent {
                        span: pest::Span::new(expression_str, 37, 41).unwrap(),
                        value: "name".into()
                    },
                    type_annotation: Some(CSTElpType {
                        span: pest::Span::new(expression_str, 42, 48).unwrap(),
                        mutability: None,
                        pointer_semantics: None,
                        value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                            span: pest::Span::new(expression_str, 42, 48).unwrap(),
                            name: CSTIdent {
                                span: pest::Span::new(expression_str, 42, 48).unwrap(),
                                value: "String".into()
                            },
                            generics: vec![]
                        })
                    }),
                    default_value: None,
                    tags: vec![]
                }],
            }
        );
    }

    #[test]
    fn object_with_multiple_implements() {
        let expression_str = "object Test implements MyInterface, AnotherInterface {.name String}";
        let mut pairs = ElpParser::parse(Rule::object_def, expression_str).unwrap();
        let ast = CSTObject::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTObject {
                span: pest::Span::new(expression_str, 0, 67).unwrap(),
                name: CSTIdent {
                    span: pest::Span::new(expression_str, 7, 11).unwrap(),
                    value: "Test".into()
                },
                implements: Some(CSTObjectImplements {
                    span: pest::Span::new(expression_str, 12, 53).unwrap(),
                    types: vec![
                        CSTElpType {
                            span: pest::Span::new(expression_str, 23, 34).unwrap(),
                            mutability: None,
                            pointer_semantics: None,
                            value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                span: pest::Span::new(expression_str, 23, 34).unwrap(),
                                name: CSTIdent {
                                    span: pest::Span::new(expression_str, 23, 34).unwrap(),
                                    value: "MyInterface".into()
                                },
                                generics: vec![]
                            })
                        },
                        CSTElpType {
                            span: pest::Span::new(expression_str, 36, 53).unwrap(),
                            mutability: None,
                            pointer_semantics: None,
                            value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                span: pest::Span::new(expression_str, 36, 53).unwrap(),
                                name: CSTIdent {
                                    span: pest::Span::new(expression_str, 36, 52).unwrap(),
                                    value: "AnotherInterface".into()
                                },
                                generics: vec![]
                            })
                        }
                    ]
                }),
                members: vec![CSTObjectMember {
                    span: pest::Span::new(expression_str, 54, 66).unwrap(),
                    visibility: None,
                    name: CSTIdent {
                        span: pest::Span::new(expression_str, 55, 59).unwrap(),
                        value: "name".into()
                    },
                    type_annotation: Some(CSTElpType {
                        span: pest::Span::new(expression_str, 60, 66).unwrap(),
                        mutability: None,
                        pointer_semantics: None,
                        value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                            span: pest::Span::new(expression_str, 60, 66).unwrap(),
                            name: CSTIdent {
                                span: pest::Span::new(expression_str, 60, 66).unwrap(),
                                value: "String".into()
                            },
                            generics: vec![]
                        })
                    }),
                    default_value: None,
                    tags: vec![]
                }],
            }
        );
    }

    #[test]
    fn complex_object() {
        let expression_str = "object Test implements Into<JSON> {
            public     .name String `json:\"name\"`,
            private    .age Int     `json:\"age\"`,
            .friends   Vec<Friend>  `json:\"friends\"`,
            .studentId = 123        `json:\"studentId\"`
    }";
        let mut pairs = ElpParser::parse(Rule::object_def, expression_str).unwrap();
        let ast = CSTObject::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTObject {
                span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                name: CSTIdent {
                    span: pest::Span::new(expression_str, 7, 11).unwrap(),
                    value: "Test".into()
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
                                value: "Into".into()
                            },
                            generics: vec![CSTElpTypeGeneric {
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
                                                span: pest::Span::new(expression_str, 28, 32)
                                                    .unwrap(),
                                                value: "JSON".into()
                                            },
                                            generics: vec![]
                                        })
                                    },
                                    type_constraint: None
                                }]
                            }]
                        })
                    }]
                }),
                members: vec![
                    CSTObjectMember {
                        span: pest::Span::new(expression_str, 48, 85).unwrap(),
                        visibility: Some(VisibilitySelector::Public(PublicVisibility {
                            span: pest::Span::new(expression_str, 48, 54).unwrap(),
                        })),
                        name: CSTIdent {
                            span: pest::Span::new(expression_str, 60, 64).unwrap(),
                            value: "name".into()
                        },
                        type_annotation: Some(CSTElpType {
                            span: pest::Span::new(expression_str, 65, 72).unwrap(),
                            mutability: None,
                            pointer_semantics: None,
                            value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                span: pest::Span::new(expression_str, 65, 72).unwrap(),
                                name: CSTIdent {
                                    span: pest::Span::new(expression_str, 65, 71).unwrap(),
                                    value: "String".into()
                                },
                                generics: vec![]
                            })
                        }),
                        default_value: None,
                        tags: vec![CSTObjectMemberTags {
                            span: pest::Span::new(expression_str, 72, 85).unwrap(),
                            name: CSTIdent {
                                span: pest::Span::new(expression_str, 73, 77).unwrap(),
                                value: "json".into()
                            },
                            contents: CSTString {
                                span: pest::Span::new(expression_str, 78, 84).unwrap(),
                                value: "name".into()
                            }
                        }]
                    },
                    CSTObjectMember {
                        span: pest::Span::new(expression_str, 99, 135).unwrap(),
                        visibility: Some(VisibilitySelector::Private(PrivateVisibility {
                            span: pest::Span::new(expression_str, 99, 106).unwrap(),
                        })),
                        name: CSTIdent {
                            span: pest::Span::new(expression_str, 111, 114).unwrap(),
                            value: "age".into()
                        },
                        type_annotation: Some(CSTElpType {
                            span: pest::Span::new(expression_str, 115, 123).unwrap(),
                            mutability: None,
                            pointer_semantics: None,
                            value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                span: pest::Span::new(expression_str, 115, 123).unwrap(),
                                name: CSTIdent {
                                    span: pest::Span::new(expression_str, 115, 118).unwrap(),
                                    value: "Int".into()
                                },
                                generics: vec![]
                            })
                        }),
                        default_value: None,
                        tags: vec![CSTObjectMemberTags {
                            span: pest::Span::new(expression_str, 123, 135).unwrap(),
                            name: CSTIdent {
                                span: pest::Span::new(expression_str, 124, 128).unwrap(),
                                value: "json".into()
                            },
                            contents: CSTString {
                                span: pest::Span::new(expression_str, 129, 134).unwrap(),
                                value: "age".into()
                            }
                        }],
                    },
                    CSTObjectMember {
                        span: pest::Span::new(expression_str, 149, 189).unwrap(),
                        visibility: None,
                        name: CSTIdent {
                            span: pest::Span::new(expression_str, 150, 157).unwrap(),
                            value: "friends".into()
                        },
                        type_annotation: Some(CSTElpType {
                            span: pest::Span::new(expression_str, 160, 171).unwrap(),
                            mutability: None,
                            pointer_semantics: None,
                            value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                span: pest::Span::new(expression_str, 160, 171).unwrap(),
                                name: CSTIdent {
                                    span: pest::Span::new(expression_str, 160, 163).unwrap(),
                                    value: "Vec".into()
                                },
                                generics: vec![CSTElpTypeGeneric {
                                    span: pest::Span::new(expression_str, 163, 171).unwrap(),
                                    params: vec![CSTElpTypeGenericParam {
                                        span: pest::Span::new(expression_str, 164, 170).unwrap(),
                                        elp_type: CSTElpType {
                                            span: pest::Span::new(expression_str, 164, 170)
                                                .unwrap(),
                                            mutability: None,
                                            pointer_semantics: None,
                                            value: CSTElpTypeValue::Parameter(
                                                CSTElpTypeParameter {
                                                    span: pest::Span::new(expression_str, 164, 170)
                                                        .unwrap(),
                                                    name: CSTIdent {
                                                        span: pest::Span::new(
                                                            expression_str,
                                                            164,
                                                            170
                                                        )
                                                        .unwrap(),
                                                        value: "Friend".into()
                                                    },
                                                    generics: vec![]
                                                }
                                            )
                                        },
                                        type_constraint: None
                                    }]
                                }]
                            })
                        }),
                        default_value: None,
                        tags: vec![CSTObjectMemberTags {
                            span: pest::Span::new(expression_str, 173, 189).unwrap(),
                            name: CSTIdent {
                                span: pest::Span::new(expression_str, 174, 178).unwrap(),
                                value: "json".into()
                            },
                            contents: CSTString {
                                span: pest::Span::new(expression_str, 179, 188).unwrap(),
                                value: "friends".into()
                            }
                        }]
                    },
                    CSTObjectMember {
                        span: pest::Span::new(expression_str, 203, 245).unwrap(),
                        visibility: None,
                        name: CSTIdent {
                            span: pest::Span::new(expression_str, 204, 213).unwrap(),
                            value: "studentId".into()
                        },
                        type_annotation: None,
                        default_value: Some(CSTObjectMemberDefaultValue {
                            span: pest::Span::new(expression_str, 214, 219).unwrap(),
                            value: CSTExpression::Number(Box::new(CSTNumber {
                                span: pest::Span::new(expression_str, 216, 219).unwrap(),
                                value: "123".into()
                            }))
                        }),
                        tags: vec![CSTObjectMemberTags {
                            span: pest::Span::new(expression_str, 227, 245).unwrap(),
                            name: CSTIdent {
                                span: pest::Span::new(expression_str, 228, 232).unwrap(),
                                value: "json".into()
                            },
                            contents: CSTString {
                                span: pest::Span::new(expression_str, 233, 244).unwrap(),
                                value: "studentId".into()
                            }
                        }]
                    }
                ]
            }
        );
    }
}
