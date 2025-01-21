use crate::cst::span_into_string;
use crate::parser::Rule;
use pest_ast::FromPest;

#[derive(Debug, FromPest, PartialEq, Eq, Copy)]
#[pest_ast(rule(Rule::IDENT))]
pub struct CSTIdent {
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
        let ast = CSTIdent::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTIdent {
                value: "hello".into(),
            }
        )
    }
}
