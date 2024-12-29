use pest_ast::FromPest;

use crate::parser::Rule;

use super::{expression::Expression, variable_access::Reference};

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
#[pest_ast(rule(Rule::OPERAND_EXPO))]
pub struct Exponential;

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

#[derive(Debug, PartialEq, Eq, FromPest)]
#[pest_ast(rule(Rule::value_assignment))]
pub struct ValueAssignment {
    pub operand: Operand,
    pub value: Box<Expression>,
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
}

#[cfg(test)]
mod tests {
    use from_pest::FromPest;
    use pest::Parser;

    use crate::{ast::string::StringValue, parser::ElpParser};

    use super::*;

    #[test]
    fn test_value_assignment() {
        let expression_str = "= \"world\"";
        let mut pairs = ElpParser::parse(Rule::value_assignment, expression_str).unwrap();
        let ast = ValueAssignment::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            ValueAssignment {
                operand: Operand::Equals(Equals {}),
                value: Box::new(Expression::String(Box::new(StringValue {
                    value: "world".into(),
                }))),
            }
        )
    }
}
