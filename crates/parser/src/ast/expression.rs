use super::{
    export::Export,
    function::{FunctionDef, FunctionReturnValue},
    import::Import,
    variable_assignment::VariableAssignment,
    variable_declaration::VariableDeclaration,
    StringValue,
};
use crate::parser::Rule;
use pest_ast::FromPest;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::expression))]
pub enum Expression {
    #[pest_ast(rule(Rule::import))]
    Import(Box<Import>),

    #[pest_ast(rule(Rule::export))]
    Export(Box<Export>),

    #[pest_ast(rule(Rule::variable_declaration))]
    VariableDeclaration(Box<VariableDeclaration>),

    #[pest_ast(rule(Rule::variable_assignment))]
    VariableAssignment(Box<VariableAssignment>),

    #[pest_ast(rule(Rule::function_def))]
    FunctionDef(Box<FunctionDef>),

    #[pest_ast(rule(Rule::function_return_value))]
    FunctionReturnValue(Box<FunctionReturnValue>),

    #[pest_ast(rule(Rule::string))]
    String(Box<StringValue>),
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::ast::variable_declaration::VariableMutability;
    use crate::{
        ast::{
            import::{ImportModulePath, ImportName, ImportNameAlias},
            StringValue,
        },
        parser::ElpParser,
    };
    use from_pest::FromPest;
    use pest::Parser;
    use pretty_assertions::assert_eq;

    #[test]
    fn parse_import_expression() {
        let expression_str = "import {Bar, Baz as BazAlias} from \"foo\"";
        let mut pairs = ElpParser::parse(Rule::expression, expression_str).unwrap();
        let ast = Expression::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            Expression::Import(Box::new(Import {
                names: vec![
                    ImportName {
                        name: "Bar".into(),
                        alias: None,
                    },
                    ImportName {
                        name: "Baz".to_string(),
                        alias: Some(ImportNameAlias {
                            alias: "BazAlias".into()
                        }),
                    }
                ],
                module_path: ImportModulePath {
                    module_path: StringValue {
                        value: "foo".into()
                    }
                }
            })),
        )
    }

    #[test]
    fn parse_export_expression() {
        let expression_str = "export const hello";
        let mut pairs = ElpParser::parse(Rule::expression, expression_str).unwrap();
        let ast = Expression::from_pest(&mut pairs).unwrap();

        assert_eq!(
            ast,
            Expression::Export(Box::new(Export {
                expression: Expression::VariableDeclaration(Box::new(VariableDeclaration {
                    mutability: VariableMutability::Immutable,
                    name: "hello".into(),
                    type_annotation: None
                }))
            }))
        )
    }
}
