use super::span_into_string;
use crate::parser::Rule;
use pest_ast::FromPest;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::elp_type))]
pub struct ElpType {
    #[pest_ast(inner(with(span_into_string)))]
    pub name: String,

    pub type_parameters: Option<Vec<ElpType>>,
}
