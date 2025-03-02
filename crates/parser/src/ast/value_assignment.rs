use pest::Span;

use crate::cst::value_assignment::{CSTBitwiseOperand, CSTOperand, CSTValueAssignment};

use super::{expression::ASTExpression, traits::FromCST};

#[derive(Debug, PartialEq, Clone)]
pub struct ASTValueAssignment<'a> {
    pub span: &'a Span<'a>,
    pub operand: ASTOperand,
    pub value: Box<ASTExpression<'a>>,
}

impl<'a> FromCST<'a, CSTValueAssignment<'a>> for ASTValueAssignment<'a> {
    fn from_cst(cst: &'a CSTValueAssignment<'a>) -> Self {
        Self {
            span: &cst.span,
            operand: ASTOperand::from_cst(&cst.operand),
            value: Box::new(ASTExpression::from_cst(&cst.value)),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ASTOperand {
    BitAnd,
    BitOr,
    Divide,
    EqualityBitNot,
    EqualityEqual,
    EqualityNot,
    Equals,
    LeftShift,
    Minus,
    Modulo,
    Multiply,
    Plus,
    Power,
    RightShift,
    Tilde,
}

impl<'a> FromCST<'a, CSTOperand<'a>> for ASTOperand {
    fn from_cst(cst: &'a CSTOperand) -> Self {
        match cst {
            CSTOperand::BitAnd(..) => ASTOperand::BitAnd,
            CSTOperand::Divide(..) => ASTOperand::Divide,
            CSTOperand::EqualityBitNot(..) => ASTOperand::EqualityBitNot,
            CSTOperand::EqualityEqual(..) => ASTOperand::EqualityEqual,
            CSTOperand::EqualityNot(..) => ASTOperand::EqualityNot,
            CSTOperand::Equals(..) => ASTOperand::Equals,
            CSTOperand::Minus(..) => ASTOperand::Minus,
            CSTOperand::Modulo(..) => ASTOperand::Modulo,
            CSTOperand::Multiply(..) => ASTOperand::Multiply,
            CSTOperand::Plus(..) => ASTOperand::Plus,
            CSTOperand::Power(..) => ASTOperand::Power,
        }
    }
}

impl<'a> FromCST<'a, CSTBitwiseOperand<'a>> for ASTOperand {
    fn from_cst(cst: &'a CSTBitwiseOperand<'a>) -> Self {
        match cst {
            CSTBitwiseOperand::LeftShift(..) => ASTOperand::LeftShift,
            CSTBitwiseOperand::RightShift(..) => ASTOperand::RightShift,
            CSTBitwiseOperand::Tilde(..) => ASTOperand::Tilde,
            CSTBitwiseOperand::BitAnd(..) => ASTOperand::BitAnd,
            CSTBitwiseOperand::BitOr(..) => ASTOperand::BitOr,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::cst::value_assignment::{
        CSTBitAnd, CSTDivide, CSTEqualityBitNot, CSTEqualityEqual, CSTEqualityNot, CSTEquals,
        CSTMinus, CSTModulo, CSTMultiply, CSTPlus, CSTPower,
    };

    use super::*;

    #[test]
    fn ast_operand_from_cst() {
        let bitand_cst_operand = CSTOperand::BitAnd(CSTBitAnd {
            span: pest::Span::new("&=", 0, 2).unwrap(),
        });
        let bitand_ast_operand = ASTOperand::from_cst(&bitand_cst_operand);
        assert_eq!(bitand_ast_operand, ASTOperand::BitAnd);

        let divide_cst_operand = CSTOperand::Divide(CSTDivide {
            span: pest::Span::new("/", 0, 1).unwrap(),
        });
        let divide_ast_operand = ASTOperand::from_cst(&divide_cst_operand);
        assert_eq!(divide_ast_operand, ASTOperand::Divide);

        let equality_bit_not_cst_operand = CSTOperand::EqualityBitNot(CSTEqualityBitNot {
            span: pest::Span::new("!=", 0, 2).unwrap(),
        });
        let equality_bit_not_ast_operand = ASTOperand::from_cst(&equality_bit_not_cst_operand);
        assert_eq!(equality_bit_not_ast_operand, ASTOperand::EqualityBitNot);

        let equality_equal_cst_operand = CSTOperand::EqualityEqual(CSTEqualityEqual {
            span: pest::Span::new("==", 0, 2).unwrap(),
        });
        let equality_equal_ast_operand = ASTOperand::from_cst(&equality_equal_cst_operand);
        assert_eq!(equality_equal_ast_operand, ASTOperand::EqualityEqual);

        let equality_not_cst_operand = CSTOperand::EqualityNot(CSTEqualityNot {
            span: pest::Span::new("!=", 0, 2).unwrap(),
        });
        let equality_not_ast_operand = ASTOperand::from_cst(&equality_not_cst_operand);
        assert_eq!(equality_not_ast_operand, ASTOperand::EqualityNot);

        let equals_cst_operand = CSTOperand::Equals(CSTEquals {
            span: pest::Span::new("=", 0, 1).unwrap(),
        });
        let equals_ast_operand = ASTOperand::from_cst(&equals_cst_operand);
        assert_eq!(equals_ast_operand, ASTOperand::Equals);

        let minus_cst_operand = CSTOperand::Minus(CSTMinus {
            span: pest::Span::new("-", 0, 1).unwrap(),
        });
        let minus_ast_operand = ASTOperand::from_cst(&minus_cst_operand);
        assert_eq!(minus_ast_operand, ASTOperand::Minus);

        let modulo_cst_operand = CSTOperand::Modulo(CSTModulo {
            span: pest::Span::new("%", 0, 1).unwrap(),
        });
        let modulo_ast_operand = ASTOperand::from_cst(&modulo_cst_operand);
        assert_eq!(modulo_ast_operand, ASTOperand::Modulo);

        let multiply_cst_operand = CSTOperand::Multiply(CSTMultiply {
            span: pest::Span::new("*", 0, 1).unwrap(),
        });
        let multiply_ast_operand = ASTOperand::from_cst(&multiply_cst_operand);
        assert_eq!(multiply_ast_operand, ASTOperand::Multiply);

        let plus_cst_operand = CSTOperand::Plus(CSTPlus {
            span: pest::Span::new("+", 0, 1).unwrap(),
        });
        let plus_ast_operand = ASTOperand::from_cst(&plus_cst_operand);
        assert_eq!(plus_ast_operand, ASTOperand::Plus);

        let power_cst_operand = CSTOperand::Power(CSTPower {
            span: pest::Span::new("**", 0, 2).unwrap(),
        });
        let power_ast_operand = ASTOperand::from_cst(&power_cst_operand);
        assert_eq!(power_ast_operand, ASTOperand::Power);
    }
}
