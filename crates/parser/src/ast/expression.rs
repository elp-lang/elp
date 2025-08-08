use lexer::span::CodeSpan;

#[derive(Debug, PartialEq)]
pub enum Expr {
    Identifier { name: String, span: CodeSpan },
    Int64 { value: i64, span: CodeSpan },
}
