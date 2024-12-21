use crate::parser::Rule;
use from_pest::{ConversionError, FromPest, Void};
use pest::iterators::Pairs;
use pest_ast::FromPest;

#[derive(Debug, PartialEq, Eq)]
pub enum Operand {
    Plus,
    Minus,
    Multiply,
    Divide,
    BitOr,
    BitAnd,
    BitXor,
    Exponential,
}

impl<'pest> FromPest<'pest> for Operand {
    type Rule = crate::Rule;
    type FatalError = Void;

    fn from_pest(
        pest: &mut Pairs<'pest, Self::Rule>,
    ) -> Result<Self, ConversionError<Self::FatalError>> {
        let symbol = pest.next().unwrap();

        match symbol.as_str() {
            "+" => Ok(Operand::Plus),
            "-" => Ok(Operand::Minus),
            "*" => Ok(Operand::Multiply),
            "/" => Ok(Operand::Divide),
            "|" => Ok(Operand::BitOr),
            "&" => Ok(Operand::BitAnd),
            "^" => Ok(Operand::BitXor),
            "**" => Ok(Operand::Exponential),
            _ => Err(ConversionError::NoMatch),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Equality {
    Equal,
    NotEqual,
}
