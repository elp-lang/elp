use pest_ast::FromPest;

use crate::parser::Rule;

use super::{block::CSTBlock, expression::CSTExpression};

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::for_loop))]
pub struct CSTForLoop {
    pub declaration_expression: CSTExpression,
    pub in_expression: CSTExpression,
    pub body: CSTBlock,
}

#[cfg(test)]
mod tests {
    use crate::{
        cst::{
            function::{CSTFunctionCall, CSTFunctionCallName},
            ident::CSTIdent,
            variable_access::{CSTVariableAccess, CSTVariableAccessNames},
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
        let ast = CSTForLoop::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            CSTForLoop {
                declaration_expression: CSTExpression::VariableAccess(Box::new(
                    CSTVariableAccess {
                        names: CSTVariableAccessNames {
                            names: vec![CSTIdent {
                                value: "thing".into()
                            }],
                        },
                        pointer_semantics: vec![],
                    }
                )),
                in_expression: CSTExpression::VariableAccess(Box::new(CSTVariableAccess {
                    names: CSTVariableAccessNames {
                        names: vec![CSTIdent {
                            value: "thingies".into()
                        }],
                    },
                    pointer_semantics: vec![],
                })),
                body: CSTBlock {
                    expressions: vec![CSTExpression::FunctionCall(Box::new(CSTFunctionCall {
                        name: CSTFunctionCallName::VariableAccess(CSTVariableAccess {
                            pointer_semantics: vec![],
                            names: CSTVariableAccessNames {
                                names: vec![CSTIdent {
                                    value: "print".into()
                                }],
                            },
                        }),
                        generics: None,
                        arguments: vec![CSTExpression::VariableAccess(Box::new(
                            CSTVariableAccess {
                                names: CSTVariableAccessNames {
                                    names: vec![CSTIdent {
                                        value: "thing".into()
                                    }],
                                },
                                pointer_semantics: vec![],
                            }
                        ))],
                    }))]
                }
            }
        )
    }
}
