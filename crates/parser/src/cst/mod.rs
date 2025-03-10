pub(crate) mod block;
pub(crate) mod component;
pub(crate) mod elp_type;
pub(crate) mod r#enum;
pub(crate) mod export;
pub(crate) mod expression;
pub(crate) mod for_loop;
pub(crate) mod function;
pub(crate) mod ident;
pub(crate) mod import;
pub(crate) mod interface;
pub(crate) mod r#match;
pub(crate) mod number_value;
pub(crate) mod object;
pub(crate) mod string;
pub(crate) mod unary;
pub(crate) mod value_assignment;
pub(crate) mod variable_access;
pub(crate) mod variable_assignment;
pub(crate) mod variable_declaration;

use expression::CSTExpression;
use pest::Span;
use pest_ast::FromPest;

use crate::parser::Rule;

#[derive(Debug, FromPest, PartialEq, Eq, Clone)]
#[pest_ast(rule(Rule::VAR))]
pub struct Var<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
}

#[derive(Debug, FromPest, PartialEq, Eq, Clone)]
#[pest_ast(rule(Rule::CONST))]
pub struct Const<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
}

#[derive(Debug, FromPest, PartialEq, Eq, Clone)]
#[pest_ast(rule(Rule::mutability_selector))]
pub enum CSTMutabilitySelector<'a> {
    Mutable(Var<'a>),
    Immutable(Const<'a>),
}

#[derive(Debug, FromPest, PartialEq, Eq, Clone)]
#[pest_ast(rule(Rule::PUBLIC))]
pub struct CSTPublicVisibility<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
}

#[derive(Debug, FromPest, PartialEq, Eq, Clone)]
#[pest_ast(rule(Rule::PRIVATE))]
pub struct CSTPrivateVisibility<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
}

#[derive(Debug, FromPest, PartialEq, Eq, Clone)]
#[pest_ast(rule(Rule::visibility_selector))]
pub enum CSTVisibilitySelector<'a> {
    Public(CSTPublicVisibility<'a>),
    Private(CSTPrivateVisibility<'a>),
}

fn span_into_string(span: Span) -> String {
    span.as_str().into()
}

#[derive(Debug, FromPest, PartialEq, Eq, Clone)]
#[pest_ast(rule(Rule::module))]
pub struct CSTModule<'a> {
    pub expressions: Vec<CSTExpression<'a>>,
    _eoi: Eoi,
}

#[derive(Debug, FromPest, PartialEq, Eq, Clone)]
#[pest_ast(rule(Rule::EOI))]
struct Eoi;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ElpParser;
    use from_pest::FromPest;
    use ident::CSTIdent;
    use import::CSTImport;
    use pest::Parser;
    use pretty_assertions::assert_eq;
    use string::CSTString;
    use value_assignment::{CSTEquals, CSTOperand, CSTValueAssignment};
    use variable_assignment::{CSTVariableAssignment, CSTVariableAssignmentTarget};

    use crate::cst::import::{CSTImportModulePath, CSTImportName, CSTImportNameAlias};

    #[test]
    fn single_expression_ast_generation() {
        let expression_str = "import {Bar, Baz as BazAlias} from \"foo\"";
        let mut pairs = ElpParser::parse(Rule::module, expression_str).unwrap();
        let ast = CSTModule::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTModule {
                expressions: vec![CSTExpression::Import(Box::new(CSTImport {
                    span: pest::Span::new(expression_str, 0, 40).unwrap(),
                    names: vec![
                        CSTImportName {
                            span: pest::Span::new(expression_str, 8, 11).unwrap(),
                            name: CSTIdent {
                                span: pest::Span::new(expression_str, 8, 11).unwrap(),
                                value: "Bar".into()
                            },
                            alias: None,
                        },
                        CSTImportName {
                            span: pest::Span::new(expression_str, 13, 28).unwrap(),
                            name: CSTIdent {
                                span: pest::Span::new(expression_str, 13, 16).unwrap(),
                                value: "Baz".to_string()
                            },
                            alias: Some(CSTImportNameAlias {
                                span: pest::Span::new(expression_str, 17, 28).unwrap(),
                                alias: CSTIdent {
                                    span: pest::Span::new(expression_str, 20, 28).unwrap(),
                                    value: "BazAlias".into(),
                                }
                            }),
                        }
                    ],
                    module_path: CSTImportModulePath {
                        span: pest::Span::new(expression_str, 35, 40).unwrap(),
                        module_path: CSTString {
                            span: pest::Span::new(expression_str, 35, 40).unwrap(),
                            value: "foo".into()
                        }
                    }
                })),],
                _eoi: Eoi {}
            }
        )
    }

    #[test]
    fn const_declaration_ast_generation() {
        let expression_str = "const a = 1";
        let mut pairs = ElpParser::parse(Rule::module, expression_str).unwrap();
        let ast = CSTModule::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTModule {
                expressions: vec![CSTExpression::VariableAssignment(Box::new(
                    CSTVariableAssignment {
                        span: pest::Span::new(expression_str, 0, 11).unwrap(),
                        value_assignment: CSTValueAssignment {
                            span: pest::Span::new(expression_str, 8, 11).unwrap(),
                            operand: CSTOperand::Equals(CSTEquals {
                                span: pest::Span::new(expression_str, 8, 9).unwrap(),
                            }),
                            value: Box::new(CSTExpression::Number(Box::new(
                                number_value::CSTNumber {
                                    span: pest::Span::new(expression_str, 10, 11).unwrap(),
                                    value: "1".into()
                                }
                            )))
                        },
                        variable_assignment_target:
                            CSTVariableAssignmentTarget::VariableDeclaration(
                                variable_declaration::CSTVariableDeclaration {
                                    span: pest::Span::new(expression_str, 0, 8).unwrap(),
                                    mutability: CSTMutabilitySelector::Immutable(Const {
                                        span: pest::Span::new(expression_str, 0, 5).unwrap(),
                                    }),
                                    name: CSTIdent {
                                        span: pest::Span::new(expression_str, 6, 7).unwrap(),
                                        value: "a".into()
                                    },
                                    type_annotation: None,
                                }
                            )
                    }
                ))],
                _eoi: Eoi {}
            }
        )
    }

    #[test]
    fn variable_declaration_cst_generation() {
        let expression_str = "var a = 1";
        let mut pairs = ElpParser::parse(Rule::module, expression_str).unwrap();
        let ast = CSTModule::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTModule {
                expressions: vec![CSTExpression::VariableAssignment(Box::new(
                    CSTVariableAssignment {
                        span: pest::Span::new(expression_str, 0, 9).unwrap(),
                        value_assignment: CSTValueAssignment {
                            span: pest::Span::new(expression_str, 6, 9).unwrap(),
                            operand: CSTOperand::Equals(CSTEquals {
                                span: pest::Span::new(expression_str, 6, 7).unwrap(),
                            }),
                            value: Box::new(CSTExpression::Number(Box::new(
                                number_value::CSTNumber {
                                    span: pest::Span::new(expression_str, 8, 9).unwrap(),
                                    value: "1".into()
                                }
                            )))
                        },
                        variable_assignment_target:
                            CSTVariableAssignmentTarget::VariableDeclaration(
                                variable_declaration::CSTVariableDeclaration {
                                    span: pest::Span::new(expression_str, 0, 6).unwrap(),
                                    mutability: CSTMutabilitySelector::Mutable(Var {
                                        span: pest::Span::new(expression_str, 0, 3).unwrap(),
                                    }),
                                    name: CSTIdent {
                                        span: pest::Span::new(expression_str, 4, 5).unwrap(),
                                        value: "a".into()
                                    },
                                    type_annotation: None,
                                }
                            )
                    }
                ))],
                _eoi: Eoi {}
            }
        )
    }
}
