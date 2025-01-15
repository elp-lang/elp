use super::{span_into_string, string::CSTString};
use crate::parser::Rule;
use pest_ast::FromPest;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::import))]
pub struct CSTImport {
    pub names: Vec<CSTImportName>,
    pub module_path: CSTImportModulePath,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::import_name))]
pub struct CSTImportName {
    #[pest_ast(inner(with(span_into_string)))]
    pub name: String,
    pub alias: Option<CSTImportNameAlias>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::import_name_alias))]
pub struct CSTImportNameAlias {
    #[pest_ast(inner(with(span_into_string)))]
    pub alias: String,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::import_module_path))]
pub struct CSTImportModulePath {
    pub module_path: CSTString,
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
            }
        )
    }
}
