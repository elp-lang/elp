use crate::cst::function::{
    CSTFunctionArgument, CSTFunctionArguments, CSTFunctionCall, CSTFunctionCallName,
    CSTFunctionDef, CSTFunctionHeaderDef, CSTFunctionReturnType, CSTFunctionReturnValue,
};

use super::{
    block::ASTBlock,
    elp_type::{ASTElpType, ASTPointerSemantics},
    expression::ASTExpression,
    traits::FromCST,
    variable_access::{ASTContextualVariableAccess, ASTVariableAccess},
};

#[derive(Debug, PartialEq, Clone)]
pub struct ASTFunctionArgument<'a> {
    pub span: &'a pest::Span<'a>,
    pub pointer_semantics: Option<ASTPointerSemantics>,
    pub name: String,
    pub type_annotation: Option<ASTElpType<'a>>,
}

impl<'a> FromCST<'a, CSTFunctionArgument<'a>> for ASTFunctionArgument<'a> {
    fn from_cst(cst: &'a CSTFunctionArgument<'a>) -> Self {
        Self {
            span: &cst.span,
            pointer_semantics: cst
                .pointer_semantics
                .as_ref()
                .map(ASTPointerSemantics::from_cst),
            name: cst.name.value.clone(),
            type_annotation: cst.type_annotation.as_ref().map(ASTElpType::from_cst),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ASTFunctionArguments<'a> {
    pub span: &'a pest::Span<'a>,
    pub arguments: Vec<ASTFunctionArgument<'a>>,
}

impl<'a> FromCST<'a, CSTFunctionArguments<'a>> for ASTFunctionArguments<'a> {
    fn from_cst(cst: &'a CSTFunctionArguments<'a>) -> Self {
        Self {
            span: &cst.span,
            arguments: cst
                .arguments
                .iter()
                .map(ASTFunctionArgument::from_cst)
                .collect(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ASTFunctionReturnType<'a> {
    pub span: &'a pest::Span<'a>,
    pub type_annotations: Vec<ASTElpType<'a>>,
}

impl<'a> FromCST<'a, CSTFunctionReturnType<'a>> for ASTFunctionReturnType<'a> {
    fn from_cst(cst: &'a CSTFunctionReturnType<'a>) -> Self {
        Self {
            span: &cst.span,
            type_annotations: cst
                .type_annotations
                .iter()
                .map(ASTElpType::from_cst)
                .collect(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ASTFunctionReturnValue<'a> {
    pub span: &'a pest::Span<'a>,
    pub value: Box<ASTExpression<'a>>,
}

impl<'a> FromCST<'a, CSTFunctionReturnValue<'a>> for ASTFunctionReturnValue<'a> {
    fn from_cst(cst: &'a CSTFunctionReturnValue<'a>) -> Self {
        Self {
            span: &cst.span,
            value: Box::new(ASTExpression::from_cst(&cst.value)),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ASTFunctionDef<'a> {
    pub span: &'a pest::Span<'a>,
    pub name: ASTVariableAccess<'a>,
    pub generics: Vec<ASTElpType<'a>>,
    pub arguments: Option<ASTFunctionArguments<'a>>,
    pub return_type: Option<ASTFunctionReturnType<'a>>,
    pub block: Box<ASTBlock<'a>>,
}

impl<'a> FromCST<'a, CSTFunctionDef<'a>> for ASTFunctionDef<'a> {
    fn from_cst(cst: &'a CSTFunctionDef<'a>) -> Self {
        Self {
            span: &cst.span,
            name: ASTVariableAccess::from_cst(&cst.name),
            generics: match &cst.generics {
                Some(generic) => generic.params.iter().map(ASTElpType::from_cst).collect(),
                None => vec![],
            },
            arguments: cst.arguments.as_ref().map(ASTFunctionArguments::from_cst),
            return_type: cst
                .return_type
                .as_ref()
                .map(ASTFunctionReturnType::from_cst),
            block: Box::new(ASTBlock::from_cst(&cst.block)),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ASTFunctionHeaderDef<'a> {
    pub span: &'a pest::Span<'a>,
    pub pointer_semantics: Option<ASTPointerSemantics>,
    pub name: ASTVariableAccess<'a>,
    pub generics: Vec<ASTElpType<'a>>,
    pub arguments: ASTFunctionArguments<'a>,
    pub return_type: ASTFunctionReturnType<'a>,
}

impl<'a> FromCST<'a, CSTFunctionHeaderDef<'a>> for ASTFunctionHeaderDef<'a> {
    fn from_cst(cst: &'a CSTFunctionHeaderDef<'a>) -> Self {
        Self {
            span: &cst.span,
            pointer_semantics: cst
                .pointer_semantics
                .as_ref()
                .map(ASTPointerSemantics::from_cst),
            name: ASTVariableAccess::from_cst(&cst.name),
            generics: match &cst.generics {
                Some(generic) => generic.params.iter().map(ASTElpType::from_cst).collect(),
                None => vec![],
            },
            arguments: ASTFunctionArguments::from_cst(&cst.arguments),
            return_type: ASTFunctionReturnType::from_cst(&cst.return_type),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ASTFunctionCallName<'a> {
    VariableAccess(ASTVariableAccess<'a>),
    ContextualVariableAccess(ASTContextualVariableAccess<'a>),
}

impl<'a> FromCST<'a, CSTFunctionCallName<'a>> for ASTFunctionCallName<'a> {
    fn from_cst(cst: &'a CSTFunctionCallName<'a>) -> Self {
        match cst {
            CSTFunctionCallName::VariableAccess(access) => {
                ASTFunctionCallName::VariableAccess(ASTVariableAccess::from_cst(access))
            }
            CSTFunctionCallName::ContextualVariableAccess(access) => {
                ASTFunctionCallName::ContextualVariableAccess(
                    ASTContextualVariableAccess::from_cst(access),
                )
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ASTFunctionCall<'a> {
    pub span: &'a pest::Span<'a>,
    pub name: ASTFunctionCallName<'a>,
    pub generics: Vec<ASTElpType<'a>>,
    pub arguments: Vec<ASTExpression<'a>>,
}

impl<'a> FromCST<'a, CSTFunctionCall<'a>> for ASTFunctionCall<'a> {
    fn from_cst(cst: &'a CSTFunctionCall<'a>) -> Self {
        Self {
            span: &cst.span,
            name: ASTFunctionCallName::from_cst(&cst.name),
            generics: match &cst.generics {
                Some(generic) => generic.params.iter().map(ASTElpType::from_cst).collect(),
                None => vec![],
            },
            arguments: cst.arguments.iter().map(ASTExpression::from_cst).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::{
        ast::{elp_type::ASTMutability, string::ASTString},
        cst::{
            block::CSTBlock,
            elp_type::{CSTElpType, CSTElpTypeParameter, CSTElpTypeValue},
            expression::CSTExpression,
            ident::CSTIdent,
            string::CSTString,
            variable_access::{CSTVariableAccess, CSTVariableAccessNames},
        },
    };

    use super::*;

    #[test]
    fn simple_function_def_to_ast() {
        let expression_str = "fn hello.name(name String) -> String { return \"hello {name}\" }";
        let cst = CSTFunctionDef {
            span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
            name: CSTVariableAccess {
                span: pest::Span::new(expression_str, 3, 13).unwrap(),
                names: CSTVariableAccessNames {
                    span: pest::Span::new(expression_str, 3, 13).unwrap(),
                    names: vec![
                        CSTIdent {
                            span: pest::Span::new(expression_str, 3, 8).unwrap(),
                            value: "hello".into(),
                        },
                        CSTIdent {
                            span: pest::Span::new(expression_str, 9, 13).unwrap(),
                            value: "name".into(),
                        },
                    ],
                },
                pointer_semantics: vec![],
            },
            generics: None,
            arguments: Some(CSTFunctionArguments {
                span: pest::Span::new(expression_str, 13, 26).unwrap(),
                arguments: vec![CSTFunctionArgument {
                    span: pest::Span::new(expression_str, 14, 25).unwrap(),
                    name: CSTIdent {
                        span: pest::Span::new(expression_str, 14, 18).unwrap(),
                        value: "name".into(),
                    },
                    pointer_semantics: None,
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
                }],
            }),
            return_type: Some(CSTFunctionReturnType {
                span: pest::Span::new(expression_str, 27, 37).unwrap(),
                type_annotations: vec![CSTElpType {
                    span: pest::Span::new(expression_str, 30, 37).unwrap(),
                    mutability: None,
                    pointer_semantics: None,
                    value: CSTElpTypeValue::Parameter(CSTElpTypeParameter {
                        span: pest::Span::new(expression_str, 30, 37).unwrap(),
                        name: CSTIdent {
                            span: pest::Span::new(expression_str, 30, 36).unwrap(),
                            value: "String".into(),
                        },
                        generics: None,
                    }),
                }],
            }),
            block: Box::new(CSTBlock {
                span: pest::Span::new(expression_str, 37, expression_str.len()).unwrap(),
                expressions: vec![CSTExpression::FunctionReturnValue(Box::new(
                    CSTFunctionReturnValue {
                        span: pest::Span::new(expression_str, 39, 60).unwrap(),
                        value: Box::new(CSTExpression::String(Box::new(CSTString {
                            span: pest::Span::new(expression_str, 46, 60).unwrap(),
                            value: "hello {name}".into(),
                        }))),
                    },
                ))],
            }),
        };
        let ast = ASTFunctionDef::from_cst(&cst);

        assert_eq!(
            ast,
            ASTFunctionDef {
                span: &pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                name: ASTVariableAccess {
                    span: &pest::Span::new(expression_str, 3, 13).unwrap(),
                    names: vec!["hello".into(), "name".into()],
                    pointer_semantics: vec![],
                },
                generics: vec![],
                arguments: Some(ASTFunctionArguments {
                    span: &pest::Span::new(expression_str, 13, 26).unwrap(),
                    arguments: vec![ASTFunctionArgument {
                        span: &pest::Span::new(expression_str, 14, 25).unwrap(),
                        name: "name".into(),
                        pointer_semantics: None,
                        type_annotation: Some(ASTElpType {
                            span: &pest::Span::new(expression_str, 19, 25).unwrap(),
                            mutability: ASTMutability::Immutable,
                            pointer_semantics: None,
                            generic_parameters: vec![],
                            name: "String".into(),
                            type_constraints: vec![]
                        }),
                    }],
                }),
                return_type: Some(ASTFunctionReturnType {
                    span: &pest::Span::new(expression_str, 27, 37).unwrap(),
                    type_annotations: vec![ASTElpType {
                        span: &pest::Span::new(expression_str, 30, 37).unwrap(),
                        mutability: ASTMutability::Immutable,
                        pointer_semantics: None,
                        name: "String".into(),
                        generic_parameters: vec![],
                        type_constraints: vec![]
                    }],
                }),
                block: Box::new(ASTBlock {
                    span: &pest::Span::new(expression_str, 37, expression_str.len()).unwrap(),
                    expressions: vec![ASTExpression::FunctionReturnValue(Box::new(
                        ASTFunctionReturnValue {
                            span: &pest::Span::new(expression_str, 39, 60).unwrap(),
                            value: Box::new(ASTExpression::String(Box::new(ASTString {
                                span: &pest::Span::new(expression_str, 46, 60).unwrap(),
                                value: "hello {name}".into()
                            })))
                        }
                    ))]
                })
            }
        )
    }
}
