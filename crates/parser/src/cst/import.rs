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

    #[test]
    fn single_import_ast_generation() {
        let expression_str = "import {Bar, Baz as BazAlias} from \"foo\"";
        let mut pairs = ElpParser::parse(Rule::import, expression_str).unwrap();
        let ast = CSTImport::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTImport {
                span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                names: vec![
                    CSTImportName {
                        span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                        name: CSTIdent {
                            span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                            value: "Bar".into()
                        },
                        alias: None,
                    },
                    CSTImportName {
                        span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                        name: CSTIdent {
                            span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                            value: "Baz".to_string()
                        },
                        alias: Some(CSTImportNameAlias {
                            span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                            alias: CSTIdent {
                                span: pest::Span::new(expression_str, 0, expression_str.len())
                                    .unwrap(),
                                value: "BazAlias".into()
                            }
                        }),
                    }
                ],
                module_path: CSTImportModulePath {
                    span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                    module_path: CSTString {
                        span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                        value: "foo".into()
                    }
                }
            }
        )
    }
}
