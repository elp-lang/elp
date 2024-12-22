use crate::ast::ident::Ident;
use crate::parser::Rule;
use from_pest::{ConversionError, FromPest, Void};
use pest::iterators::Pairs;
use pest_ast::FromPest;

#[derive(Debug, PartialEq, Eq)]
pub enum PointerSemantics {
    Pointer,
    Reference,
}

impl<'pest> FromPest<'pest> for PointerSemantics {
    type Rule = crate::Rule;
    type FatalError = Void;

    fn from_pest(
        pest: &mut Pairs<'pest, Self::Rule>,
    ) -> Result<Self, ConversionError<Self::FatalError>> {
        let symbol = pest.next().unwrap();

        match symbol.as_str() {
            "*" => Ok(PointerSemantics::Pointer),
            "&" => Ok(PointerSemantics::Reference),
            _ => Err(ConversionError::NoMatch),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct VariableAccess {
    pub access_semantics: PointerSemantics,
    pub names: Vec<Ident>,
}

impl<'pest> FromPest<'pest> for VariableAccess {
    type Rule = crate::Rule;

    type FatalError = Void;

    fn from_pest(
        pest: &mut Pairs<'pest, Self::Rule>,
    ) -> Result<Self, ConversionError<Self::FatalError>> {
        // Get any access semantics.
        let access_semantics =
            PointerSemantics::from_pest(pest).unwrap_or(PointerSemantics::Reference);

        // Then loop over the remaining tokens to get all the names.
        let mut names: Vec<Ident> = vec![];

        for name in pest.by_ref() {
            names.push(Ident::from_pest(&mut name.into_inner())?);
        }

        Ok(VariableAccess {
            access_semantics,
            names,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::variable_access::VariableAccess;
    use crate::parser::ElpParser;
    use from_pest::FromPest;
    use pest::Parser;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_pointer_semantics() {
        let ref_expression_str = "&";
        let mut ref_pairs = ElpParser::parse(Rule::pointer_semantics, ref_expression_str).unwrap();
        let ref_ast = PointerSemantics::from_pest(&mut ref_pairs).unwrap();

        assert_eq!(ref_ast, PointerSemantics::Reference);

        let ptr_expression_str = "*";
        let mut ptr_pairs = ElpParser::parse(Rule::pointer_semantics, ptr_expression_str).unwrap();
        let ptr_ast = PointerSemantics::from_pest(&mut ptr_pairs).unwrap();

        assert_eq!(ptr_ast, PointerSemantics::Pointer);
    }

    #[test]
    fn variable_access() {
        let expression_str = "hello.world.my.name.is.dave";
        let mut pairs = ElpParser::parse(Rule::variable_access, expression_str).unwrap();
        let ast = VariableAccess::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            VariableAccess {
                access_semantics: PointerSemantics::Reference,
                names: vec![
                    Ident {
                        value: "hello".into()
                    },
                    Ident {
                        value: "world".into()
                    },
                    Ident { value: "my".into() },
                    Ident {
                        value: "name".into()
                    },
                    Ident { value: "is".into() },
                    Ident {
                        value: "dave".into()
                    },
                ],
            }
        )
    }

    #[test]
    fn variable_access_with_pointer_semantics() {
        let expression_str_reference = "&hello.world.my.name.is.dave";
        let mut pairs_reference =
            ElpParser::parse(Rule::variable_access, expression_str_reference).unwrap();
        let reference_ast = VariableAccess::from_pest(&mut pairs_reference).unwrap();

        assert_eq!(
            reference_ast,
            VariableAccess {
                access_semantics: PointerSemantics::Reference,
                names: vec![
                    Ident {
                        value: "hello".into()
                    },
                    Ident {
                        value: "world".into()
                    },
                    Ident { value: "my".into() },
                    Ident {
                        value: "name".into()
                    },
                    Ident { value: "is".into() },
                    Ident {
                        value: "dave".into()
                    },
                ],
            }
        )
    }
}
