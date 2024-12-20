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
