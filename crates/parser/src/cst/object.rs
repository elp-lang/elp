use super::{
    elp_type::CSTElpType, expression::CSTExpression, ident::CSTIdent, string::CSTString,
    VisibilitySelector,
};
use crate::parser::Rule;
use pest_ast::FromPest;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::object_implements))]
pub struct CSTObjectImplements {
    pub types: Vec<CSTElpType>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::object_key_default_value))]
pub struct CSTObjectMemberDefaultValue {
    pub value: CSTExpression,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::object_key_tags))]
pub struct CSTObjectMemberTags {
    pub name: CSTIdent,
    pub contents: CSTString,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::object_member))]
pub struct CSTObjectMember {
    pub visibility: Option<VisibilitySelector>,
    pub name: CSTIdent,
    pub type_annotation: Option<CSTElpType>,
    pub default_value: Option<CSTObjectMemberDefaultValue>,
    pub tags: Vec<CSTObjectMemberTags>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::object_def))]
pub struct CSTObject {
    pub name: CSTIdent,
    pub implements: Option<CSTObjectImplements>,
    pub members: Vec<CSTObjectMember>,
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
                types: vec![CSTElpType {
                    mutability: None,
                    pointer_semantics: None,
                    value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                        name: CSTIdent {
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
                types: vec![
                    CSTElpType {
                        mutability: None,
                        pointer_semantics: None,
                        value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                            name: CSTIdent {
                                value: "String".into()
                            },
                            generics: vec![],
                        })
                    },
                    CSTElpType {
                        mutability: None,
                        pointer_semantics: None,
                        value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                            name: CSTIdent {
                                value: "Number".into()
                            },
                            generics: vec![],
                        })
                    },
                    CSTElpType {
                        mutability: None,
                        pointer_semantics: None,
                        value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                            name: CSTIdent {
                                value: "Into".into()
                            },
                            generics: vec![CSTElpTypeGeneric {
                                params: vec![CSTElpTypeGenericParam {
                                    elp_type: CSTElpType {
                                        mutability: None,
                                        pointer_semantics: None,
                                        value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                            name: CSTIdent {
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
            VisibilitySelector::Private(PrivateVisibility {})
        );

        let expression_str_public = "public";
        let mut public_pairs =
            ElpParser::parse(Rule::visibility_selector, expression_str_public).unwrap();
        let public_ast = VisibilitySelector::from_pest(&mut public_pairs).unwrap();

        assert_eq!(public_ast, VisibilitySelector::Public(PublicVisibility {}));
    }

    #[test]
    fn object_member_tags() {
        let expression_str = "`name: \"example\"`";
        let mut pairs = ElpParser::parse(Rule::object_key_tags, expression_str).unwrap();
        let ast = CSTObjectMemberTags::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTObjectMemberTags {
                name: CSTIdent {
                    value: "name".into()
                },
                contents: CSTString {
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
                visibility: None,
                name: CSTIdent {
                    value: "name".into()
                },
                type_annotation: Some(CSTElpType {
                    mutability: None,
                    pointer_semantics: None,
                    value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                        name: CSTIdent {
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
                visibility: Some(VisibilitySelector::Private(PrivateVisibility {})),
                name: CSTIdent {
                    value: "name".into()
                },
                type_annotation: Some(CSTElpType {
                    mutability: None,
                    pointer_semantics: None,
                    value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                        name: CSTIdent {
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
                visibility: Some(VisibilitySelector::Public(PublicVisibility {})),
                name: CSTIdent {
                    value: "name".into()
                },
                type_annotation: Some(CSTElpType {
                    mutability: None,
                    pointer_semantics: None,
                    value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                        name: CSTIdent {
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
                visibility: None,
                name: CSTIdent {
                    value: "name".into()
                },
                type_annotation: Some(CSTElpType {
                    mutability: None,
                    pointer_semantics: None,
                    value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                        name: CSTIdent {
                            value: "String".into()
                        },
                        generics: vec![]
                    })
                }),
                default_value: None,
                tags: vec![CSTObjectMemberTags {
                    name: CSTIdent {
                        value: "name".into()
                    },
                    contents: CSTString {
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
                visibility: None,
                name: CSTIdent {
                    value: "name".into()
                },
                type_annotation: Some(CSTElpType {
                    mutability: None,
                    pointer_semantics: None,
                    value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                        name: CSTIdent {
                            value: "String".into()
                        },
                        generics: vec![]
                    })
                }),
                default_value: Some(CSTObjectMemberDefaultValue {
                    value: CSTExpression::String(Box::new(CSTString {
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
                visibility: None,
                name: CSTIdent {
                    value: "name".into()
                },
                type_annotation: Some(CSTElpType {
                    mutability: None,
                    pointer_semantics: None,
                    value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                        name: CSTIdent {
                            value: "String".into()
                        },
                        generics: vec![]
                    })
                }),
                default_value: Some(CSTObjectMemberDefaultValue {
                    value: CSTExpression::String(Box::new(CSTString {
                        value: "example_default".into()
                    }))
                }),
                tags: vec![CSTObjectMemberTags {
                    name: CSTIdent {
                        value: "name".into()
                    },
                    contents: CSTString {
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
                name: CSTIdent {
                    value: "Test".into()
                },
                implements: None,
                members: vec![CSTObjectMember {
                    visibility: None,
                    name: CSTIdent {
                        value: "name".into()
                    },
                    type_annotation: Some(CSTElpType {
                        mutability: None,
                        pointer_semantics: None,
                        value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                            name: CSTIdent {
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
                name: CSTIdent {
                    value: "Test".into()
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
                members: vec![CSTObjectMember {
                    visibility: None,
                    name: CSTIdent {
                        value: "name".into()
                    },
                    type_annotation: Some(CSTElpType {
                        mutability: None,
                        pointer_semantics: None,
                        value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                            name: CSTIdent {
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
                name: CSTIdent {
                    value: "Test".into()
                },
                implements: Some(CSTObjectImplements {
                    types: vec![
                        CSTElpType {
                            mutability: None,
                            pointer_semantics: None,
                            value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                name: CSTIdent {
                                    value: "MyInterface".into()
                                },
                                generics: vec![]
                            })
                        },
                        CSTElpType {
                            mutability: None,
                            pointer_semantics: None,
                            value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                name: CSTIdent {
                                    value: "AnotherInterface".into()
                                },
                                generics: vec![]
                            })
                        }
                    ]
                }),
                members: vec![CSTObjectMember {
                    visibility: None,
                    name: CSTIdent {
                        value: "name".into()
                    },
                    type_annotation: Some(CSTElpType {
                        mutability: None,
                        pointer_semantics: None,
                        value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                            name: CSTIdent {
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
                name: CSTIdent {
                    value: "Test".into()
                },
                implements: Some(CSTObjectImplements {
                    types: vec![CSTElpType {
                        mutability: None,
                        pointer_semantics: None,
                        value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                            name: CSTIdent {
                                value: "Into".into()
                            },
                            generics: vec![CSTElpTypeGeneric {
                                params: vec![CSTElpTypeGenericParam {
                                    elp_type: CSTElpType {
                                        mutability: None,
                                        pointer_semantics: None,
                                        value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                            name: CSTIdent {
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
                        visibility: Some(VisibilitySelector::Public(PublicVisibility {})),
                        name: CSTIdent {
                            value: "name".into()
                        },
                        type_annotation: Some(CSTElpType {
                            mutability: None,
                            pointer_semantics: None,
                            value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                name: CSTIdent {
                                    value: "String".into()
                                },
                                generics: vec![]
                            })
                        }),
                        default_value: None,
                        tags: vec![CSTObjectMemberTags {
                            name: CSTIdent {
                                value: "json".into()
                            },
                            contents: CSTString {
                                value: "name".into()
                            }
                        }]
                    },
                    CSTObjectMember {
                        visibility: Some(VisibilitySelector::Private(PrivateVisibility {})),
                        name: CSTIdent {
                            value: "age".into()
                        },
                        type_annotation: Some(CSTElpType {
                            mutability: None,
                            pointer_semantics: None,
                            value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                name: CSTIdent {
                                    value: "Int".into()
                                },
                                generics: vec![]
                            })
                        }),
                        default_value: None,
                        tags: vec![CSTObjectMemberTags {
                            name: CSTIdent {
                                value: "json".into()
                            },
                            contents: CSTString {
                                value: "age".into()
                            }
                        }],
                    },
                    CSTObjectMember {
                        visibility: None,
                        name: CSTIdent {
                            value: "friends".into()
                        },
                        type_annotation: Some(CSTElpType {
                            mutability: None,
                            pointer_semantics: None,
                            value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                                name: CSTIdent {
                                    value: "Vec".into()
                                },
                                generics: vec![CSTElpTypeGeneric {
                                    params: vec![CSTElpTypeGenericParam {
                                        elp_type: CSTElpType {
                                            mutability: None,
                                            pointer_semantics: None,
                                            value: CSTElpTypeValue::Parameter(
                                                CSTElpTypeParameter {
                                                    name: CSTIdent {
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
                            name: CSTIdent {
                                value: "json".into()
                            },
                            contents: CSTString {
                                value: "friends".into()
                            }
                        }]
                    },
                    CSTObjectMember {
                        visibility: None,
                        name: CSTIdent {
                            value: "studentId".into()
                        },
                        type_annotation: None,
                        default_value: Some(CSTObjectMemberDefaultValue {
                            value: CSTExpression::Number(Box::new(CSTNumber {
                                value: "123".into()
                            }))
                        }),
                        tags: vec![CSTObjectMemberTags {
                            name: CSTIdent {
                                value: "json".into()
                            },
                            contents: CSTString {
                                value: "studentId".into()
                            }
                        }]
                    }
                ]
            }
        );
    }
}
