use pest_ast::FromPest;

use crate::parser::Rule;

use super::{block::Block, expression::CSTExpression};

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::for_loop))]
pub struct ForLoop {
    pub declaration_expression: CSTExpression,
    pub in_expression: CSTExpression,
    pub body: Block,
}

#[cfg(test)]
mod tests {
    use crate::{
        cst::{
            function::{FunctionCall, FunctionCallName},
            ident::Ident,
            variable_access::{VariableAccess, VariableAccessNames},
        },
        parser::ElpParser,
    };

    use super::*;
    use from_pest::FromPest;
    use pest::Parser;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_for_loop() {
        let expression_str = "for thing in thingies { print(thing) }";
        let mut pairs = ElpParser::parse(Rule::for_loop, expression_str).unwrap();
        let ast = ForLoop::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            ForLoop {
                declaration_expression: CSTExpression::VariableAccess(Box::new(VariableAccess {
                    names: VariableAccessNames {
                        names: vec![Ident {
                            value: "thing".into()
                        }],
                    },
                    pointer_semantics: vec![],
                })),
                in_expression: CSTExpression::VariableAccess(Box::new(VariableAccess {
                    names: VariableAccessNames {
                        names: vec![Ident {
                            value: "thingies".into()
                        }],
                    },
                    pointer_semantics: vec![],
                })),
                body: Block {
                    expressions: vec![CSTExpression::FunctionCall(Box::new(FunctionCall {
                        name: FunctionCallName::VariableAccess(VariableAccess {
                            pointer_semantics: vec![],
                            names: VariableAccessNames {
                                names: vec![Ident {
                                    value: "print".into()
                                }],
                            },
                        }),
                        generics: None,
                        arguments: vec![CSTExpression::VariableAccess(Box::new(VariableAccess {
                            names: VariableAccessNames {
                                names: vec![Ident {
                                    value: "thing".into()
                                }],
                            },
                            pointer_semantics: vec![],
                        }))],
                    }))]
                }
            }
        )
    }
}
