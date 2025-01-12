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
