pub trait FromCST<'a, T> {
    fn from_cst(cst: &'a T) -> Self;
}
