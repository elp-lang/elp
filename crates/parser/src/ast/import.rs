use crate::cst::import::CSTImport;

use super::{module::ASTModule, traits::FromCST};

#[derive(Debug, PartialEq, Clone)]
pub struct ASTImport<'a> {
    pub span: &'a pest::Span<'a>,
    pub names: Vec<(String, Option<String>)>,
    pub module_path: String,
    // This gets expanded after preprocs are expanded.
    pub module: Option<ASTModule<'a>>,
}

impl<'a> FromCST<'a, CSTImport<'a>> for ASTImport<'a> {
    fn from_cst(cst: &'a CSTImport) -> Self {
        ASTImport {
            span: &cst.span,
            names: cst
                .names
                .iter()
                .map(|n| {
                    if let Some(alias) = &n.alias {
                        (n.name.value.clone(), Some(alias.alias.value.clone()))
                    } else {
                        (n.name.value.clone(), None)
                    }
                })
                .collect(),
            module_path: cst.module_path.module_path.value.clone(),
            module: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::cst::{
        ident::CSTIdent,
        import::{CSTImportModulePath, CSTImportName, CSTImportNameAlias},
        string::CSTString,
    };

    use super::*;

    #[test]
    fn test_from_cst() {
        let expression_str = "import {name, aliasme as newname} from \"test-module\"";
        let cst_import = CSTImport {
            span: pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
            module_path: CSTImportModulePath {
                span: pest::Span::new(expression_str, 35, 40).unwrap(),
                module_path: CSTString {
                    span: pest::Span::new(expression_str, 35, 40).unwrap(),
                    value: "test-module".into(),
                },
            },
            names: vec![
                CSTImportName {
                    span: pest::Span::new(expression_str, 8, 12).unwrap(),
                    name: CSTIdent {
                        span: pest::Span::new(expression_str, 8, 12).unwrap(),
                        value: "name".into(),
                    },
                    alias: None,
                },
                CSTImportName {
                    span: pest::Span::new(expression_str, 13, 28).unwrap(),
                    name: CSTIdent {
                        span: pest::Span::new(expression_str, 13, 16).unwrap(),
                        value: "aliasme".into(),
                    },
                    alias: Some(CSTImportNameAlias {
                        span: pest::Span::new(expression_str, 17, 28).unwrap(),
                        alias: CSTIdent {
                            span: pest::Span::new(expression_str, 20, 28).unwrap(),
                            value: "newname".into(),
                        },
                    }),
                },
            ],
        };

        let ast_import = ASTImport::from_cst(&cst_import);

        assert_eq!(
            ast_import,
            ASTImport {
                span: &pest::Span::new(expression_str, 0, expression_str.len()).unwrap(),
                names: vec![
                    ("name".to_string(), None),
                    ("aliasme".to_string(), Some("newname".to_string())),
                ],
                module_path: "test-module".to_string(),
                module: None,
            }
        )
    }
}
