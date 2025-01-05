pub(crate) mod block;
pub(crate) mod elp_type;
pub(crate) mod r#enum;
pub(crate) mod export;
pub(crate) mod expression;
pub(crate) mod function;
pub(crate) mod ident;
pub(crate) mod import;
pub(crate) mod number_value;
pub(crate) mod object;
pub(crate) mod string;
pub(crate) mod value_assignment;
pub(crate) mod variable_access;
pub(crate) mod variable_assignment;
pub(crate) mod variable_declaration;

use expression::Expression;
use pest::Span;
use pest_ast::FromPest;

use crate::parser::Rule;

fn span_into_string(span: Span) -> String {
    span.as_str().into()
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::module))]
pub struct Module {
    pub expressions: Vec<Expression>,
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
    use import::Import;
    use pest::Parser;
    use string::StringValue;

    use crate::ast::import::{ImportModulePath, ImportName, ImportNameAlias};

    #[test]
    fn single_expression_ast_generation() {
        let expression_str = "import {Bar, Baz as BazAlias} from \"foo\"";
        let mut pairs = ElpParser::parse(Rule::module, expression_str).unwrap();
        let ast = Module::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            Module {
                expressions: vec![Expression::Import(Box::new(Import {
                    names: vec![
                        ImportName {
                            name: "Bar".into(),
                            alias: None,
                        },
                        ImportName {
                            name: "Baz".to_string(),
                            alias: Some(ImportNameAlias {
                                alias: "BazAlias".into()
                            }),
                        }
                    ],
                    module_path: ImportModulePath {
                        module_path: StringValue {
                            value: "foo".into()
                        }
                    }
                })),],
                _eoi: Eoi {}
            }
        )
    }
}
