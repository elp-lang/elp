use crate::cst::expression::CSTExpression;

pub trait FromCST {
    fn from_cst(cst: &CSTExpression) -> Self;
}
