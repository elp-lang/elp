use pest_ast::FromPest;

use crate::parser::Rule;

use super::{elp_type::ElpType, span_into_string};

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::enum_member))]
pub struct EnumMember {
    #[pest_ast(inner(with(span_into_string)))]
    pub name: String,
    pub params: Vec<ElpType>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::r#enum))]
pub struct Enum {
    #[pest_ast(inner(with(span_into_string)))]
    pub name: String,
    pub members: Vec<EnumMember>,
}

#[cfg(test)]
mod tests {
    use crate::parser::ElpParser;

    use super::*;
    use from_pest::FromPest;
    use pest::Parser;
    use pretty_assertions::assert_eq;

    #[test]
    fn enum_member() {
        let expression_str = ".MEMBER";
        let mut pairs = ElpParser::parse(Rule::enum_member, expression_str).unwrap();
        let ast = EnumMember::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            EnumMember {
                name: "MEMBER".into(),
                params: vec![]
            }
        )
    }

    #[test]
    fn enum_member_with_params() {
        let expression_str = ".Member(T)";
        let mut pairs = ElpParser::parse(Rule::enum_member, expression_str).unwrap();
        let ast = EnumMember::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            EnumMember {
                name: "Member".into(),
                params: vec![ElpType {
                    name: "T".into(),
                    type_parameters: vec![],
                }]
            }
        )
    }

    #[test]
    fn enum_def() {
        let expression_str = "enum MyEnum { .MEMBER }";
        let mut pairs = ElpParser::parse(Rule::r#enum, expression_str).unwrap();
        let ast = Enum::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            Enum {
                name: "MyEnum".into(),
                members: vec![EnumMember {
                    name: "MEMBER".into(),
                    params: vec![]
                }]
            }
        )
    }
}
