#![allow(clippy::clone_on_copy)]
// We allow the clone on copy rule in this file because outer calls clone on a Copy-able object
// which is actually a merged fix but not in the release.

use crate::ast::span_into_string;
use crate::parser::Rule;
use pest_ast::FromPest;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::IDENT))]
pub struct Ident {
    #[pest_ast(outer(with(span_into_string)))]
    pub value: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ElpParser;
    use from_pest::FromPest;
    use pest::Parser;

    #[test]
    fn test_ident() {
        let expression_str = "hello";
        let mut pairs = ElpParser::parse(Rule::IDENT, expression_str).unwrap();
        let ast = Ident::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            Ident {
                value: "hello".into(),
            }
        )
    }
}
