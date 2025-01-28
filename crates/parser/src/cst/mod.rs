pub(crate) mod block;
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
use pest::{LinesSpan, Span};
use pest_ast::FromPest;

use crate::parser::Rule;

#[derive(Debug, FromPest, PartialEq, Eq, Clone)]
#[pest_ast(rule(Rule::VAR))]
pub struct Var;

#[derive(Debug, FromPest, PartialEq, Eq, Clone)]
#[pest_ast(rule(Rule::CONST))]
pub struct Const;

#[derive(Debug, FromPest, PartialEq, Eq, Clone)]
#[pest_ast(rule(Rule::mutability_selector))]
pub enum CSTMutabilitySelector {
    Mutable(Var),
    Immutable(Const),
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::PUBLIC))]
pub struct PublicVisibility;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::PRIVATE))]
pub struct PrivateVisibility;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::visibility_selector))]
pub enum VisibilitySelector {
    Public(PublicVisibility),
    Private(PrivateVisibility),
}

fn span_into_string(span: Span) -> String {
    span.as_str().into()
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::module))]
pub struct Module {
    pub expressions: Vec<CSTExpression>,
    _eoi: Eoi,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::EOI))]
struct Eoi;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ElpParser;
    use from_pest::FromPest;
    use import::CSTImport;
    use pest::Parser;
    use string::CSTString;

    use crate::cst::import::{CSTImportModulePath, CSTImportName, CSTImportNameAlias};

    #[test]
    fn single_expression_ast_generation() {
        let expression_str = "import {Bar, Baz as BazAlias} from \"foo\"";
        let mut pairs = ElpParser::parse(Rule::module, expression_str).unwrap();
        let ast = Module::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            Module {
                expressions: vec![CSTExpression::Import(Box::new(CSTImport {
                    names: vec![
                        CSTImportName {
                            name: "Bar".into(),
                            alias: None,
                        },
                        CSTImportName {
                            name: "Baz".to_string(),
                            alias: Some(CSTImportNameAlias {
                                alias: "BazAlias".into()
                            }),
                        }
                    ],
                    module_path: CSTImportModulePath {
                        module_path: CSTString {
                            value: "foo".into()
                        }
                    }
                })),],
                _eoi: Eoi {}
            }
        )
    }
}
