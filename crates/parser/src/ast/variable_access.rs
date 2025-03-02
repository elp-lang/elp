use pest::Span;

use crate::cst::variable_access::{
    CSTContextualVariableAccess, CSTVariableAccess, CSTVariableAccessNames,
};

use super::{
    elp_type::{ASTElpType, ASTPointerSemantics},
    traits::FromCST,
};

#[derive(Debug, PartialEq, Clone)]
pub struct ASTVariableAccess<'a> {
    pub span: &'a Span<'a>,
    pub pointer_semantics: Vec<ASTPointerSemantics>,
    pub names: ASTVariableAccessNames<'a>,
}

impl<'a> FromCST<'a, CSTVariableAccess<'a>> for ASTVariableAccess<'a> {
    fn from_cst(cst: &'a CSTVariableAccess<'a>) -> Self {
        Self {
            span: &cst.span,
            pointer_semantics: cst
                .pointer_semantics
                .iter()
                .map(ASTPointerSemantics::from_cst)
                .collect(),
            names: ASTVariableAccessNames::from_cst(&cst.names),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ASTVariableAccessNames<'a> {
    pub span: &'a Span<'a>,
    pub names: Vec<String>,
}

impl<'a> FromCST<'a, CSTVariableAccessNames<'a>> for ASTVariableAccessNames<'a> {
    fn from_cst(cst: &'a CSTVariableAccessNames<'a>) -> Self {
        Self {
            span: &cst.span,
            names: cst.names.iter().map(|s| s.value.clone()).collect(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ASTContextualVariableAccess<'a> {
    pub span: &'a pest::Span<'a>,
    pub context_type: Option<ASTElpType<'a>>,
    pub name: String,
}

impl<'a> FromCST<'a, CSTContextualVariableAccess<'a>> for ASTContextualVariableAccess<'a> {
    fn from_cst(cst: &'a CSTContextualVariableAccess<'a>) -> Self {
        Self {
            span: &cst.span,
            // @TODO: We can only assume context type later and via a "Variable Access" and not from contextual access.
            context_type: None,
            name: cst.name.value.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::cst::{ident::CSTIdent, variable_access::CSTContextualVariableAccess};

    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn test_variable_access() {
        let expression_str = "hello.world.my.name.is.dave";
        let cst = CSTVariableAccess {
            span: pest::Span::new(expression_str, 0, 27).unwrap(),
            pointer_semantics: vec![],
            names: CSTVariableAccessNames {
                span: pest::Span::new(expression_str, 0, 27).unwrap(),
                names: vec![
                    CSTIdent {
                        span: pest::Span::new(expression_str, 0, 5).unwrap(),
                        value: "hello".into(),
                    },
                    CSTIdent {
                        span: pest::Span::new(expression_str, 6, 11).unwrap(),
                        value: "world".into(),
                    },
                    CSTIdent {
                        span: pest::Span::new(expression_str, 12, 14).unwrap(),
                        value: "my".into(),
                    },
                    CSTIdent {
                        span: pest::Span::new(expression_str, 15, 19).unwrap(),
                        value: "name".into(),
                    },
                    CSTIdent {
                        span: pest::Span::new(expression_str, 20, 22).unwrap(),
                        value: "is".into(),
                    },
                    CSTIdent {
                        span: pest::Span::new(expression_str, 23, 27).unwrap(),
                        value: "dave".into(),
                    },
                ],
            },
        };

        let ast = ASTVariableAccess::from_cst(&cst);

        assert_eq!(
            ast,
            ASTVariableAccess {
                span: &pest::Span::new(expression_str, 0, 27).unwrap(),
                pointer_semantics: vec![],
                names: ASTVariableAccessNames {
                    span: &pest::Span::new(expression_str, 0, 27).unwrap(),
                    names: vec![
                        "hello".to_string(),
                        "world".to_string(),
                        "my".to_string(),
                        "name".to_string(),
                        "is".to_string(),
                        "dave".to_string(),
                    ],
                }
            }
        )
    }

    #[test]
    fn contextual_variable_access() {
        let expression_str_pointer = ".CONTEXTUAL";
        let cst = CSTContextualVariableAccess {
            span: pest::Span::new(expression_str_pointer, 0, 11).unwrap(),
            name: CSTIdent {
                span: pest::Span::new(expression_str_pointer, 1, 11).unwrap(),
                value: "CONTEXTUAL".into(),
            },
        };

        let ast = ASTContextualVariableAccess::from_cst(&cst);

        assert_eq!(
            ast,
            ASTContextualVariableAccess {
                span: &pest::Span::new(expression_str_pointer, 0, 11).unwrap(),
                context_type: None,
                name: "CONTEXTUAL".to_string(),
            }
        )
    }
}
