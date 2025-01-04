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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::parser::ElpParser;
    use from_pest::FromPest;
    use pest::Parser;

    #[test]
    fn elp_type_ast_generation() {
        let expression_str = "String";
        let mut pairs = ElpParser::parse(Rule::elp_type, expression_str).unwrap();
        let ast = ElpType::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            ElpType {
                name: "String".into(),
                type_parameters: vec![]
            }
        )
    }

    #[test]
    fn elp_generic_ast_generation() {
        let expression_str = "<String: Copy>";
        let mut pairs = ElpParser::parse(Rule::elp_generic, expression_str).unwrap();
        let ast = ElpTypeGeneric::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            ElpTypeGeneric {
                elp_type: ElpType {
                    name: "String".into(),
                    type_parameters: vec![]
                },
                type_constraint: Some(ElpTypeGenericConstraint {
                    constraints: vec![ElpType {
                        name: "Copy".into(),
                        type_parameters: vec![]
                    }]
                })
            }
        )
    }

    #[test]
    fn elp_generic_constraint_ast_generation() {
        let expression_str = "<String: Copy + Clone>";
        let mut pairs = ElpParser::parse(Rule::elp_generic, expression_str).unwrap();
        let ast = ElpTypeGeneric::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            ElpTypeGeneric {
                elp_type: ElpType {
                    name: "String".into(),
                    type_parameters: vec![]
                },
                type_constraint: Some(ElpTypeGenericConstraint {
                    constraints: vec![
                        ElpType {
                            name: "Copy".into(),
                            type_parameters: vec![]
                        },
                        ElpType {
                            name: "Clone".into(),
                            type_parameters: vec![]
                        }
                    ]
                })
            }
        )
    }
}
