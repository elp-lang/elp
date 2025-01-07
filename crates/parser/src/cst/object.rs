use super::{
    elp_type::ElpType, expression::Expression, ident::Ident, string::StringValue,
    VisibilitySelector,
};
use crate::parser::Rule;
use pest_ast::FromPest;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::object_implements))]
pub struct ObjectImplements {
    pub types: Vec<ElpType>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::object_key_default_value))]
pub struct ObjectMemberDefaultValue {
    pub value: Expression,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::object_key_tags))]
pub struct ObjectMemberTags {
    pub name: Ident,
    pub contents: StringValue,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::object_member))]
pub struct ObjectMember {
    pub visibility: Option<VisibilitySelector>,
    pub name: Ident,
    pub type_annotation: Option<ElpType>,
    pub default_value: Option<ObjectMemberDefaultValue>,
    pub tags: Vec<ObjectMemberTags>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::object_def))]
pub struct Object {
    pub name: Ident,
    pub implements: Option<ObjectImplements>,
    pub members: Vec<ObjectMember>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cst::{
            elp_type::{ElpTypeGeneric, ElpTypeGenericParam},
            number_value::Number,
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
        let ast_basic = ObjectImplements::from_pest(&mut pairs_basic).unwrap();

        assert_eq!(
            ast_basic,
            ObjectImplements {
                types: vec![ElpType {
                    mutability: None,
                    name: Ident {
                        value: "String".into()
                    },
                    generics: vec![],
                }]
            }
        );

        let expression_str_multiple = "implements String, Number, Into<JSON>";
        let mut pairs_multiple =
            ElpParser::parse(Rule::object_implements, expression_str_multiple).unwrap();
        let ast_multiple = ObjectImplements::from_pest(&mut pairs_multiple).unwrap();

        assert_eq!(
            ast_multiple,
            ObjectImplements {
                types: vec![
                    ElpType {
                        mutability: None,
                        name: Ident {
                            value: "String".into()
                        },
                        generics: vec![],
                    },
                    ElpType {
                        mutability: None,
                        name: Ident {
                            value: "Number".into()
                        },
                        generics: vec![],
                    },
                    ElpType {
                        mutability: None,
                        name: Ident {
                            value: "Into".into()
                        },
                        generics: vec![ElpTypeGeneric {
                            params: vec![ElpTypeGenericParam {
                                elp_type: ElpType {
                                    mutability: None,
                                    name: Ident {
                                        value: "JSON".into()
                                    },
                                    generics: vec![]
                                },
                                type_constraint: None
                            }]
                        }]
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
        let ast = ObjectMemberTags::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            ObjectMemberTags {
                name: Ident {
                    value: "name".into()
                },
                contents: StringValue {
                    value: "example".into()
                }
            }
        );
    }

    #[test]
    fn basic_object_member() {
        let expression_str = ".name String";
        let mut pairs = ElpParser::parse(Rule::object_member, expression_str).unwrap();
        let ast = ObjectMember::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            ObjectMember {
                visibility: None,
                name: Ident {
                    value: "name".into()
                },
                type_annotation: Some(ElpType {
                    mutability: None,
                    name: Ident {
                        value: "String".into()
                    },
                    generics: vec![]
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
        let ast = ObjectMember::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            ObjectMember {
                visibility: Some(VisibilitySelector::Private(PrivateVisibility {})),
                name: Ident {
                    value: "name".into()
                },
                type_annotation: Some(ElpType {
                    mutability: None,
                    name: Ident {
                        value: "String".into()
                    },
                    generics: vec![]
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
        let ast = ObjectMember::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            ObjectMember {
                visibility: Some(VisibilitySelector::Public(PublicVisibility {})),
                name: Ident {
                    value: "name".into()
                },
                type_annotation: Some(ElpType {
                    mutability: None,
                    name: Ident {
                        value: "String".into()
                    },
                    generics: vec![]
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
        let ast = ObjectMember::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            ObjectMember {
                visibility: None,
                name: Ident {
                    value: "name".into()
                },
                type_annotation: Some(ElpType {
                    mutability: None,
                    name: Ident {
                        value: "String".into()
                    },
                    generics: vec![]
                }),
                default_value: None,
                tags: vec![ObjectMemberTags {
                    name: Ident {
                        value: "name".into()
                    },
                    contents: StringValue {
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
        let ast = ObjectMember::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            ObjectMember {
                visibility: None,
                name: Ident {
                    value: "name".into()
                },
                type_annotation: Some(ElpType {
                    mutability: None,
                    name: Ident {
                        value: "String".into()
                    },
                    generics: vec![]
                }),
                default_value: Some(ObjectMemberDefaultValue {
                    value: Expression::String(Box::new(StringValue {
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
        let ast = ObjectMember::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            ObjectMember {
                visibility: None,
                name: Ident {
                    value: "name".into()
                },
                type_annotation: Some(ElpType {
                    mutability: None,
                    name: Ident {
                        value: "String".into()
                    },
                    generics: vec![]
                }),
                default_value: Some(ObjectMemberDefaultValue {
                    value: Expression::String(Box::new(StringValue {
                        value: "example_default".into()
                    }))
                }),
                tags: vec![ObjectMemberTags {
                    name: Ident {
                        value: "name".into()
                    },
                    contents: StringValue {
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
        let ast = Object::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            Object {
                name: Ident {
                    value: "Test".into()
                },
                implements: None,
                members: vec![ObjectMember {
                    visibility: None,
                    name: Ident {
                        value: "name".into()
                    },
                    type_annotation: Some(ElpType {
                        mutability: None,
                        name: Ident {
                            value: "String".into()
                        },
                        generics: vec![]
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
        let ast = Object::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            Object {
                name: Ident {
                    value: "Test".into()
                },
                implements: Some(ObjectImplements {
                    types: vec![ElpType {
                        mutability: None,
                        name: Ident {
                            value: "MyInterface".into()
                        },
                        generics: vec![]
                    }]
                }),
                members: vec![ObjectMember {
                    visibility: None,
                    name: Ident {
                        value: "name".into()
                    },
                    type_annotation: Some(ElpType {
                        mutability: None,
                        name: Ident {
                            value: "String".into()
                        },
                        generics: vec![]
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
        let ast = Object::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            Object {
                name: Ident {
                    value: "Test".into()
                },
                implements: Some(ObjectImplements {
                    types: vec![
                        ElpType {
                            mutability: None,
                            name: Ident {
                                value: "MyInterface".into()
                            },
                            generics: vec![]
                        },
                        ElpType {
                            mutability: None,
                            name: Ident {
                                value: "AnotherInterface".into()
                            },
                            generics: vec![]
                        }
                    ]
                }),
                members: vec![ObjectMember {
                    visibility: None,
                    name: Ident {
                        value: "name".into()
                    },
                    type_annotation: Some(ElpType {
                        mutability: None,
                        name: Ident {
                            value: "String".into()
                        },
                        generics: vec![]
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
        let ast = Object::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            Object {
                name: Ident {
                    value: "Test".into()
                },
                implements: Some(ObjectImplements {
                    types: vec![ElpType {
                        mutability: None,
                        name: Ident {
                            value: "Into".into()
                        },
                        generics: vec![ElpTypeGeneric {
                            params: vec![ElpTypeGenericParam {
                                elp_type: ElpType {
                                    mutability: None,
                                    name: Ident {
                                        value: "JSON".into()
                                    },
                                    generics: vec![]
                                },
                                type_constraint: None
                            }]
                        }]
                    }]
                }),
                members: vec![
                    ObjectMember {
                        visibility: Some(VisibilitySelector::Public(PublicVisibility {})),
                        name: Ident {
                            value: "name".into()
                        },
                        type_annotation: Some(ElpType {
                            mutability: None,
                            name: Ident {
                                value: "String".into()
                            },
                            generics: vec![]
                        }),
                        default_value: None,
                        tags: vec![ObjectMemberTags {
                            name: Ident {
                                value: "json".into()
                            },
                            contents: StringValue {
                                value: "name".into()
                            }
                        }]
                    },
                    ObjectMember {
                        visibility: Some(VisibilitySelector::Private(PrivateVisibility {})),
                        name: Ident {
                            value: "age".into()
                        },
                        type_annotation: Some(ElpType {
                            mutability: None,
                            name: Ident {
                                value: "Int".into()
                            },
                            generics: vec![]
                        }),
                        default_value: None,
                        tags: vec![ObjectMemberTags {
                            name: Ident {
                                value: "json".into()
                            },
                            contents: StringValue {
                                value: "age".into()
                            }
                        }],
                    },
                    ObjectMember {
                        visibility: None,
                        name: Ident {
                            value: "friends".into()
                        },
                        type_annotation: Some(ElpType {
                            mutability: None,
                            name: Ident {
                                value: "Vec".into()
                            },
                            generics: vec![ElpTypeGeneric {
                                params: vec![ElpTypeGenericParam {
                                    elp_type: ElpType {
                                        mutability: None,
                                        name: Ident {
                                            value: "Friend".into()
                                        },
                                        generics: vec![]
                                    },
                                    type_constraint: None
                                }]
                            }]
                        }),
                        default_value: None,
                        tags: vec![ObjectMemberTags {
                            name: Ident {
                                value: "json".into()
                            },
                            contents: StringValue {
                                value: "friends".into()
                            }
                        }]
                    },
                    ObjectMember {
                        visibility: None,
                        name: Ident {
                            value: "studentId".into()
                        },
                        type_annotation: None,
                        default_value: Some(ObjectMemberDefaultValue {
                            value: Expression::Number(Box::new(Number {
                                value: "123".into()
                            }))
                        }),
                        tags: vec![ObjectMemberTags {
                            name: Ident {
                                value: "json".into()
                            },
                            contents: StringValue {
                                value: "studentId".into()
                            }
                        }]
                    }
                ]
            }
        );
    }
}
