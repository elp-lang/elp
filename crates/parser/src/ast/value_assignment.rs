use crate::parser::Rule;
use pest_ast::FromPest;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::OPERAND))]
pub enum Operand {
    Plus,
    Minus,
    Multiply,
    Divide,
    BitOr,
    BitAnd,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::EQUALITY))]
pub enum Equality {
    Equal,
    NotEqual,
}
