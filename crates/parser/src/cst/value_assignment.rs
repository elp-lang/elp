use pest_ast::FromPest;

use crate::parser::Rule;

use super::{expression::CSTExpression, variable_access::Reference};

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::BITWISE_OPERAND_TILDE))]
pub struct Tilde;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::BITWISE_OPERAND_LEFT_SHIFT))]
pub struct LeftShift;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::BITWISE_OPERAND_RIGHT_SHIFT))]
pub struct RightShift;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::BITWISE_OPERAND_OR))]
pub struct Or;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::BITWISE_OPERAND))]
pub enum BitwiseOperand {
    Tilde(Tilde),
    LeftShift(LeftShift),
    RightShift(RightShift),
    BitOr(Or),
    BitAnd(Reference),
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::OPERAND_PLUS))]
pub struct Plus;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::OPERAND_MINUS))]
pub struct Minus;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::OPERAND_MUL))]
pub struct Multiply;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::OPERAND_DIV))]
pub struct Divide;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::OPERAND_MOD))]
pub struct Modulo;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::OPERAND_POW))]
pub struct Power;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::OPERAND_NOT_EQUAL))]
pub struct EqualityNot;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::OPERAND_BIT_NOT_EQUAL))]
pub struct EqualityBitNot;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::OPERAND_EQUALS))]
pub struct Equals;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::OPERAND_EQUAL))]
pub struct EqualityEqual;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::OPERAND_BITAND))]
pub struct BitAnd;

#[derive(Debug, PartialEq, Eq, FromPest)]
#[pest_ast(rule(Rule::value_assignment))]
pub struct ValueAssignment {
    pub operand: Operand,
    pub value: Box<CSTExpression>,
}

#[derive(Debug, PartialEq, FromPest, Eq)]
#[pest_ast(rule(Rule::OPERAND))]
pub enum Operand {
    Plus(Plus),
    Minus(Minus),
    Multiply(Multiply),
    Divide(Divide),
    Modulo(Modulo),
    Power(Power),
    EqualityNot(EqualityNot),
    EqualityBitNot(EqualityBitNot),
    Equals(Equals),
    EqualityEqual(EqualityEqual),
    BitAnd(BitAnd),
}

#[cfg(test)]
mod tests {
    use from_pest::FromPest;
    use pest::Parser;

    use crate::{
        cst::{number_value::Number, string::StringValue},
        parser::ElpParser,
    };

    use super::*;

    #[test]
    fn test_value_assignment_equals() {
        let expression_str = "= \"world\"";
        let mut pairs = ElpParser::parse(Rule::value_assignment, expression_str).unwrap();
        let ast = ValueAssignment::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            ValueAssignment {
                operand: Operand::Equals(Equals {}),
                value: Box::new(CSTExpression::String(Box::new(StringValue {
                    value: "world".into(),
                }))),
            }
        )
    }

    #[test]
    fn test_value_assignment_plus() {
        let expression_str = "+= 1";
        let mut pairs = ElpParser::parse(Rule::value_assignment, expression_str).unwrap();
        let ast = ValueAssignment::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            ValueAssignment {
                operand: Operand::Plus(Plus {}),
                value: Box::new(CSTExpression::Number(Box::new(Number {
                    value: "1".into()
                }))),
            }
        )
    }

    #[test]
    fn test_value_assignment_modulo() {
        let expression_str = "%= 1";
        let mut pairs = ElpParser::parse(Rule::value_assignment, expression_str).unwrap();
        let ast = ValueAssignment::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            ValueAssignment {
                operand: Operand::Modulo(Modulo {}),
                value: Box::new(CSTExpression::Number(Box::new(Number {
                    value: "1".into()
                }))),
            }
        )
    }

    #[test]
    fn test_value_assignment_equality_equal() {
        let expression_str = "== 2";
        let mut pairs = ElpParser::parse(Rule::value_assignment, expression_str).unwrap();
        let ast = ValueAssignment::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            ValueAssignment {
                operand: Operand::EqualityEqual(EqualityEqual {}),
                value: Box::new(CSTExpression::Number(Box::new(Number {
                    value: "2".into()
                }))),
            }
        )
    }

    #[test]
    fn test_value_assignment_equality_not() {
        let expression_str = "!= 2";
        let mut pairs = ElpParser::parse(Rule::value_assignment, expression_str).unwrap();
        let ast = ValueAssignment::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            ValueAssignment {
                operand: Operand::EqualityNot(EqualityNot {}),
                value: Box::new(CSTExpression::Number(Box::new(Number {
                    value: "2".into()
                }))),
            }
        )
    }

    #[test]
    fn test_value_assignment_equality_bit_not() {
        let expression_str = "~= 2";
        let mut pairs = ElpParser::parse(Rule::value_assignment, expression_str).unwrap();
        let ast = ValueAssignment::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            ValueAssignment {
                operand: Operand::EqualityBitNot(EqualityBitNot {}),
                value: Box::new(CSTExpression::Number(Box::new(Number {
                    value: "2".into()
                }))),
            }
        )
    }

    #[test]
    fn test_value_assignment_bitwise_and() {
        let expression_str = "&= 2";
        let mut pairs = ElpParser::parse(Rule::value_assignment, expression_str).unwrap();
        let ast = ValueAssignment::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            ValueAssignment {
                operand: Operand::BitAnd(BitAnd),
                value: Box::new(CSTExpression::Number(Box::new(Number {
                    value: "2".into()
                }))),
            }
        )
    }
}
