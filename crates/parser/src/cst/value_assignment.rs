use pest_ast::FromPest;

use crate::parser::Rule;

use super::{expression::CSTExpression, variable_access::CSTReference};

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::BITWISE_OPERAND_TILDE))]
pub struct CSTTilde;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::BITWISE_OPERAND_LEFT_SHIFT))]
pub struct CSTLeftShift;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::BITWISE_OPERAND_RIGHT_SHIFT))]
pub struct CSTRightShift;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::BITWISE_OPERAND_OR))]
pub struct CSTOr;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::BITWISE_OPERAND))]
pub enum CSTBitwiseOperand {
    Tilde(CSTTilde),
    LeftShift(CSTLeftShift),
    RightShift(CSTRightShift),
    BitOr(CSTOr),
    BitAnd(CSTReference),
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::OPERAND_PLUS))]
pub struct CSTPlus;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::OPERAND_MINUS))]
pub struct CSTMinus;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::OPERAND_MUL))]
pub struct CSTMultiply;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::OPERAND_DIV))]
pub struct CSTDivide;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::OPERAND_MOD))]
pub struct CSTModulo;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::OPERAND_POW))]
pub struct CSTPower;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::OPERAND_NOT_EQUAL))]
pub struct CSTEqualityNot;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::OPERAND_BIT_NOT_EQUAL))]
pub struct CSTEqualityBitNot;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::OPERAND_EQUALS))]
pub struct CSTEquals;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::OPERAND_EQUAL))]
pub struct CSTEqualityEqual;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::OPERAND_BITAND))]
pub struct CSTBitAnd;

#[derive(Debug, PartialEq, Eq, FromPest)]
#[pest_ast(rule(Rule::value_assignment))]
pub struct CSTValueAssignment {
    pub operand: CSTOperand,
    pub value: Box<CSTExpression>,
}

#[derive(Debug, PartialEq, FromPest, Eq)]
#[pest_ast(rule(Rule::OPERAND))]
pub enum CSTOperand {
    Plus(CSTPlus),
    Minus(CSTMinus),
    Multiply(CSTMultiply),
    Divide(CSTDivide),
    Modulo(CSTModulo),
    Power(CSTPower),
    EqualityNot(CSTEqualityNot),
    EqualityBitNot(CSTEqualityBitNot),
    Equals(CSTEquals),
    EqualityEqual(CSTEqualityEqual),
    BitAnd(CSTBitAnd),
}

#[cfg(test)]
mod tests {
    use from_pest::FromPest;
    use pest::Parser;

    use crate::{
        cst::{number_value::CSTNumber, string::CSTString},
        parser::ElpParser,
    };

    use super::*;

    #[test]
    fn test_value_assignment_equals() {
        let expression_str = "= \"world\"";
        let mut pairs = ElpParser::parse(Rule::value_assignment, expression_str).unwrap();
        let ast = CSTValueAssignment::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTValueAssignment {
                operand: CSTOperand::Equals(CSTEquals {}),
                value: Box::new(CSTExpression::String(Box::new(CSTString {
                    value: "world".into(),
                }))),
            }
        )
    }

    #[test]
    fn test_value_assignment_plus() {
        let expression_str = "+= 1";
        let mut pairs = ElpParser::parse(Rule::value_assignment, expression_str).unwrap();
        let ast = CSTValueAssignment::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTValueAssignment {
                operand: CSTOperand::Plus(CSTPlus {}),
                value: Box::new(CSTExpression::Number(Box::new(CSTNumber {
                    value: "1".into()
                }))),
            }
        )
    }

    #[test]
    fn test_value_assignment_modulo() {
        let expression_str = "%= 1";
        let mut pairs = ElpParser::parse(Rule::value_assignment, expression_str).unwrap();
        let ast = CSTValueAssignment::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTValueAssignment {
                operand: CSTOperand::Modulo(CSTModulo {}),
                value: Box::new(CSTExpression::Number(Box::new(CSTNumber {
                    value: "1".into()
                }))),
            }
        )
    }

    #[test]
    fn test_value_assignment_equality_equal() {
        let expression_str = "== 2";
        let mut pairs = ElpParser::parse(Rule::value_assignment, expression_str).unwrap();
        let ast = CSTValueAssignment::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTValueAssignment {
                operand: CSTOperand::EqualityEqual(CSTEqualityEqual {}),
                value: Box::new(CSTExpression::Number(Box::new(CSTNumber {
                    value: "2".into()
                }))),
            }
        )
    }

    #[test]
    fn test_value_assignment_equality_not() {
        let expression_str = "!= 2";
        let mut pairs = ElpParser::parse(Rule::value_assignment, expression_str).unwrap();
        let ast = CSTValueAssignment::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTValueAssignment {
                operand: CSTOperand::EqualityNot(CSTEqualityNot {}),
                value: Box::new(CSTExpression::Number(Box::new(CSTNumber {
                    value: "2".into()
                }))),
            }
        )
    }

    #[test]
    fn test_value_assignment_equality_bit_not() {
        let expression_str = "~= 2";
        let mut pairs = ElpParser::parse(Rule::value_assignment, expression_str).unwrap();
        let ast = CSTValueAssignment::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTValueAssignment {
                operand: CSTOperand::EqualityBitNot(CSTEqualityBitNot {}),
                value: Box::new(CSTExpression::Number(Box::new(CSTNumber {
                    value: "2".into()
                }))),
            }
        )
    }

    #[test]
    fn test_value_assignment_bitwise_and() {
        let expression_str = "&= 2";
        let mut pairs = ElpParser::parse(Rule::value_assignment, expression_str).unwrap();
        let ast = CSTValueAssignment::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTValueAssignment {
                operand: CSTOperand::BitAnd(CSTBitAnd),
                value: Box::new(CSTExpression::Number(Box::new(CSTNumber {
                    value: "2".into()
                }))),
            }
        )
    }
}
