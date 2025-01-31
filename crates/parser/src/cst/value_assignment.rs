use pest::Span;
use pest_ast::FromPest;

use crate::parser::Rule;

use super::{expression::CSTExpression, variable_access::CSTReference};

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::BITWISE_OPERAND_TILDE))]
pub struct CSTTilde<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::BITWISE_OPERAND_LEFT_SHIFT))]
pub struct CSTLeftShift<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::BITWISE_OPERAND_RIGHT_SHIFT))]
pub struct CSTRightShift<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::BITWISE_OPERAND_OR))]
pub struct CSTOr<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::BITWISE_OPERAND))]
pub enum CSTBitwiseOperand<'a> {
    Tilde(CSTTilde<'a>),
    LeftShift(CSTLeftShift<'a>),
    RightShift(CSTRightShift<'a>),
    BitOr(CSTOr<'a>),
    BitAnd(CSTReference<'a>),
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::OPERAND_PLUS))]
pub struct CSTPlus<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::OPERAND_MINUS))]
pub struct CSTMinus<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::OPERAND_MUL))]
pub struct CSTMultiply<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::OPERAND_DIV))]
pub struct CSTDivide<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::OPERAND_MOD))]
pub struct CSTModulo<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::OPERAND_POW))]
pub struct CSTPower<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::OPERAND_NOT_EQUAL))]
pub struct CSTEqualityNot<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::OPERAND_BIT_NOT_EQUAL))]
pub struct CSTEqualityBitNot<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::OPERAND_EQUALS))]
pub struct CSTEquals<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::OPERAND_EQUAL))]
pub struct CSTEqualityEqual<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::OPERAND_BITAND))]
pub struct CSTBitAnd<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
}

#[derive(Debug, PartialEq, Eq, FromPest)]
#[pest_ast(rule(Rule::value_assignment))]
pub struct CSTValueAssignment<'a> {
    #[pest_ast(outer())]
    pub span: Span<'a>,
    pub operand: CSTOperand<'a>,
    pub value: Box<CSTExpression<'a>>,
}

#[derive(Debug, PartialEq, FromPest, Eq)]
#[pest_ast(rule(Rule::OPERAND))]
pub enum CSTOperand<'a> {
    Plus(CSTPlus<'a>),
    Minus(CSTMinus<'a>),
    Multiply(CSTMultiply<'a>),
    Divide(CSTDivide<'a>),
    Modulo(CSTModulo<'a>),
    Power(CSTPower<'a>),
    EqualityNot(CSTEqualityNot<'a>),
    EqualityBitNot(CSTEqualityBitNot<'a>),
    Equals(CSTEquals<'a>),
    EqualityEqual(CSTEqualityEqual<'a>),
    BitAnd(CSTBitAnd<'a>),
}

#[cfg(test)]
mod tests {
    use from_pest::FromPest;
    use pest::Parser;

    use crate::{
        cst::{number_value::CSTNumber, string::CSTString},
        parser::ElpParser,
    };
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_value_assignment_equals() {
        let expression_str = "= \"world\"";
        let mut pairs = ElpParser::parse(Rule::value_assignment, expression_str).unwrap();
        let ast = CSTValueAssignment::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTValueAssignment {
                span: pest::Span::new(expression_str, 0, 9).unwrap(),
                operand: CSTOperand::Equals(CSTEquals {
                    span: pest::Span::new(expression_str, 0, 1).unwrap(),
                }),
                value: Box::new(CSTExpression::String(Box::new(CSTString {
                    span: pest::Span::new(expression_str, 2, 9).unwrap(),
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
                span: pest::Span::new(expression_str, 0, 4).unwrap(),
                operand: CSTOperand::Plus(CSTPlus {
                    span: pest::Span::new(expression_str, 0, 2).unwrap(),
                }),
                value: Box::new(CSTExpression::Number(Box::new(CSTNumber {
                    span: pest::Span::new(expression_str, 3, 4).unwrap(),
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
                span: pest::Span::new(expression_str, 0, 4).unwrap(),
                operand: CSTOperand::Modulo(CSTModulo {
                    span: pest::Span::new(expression_str, 0, 2).unwrap(),
                }),
                value: Box::new(CSTExpression::Number(Box::new(CSTNumber {
                    span: pest::Span::new(expression_str, 3, 4).unwrap(),
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
                span: pest::Span::new(expression_str, 0, 4).unwrap(),
                operand: CSTOperand::EqualityEqual(CSTEqualityEqual {
                    span: pest::Span::new(expression_str, 0, 2).unwrap(),
                }),
                value: Box::new(CSTExpression::Number(Box::new(CSTNumber {
                    span: pest::Span::new(expression_str, 3, 4).unwrap(),
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
                span: pest::Span::new(expression_str, 0, 4).unwrap(),
                operand: CSTOperand::EqualityNot(CSTEqualityNot {
                    span: pest::Span::new(expression_str, 0, 2).unwrap(),
                }),
                value: Box::new(CSTExpression::Number(Box::new(CSTNumber {
                    span: pest::Span::new(expression_str, 3, 4).unwrap(),
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
                span: pest::Span::new(expression_str, 0, 4).unwrap(),
                operand: CSTOperand::EqualityBitNot(CSTEqualityBitNot {
                    span: pest::Span::new(expression_str, 0, 2).unwrap(),
                }),
                value: Box::new(CSTExpression::Number(Box::new(CSTNumber {
                    span: pest::Span::new(expression_str, 3, 4).unwrap(),
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
                span: pest::Span::new(expression_str, 0, 4).unwrap(),
                operand: CSTOperand::BitAnd(CSTBitAnd {
                    span: pest::Span::new(expression_str, 0, 2).unwrap(),
                }),
                value: Box::new(CSTExpression::Number(Box::new(CSTNumber {
                    span: pest::Span::new(expression_str, 3, 4).unwrap(),
                    value: "2".into()
                }))),
            }
        )
    }
}
