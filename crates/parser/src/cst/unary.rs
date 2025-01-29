use pest::Span;
use pest_ast::FromPest;

use crate::parser::Rule;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::UNARY_OPERATOR))]
pub enum CSTUnaryOperator<'a> {
    Plus(CSTUnaryPlus<'a>),
    Minus(CSTUnaryMinus<'a>),
    Multiply(CSTUnaryMultiply<'a>),
    Divide(CSTUnaryDivide<'a>),
    Power(CSTUnaryPower<'a>),
    Modulo(CSTUnaryModulo<'a>),
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::UNARY_ADD))]
pub struct CSTUnaryPlus<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::UNARY_SUB))]
pub struct CSTUnaryMinus<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::UNARY_MULT))]
pub struct CSTUnaryMultiply<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::UNARY_DIV))]
pub struct CSTUnaryDivide<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::UNARY_POW))]
pub struct CSTUnaryPower<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::UNARY_MOD))]
pub struct CSTUnaryModulo<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ElpParser;
    use from_pest::FromPest;
    use pest::Parser;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_unary_operator() {
        let plus_expression_str = "+";
        let mut plus_pairs = ElpParser::parse(Rule::UNARY_OPERATOR, plus_expression_str).unwrap();
        let plus_ast = CSTUnaryOperator::from_pest(&mut plus_pairs).unwrap();
        assert_eq!(
            plus_ast,
            CSTUnaryOperator::Plus(CSTUnaryPlus {
                span: pest::Span::new(plus_expression_str, 0, plus_expression_str.len()).unwrap(),
            })
        );

        let minus_expression_str = "-";
        let mut minus_pairs = ElpParser::parse(Rule::UNARY_OPERATOR, minus_expression_str).unwrap();
        let minus_ast = CSTUnaryOperator::from_pest(&mut minus_pairs).unwrap();
        assert_eq!(
            minus_ast,
            CSTUnaryOperator::Minus(CSTUnaryMinus {
                span: pest::Span::new(plus_expression_str, 0, plus_expression_str.len()).unwrap(),
            })
        );

        let multiply_expression_str = "*";
        let mut multiply_pairs =
            ElpParser::parse(Rule::UNARY_OPERATOR, multiply_expression_str).unwrap();
        let multiply_ast = CSTUnaryOperator::from_pest(&mut multiply_pairs).unwrap();
        assert_eq!(
            multiply_ast,
            CSTUnaryOperator::Multiply(CSTUnaryMultiply {
                span: pest::Span::new(plus_expression_str, 0, plus_expression_str.len()).unwrap(),
            })
        );

        let divide_expression_str = "/";
        let mut divide_pairs =
            ElpParser::parse(Rule::UNARY_OPERATOR, divide_expression_str).unwrap();
        let divide_ast = CSTUnaryOperator::from_pest(&mut divide_pairs).unwrap();
        assert_eq!(
            divide_ast,
            CSTUnaryOperator::Divide(CSTUnaryDivide {
                span: pest::Span::new(plus_expression_str, 0, plus_expression_str.len()).unwrap(),
            })
        );

        let power_expression_str = "^";
        let mut power_pairs = ElpParser::parse(Rule::UNARY_OPERATOR, power_expression_str).unwrap();
        let power_ast = CSTUnaryOperator::from_pest(&mut power_pairs).unwrap();
        assert_eq!(
            power_ast,
            CSTUnaryOperator::Power(CSTUnaryPower {
                span: pest::Span::new(plus_expression_str, 0, plus_expression_str.len()).unwrap(),
            })
        );

        let modulo_expression_str = "%";
        let mut modulo_pairs =
            ElpParser::parse(Rule::UNARY_OPERATOR, modulo_expression_str).unwrap();
        let modulo_ast = CSTUnaryOperator::from_pest(&mut modulo_pairs).unwrap();
        assert_eq!(
            modulo_ast,
            CSTUnaryOperator::Modulo(CSTUnaryModulo {
                span: pest::Span::new(plus_expression_str, 0, plus_expression_str.len()).unwrap(),
            })
        );
    }
}
