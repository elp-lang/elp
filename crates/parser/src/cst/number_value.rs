use pest::Span;
use pest_ast::FromPest;

use crate::parser::Rule;

use super::span_into_string;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::number))]
pub struct CSTNumber<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
    // Numbers in elp are similar to numbers in JavaScript where they can appear in multiple forms.
    // For example, -1, 10, 10.5, 1e3, etc.
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
    fn single_number_ast_generation() {
        let number_str = "10";
        let mut pairs = ElpParser::parse(Rule::number, number_str).unwrap();
        let ast = CSTNumber::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTNumber {
                span: pest::Span::new(number_str, 0, number_str.len()).unwrap(),
                value: "10".into()
            }
        )
    }

    #[test]
    fn number_with_decimal_part_ast_generation() {
        let number_str = "10.5";
        let mut pairs = ElpParser::parse(Rule::number, number_str).unwrap();
        let ast = CSTNumber::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTNumber {
                span: pest::Span::new(number_str, 0, number_str.len()).unwrap(),
                value: "10.5".into()
            }
        )
    }

    #[test]
    fn number_with_exponent_ast_generation() {
        let number_str = "1e3";
        let mut pairs = ElpParser::parse(Rule::number, number_str).unwrap();
        let ast = CSTNumber::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTNumber {
                span: pest::Span::new(number_str, 0, number_str.len()).unwrap(),
                value: "1e3".into()
            }
        )
    }
}
