use super::{
    export::Export,
    function::{FunctionDef, FunctionHeaderDef, FunctionReturnValue},
    import::Import,
    interface::Interface,
    number_value::Number,
    object::Object,
    string::StringValue,
    value_assignment::ValueAssignment,
    variable_access::{PointerSemantics, VariableAccess},
    variable_assignment::VariableAssignment,
    variable_declaration::VariableDeclaration,
};
use crate::parser::Rule;
use pest_ast::FromPest;

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::expression))]
pub enum Expression {
    Import(Box<Import>),
    Export(Box<Export>),
    PointerSemantics(Box<PointerSemantics>),
    VariableAccess(Box<VariableAccess>),
    VariableDeclaration(Box<VariableDeclaration>),
    ValueAssignment(Box<ValueAssignment>),
    VariableAssignment(Box<VariableAssignment>),
    FunctionDef(Box<FunctionDef>),
    FunctionHeaderDef(Box<FunctionHeaderDef>),
    FunctionReturnValue(Box<FunctionReturnValue>),
    String(Box<StringValue>),
    Number(Box<Number>),
    Object(Box<Object>),
    Interface(Box<Interface>),
}
