use crate::parser::Rule;
use pest_ast::FromPest;

use super::{block::Block, expression::Expression};

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::match_tree))]
pub struct MatchTree {
    pub match_expression: Expression,
    pub match_arms: Vec<MatchTreeArm>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::match_arm_subject))]
pub enum MatchArmSubject {
    Expression(Expression),
    MatchRange(MatchRange),
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::match_range))]
pub struct MatchRange {
    pub range_start: Option<Box<Expression>>,
    pub range_end: Option<Box<Expression>>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::match_arm_body))]
pub enum MatchBody {
    Expression(Box<Expression>),
    Block(Box<Block>),
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::match_arm))]
pub struct MatchTreeArm {
    pub subject: MatchArmSubject,
    pub body: MatchBody,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cst::{
            ident::Ident,
            string::StringValue,
            variable_access::{VariableAccess, VariableAccessNames},
        },
        parser::ElpParser,
    };
    use from_pest::FromPest;
    use pest::Parser;
    use pretty_assertions::assert_eq;

    #[test]
    fn match_tree_arm() {
        let expression_str = "expr -> {
\"Hello, World!\"
}";
        let mut pairs = ElpParser::parse(Rule::match_arm, expression_str).unwrap();
        let ast = MatchTreeArm::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            MatchTreeArm {
                subject: MatchArmSubject::Expression(Expression::VariableAccess(Box::new(
                    VariableAccess {
                        pointer_semantics: vec![],
                        names: VariableAccessNames {
                            names: vec![Ident {
                                value: "expr".into()
                            }],
                        },
                    }
                ))),
                body: MatchBody::Block(Box::new(Block {
                    expressions: vec![Expression::String(Box::new(StringValue {
                        value: "Hello, World!".into()
                    }))]
                }))
            }
        )
    }
}
