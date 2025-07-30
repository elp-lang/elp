pub struct CodeSpan<'s> {
    pub start: usize,
    pub end: usize,
    pub source: &'s str,
}
