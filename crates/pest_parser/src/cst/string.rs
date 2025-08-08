use pest::Span;
use pest_ast::FromPest;

use crate::parser::Rule;

use super::span_into_string;

#[derive(Debug, FromPest, PartialEq, Eq, Clone)]
#[pest_ast(rule(Rule::string))]
pub struct CSTString<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
    #[pest_ast(inner(with(span_into_string)))]
    pub value: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ElpParser;
    use from_pest::FromPest;
    use pest::Parser;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_pointer_semantics() {
        let ref_expression_str = "\"hello world\"";
        let mut ref_pairs = ElpParser::parse(Rule::string, ref_expression_str).unwrap();
        let ref_ast = CSTString::from_pest(&mut ref_pairs).unwrap();

        assert_eq!(
            ref_ast,
            CSTString {
                span: pest::Span::new(ref_expression_str, 0, ref_expression_str.len()).unwrap(),
                value: "hello world".into()
            }
        );
    }
}
