use super::{span_into_string, StringValue};
use crate::parser::Rule;
use pest_ast::FromPest;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::import))]
pub struct Import {
    pub names: Vec<ImportName>,
    pub module_path: ImportModulePath,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::import_name))]
pub struct ImportName {
    #[pest_ast(inner(with(span_into_string)))]
    pub name: String,
    pub alias: Option<ImportNameAlias>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::import_name_alias))]
pub struct ImportNameAlias {
    #[pest_ast(inner(with(span_into_string)))]
    pub alias: String,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::import_module_path))]
pub struct ImportModulePath {
    pub module_path: StringValue,
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
        let ast = Import::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            Import {
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
            }
        )
    }
}
