use crate::cst::import::CSTImport;

use super::{module::ASTModule, traits::FromCST};

#[derive(Debug, PartialEq, Clone)]
pub struct ASTImport<'a> {
    pub names: Vec<(String, Option<String>)>,
    pub module_path: String,
    pub module: Option<ASTModule<'a>>,
}

impl<'a> FromCST<'a, CSTImport<'a>> for ASTImport<'a> {
    fn from_cst(cst: &'a CSTImport) -> Self {
        ASTImport {
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
