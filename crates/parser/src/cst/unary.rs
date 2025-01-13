use pest_ast::FromPest;

use crate::parser::Rule;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::UNARY_OPERATOR))]
pub enum UnaryOperator {
    Plus(UnaryPlus),
    Minus(UnaryMinus),
    Multiply(UnaryMultiply),
    Divide(UnaryDivide),
    Power(UnaryPower),
    Modulo(UnaryModulo),
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::UNARY_ADD))]
pub struct UnaryPlus;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::UNARY_SUB))]
pub struct UnaryMinus;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::UNARY_MULT))]
pub struct UnaryMultiply;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::UNARY_DIV))]
pub struct UnaryDivide;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::UNARY_POW))]
pub struct UnaryPower;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::UNARY_MOD))]
pub struct UnaryModulo;

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
        let plus_ast = UnaryOperator::from_pest(&mut plus_pairs).unwrap();
        assert_eq!(plus_ast, UnaryOperator::Plus(UnaryPlus {}));

        let minus_expression_str = "-";
        let mut minus_pairs = ElpParser::parse(Rule::UNARY_OPERATOR, minus_expression_str).unwrap();
        let minus_ast = UnaryOperator::from_pest(&mut minus_pairs).unwrap();
        assert_eq!(minus_ast, UnaryOperator::Minus(UnaryMinus {}));

        let multiply_expression_str = "*";
        let mut multiply_pairs =
            ElpParser::parse(Rule::UNARY_OPERATOR, multiply_expression_str).unwrap();
        let multiply_ast = UnaryOperator::from_pest(&mut multiply_pairs).unwrap();
        assert_eq!(multiply_ast, UnaryOperator::Multiply(UnaryMultiply {}));

        let divide_expression_str = "/";
        let mut divide_pairs =
            ElpParser::parse(Rule::UNARY_OPERATOR, divide_expression_str).unwrap();
        let divide_ast = UnaryOperator::from_pest(&mut divide_pairs).unwrap();
        assert_eq!(divide_ast, UnaryOperator::Divide(UnaryDivide {}));

        let power_expression_str = "^";
        let mut power_pairs = ElpParser::parse(Rule::UNARY_OPERATOR, power_expression_str).unwrap();
        let power_ast = UnaryOperator::from_pest(&mut power_pairs).unwrap();
        assert_eq!(power_ast, UnaryOperator::Power(UnaryPower {}));

        let modulo_expression_str = "%";
        let mut modulo_pairs =
            ElpParser::parse(Rule::UNARY_OPERATOR, modulo_expression_str).unwrap();
        let modulo_ast = UnaryOperator::from_pest(&mut modulo_pairs).unwrap();
        assert_eq!(modulo_ast, UnaryOperator::Modulo(UnaryModulo {}));
    }
}
