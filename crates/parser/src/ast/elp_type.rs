use super::span_into_string;
use crate::parser::Rule;
use pest_ast::FromPest;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::elp_type))]
pub struct ElpType {
    #[pest_ast(inner(with(span_into_string)))]
    pub name: String,

    pub type_parameters: Vec<ElpType>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::elp_generic))]
pub struct ElpTypeGeneric {
    pub elp_type: ElpType,
    pub type_constraint: Option<ElpTypeGenericConstraint>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::elp_generic_constraint))]
pub struct ElpTypeGenericConstraint {
    pub constraints: Vec<ElpType>,
}
