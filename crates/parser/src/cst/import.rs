use super::{ident::CSTIdent, string::CSTString};
use crate::parser::Rule;
use pest::Span;
use pest_ast::FromPest;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::import))]
pub struct CSTImport<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
    pub names: Vec<CSTImportName<'a>>,
    pub module_path: CSTImportModulePath<'a>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::import_name))]
pub struct CSTImportName<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
    pub name: CSTIdent<'a>,
    pub alias: Option<CSTImportNameAlias<'a>>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::import_name_alias))]
pub struct CSTImportNameAlias<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
    pub alias: CSTIdent<'a>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::import_module_path))]
pub struct CSTImportModulePath<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
    pub module_path: CSTString<'a>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ElpParser;
    use from_pest::FromPest;
    use pest::Parser;
    use pretty_assertions::assert_eq;

    #[test]
    fn single_import_ast_generation() {
        let expression_str = "import {Bar, Baz as BazAlias} from \"foo\"";
        let mut pairs = ElpParser::parse(Rule::import, expression_str).unwrap();
        let ast = CSTImport::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTImport {
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
                                value: "BazAlias".into()
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
            }
        )
    }
}
