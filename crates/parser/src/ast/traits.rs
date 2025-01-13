pub trait FromCST<T> {
    fn from_cst(cst: &T) -> Self;
}
