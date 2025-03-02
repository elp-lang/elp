use crate::cst::string::CSTString;

use super::traits::FromCST;

#[derive(Debug, PartialEq, Clone)]
pub struct ASTString<'a> {
    pub span: &'a pest::Span<'a>,
    pub value: String,
}

impl<'a> FromCST<'a, CSTString<'a>> for ASTString<'a> {
    fn from_cst(cst: &'a CSTString<'a>) -> Self {
        Self {
            span: &cst.span,
            value: cst.value.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn string_from_cst() {
        let cst_string = CSTString {
            span: pest::Span::new("", 0, 0).unwrap(),
            value: "Hello, world!".to_string(),
        };
        let ast_string = ASTString::from_cst(&cst_string);
        assert_eq!(
            ast_string,
            ASTString {
                span: &cst_string.span,
                value: "Hello, world!".to_string(),
            }
        );
    }
}
