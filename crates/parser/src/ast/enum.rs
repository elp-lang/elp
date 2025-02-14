use pest::Span;

use super::elp_type::ASTElpType;

#[derive(Debug, PartialEq, Clone)]
pub struct ASTEnum<'a> {
    pub span: &'a Span<'a>,
    pub name: String,
    pub variants: Vec<ASTEnumVariant<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ASTEnumVariant<'a> {
    pub span: &'a Span<'a>,
    pub name: String,
    pub fields: Vec<ASTField<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ASTField<'a> {
    pub span: &'a Span<'a>,
    pub name: String,
    pub ty: Option<ASTElpType<'a>>,
}
