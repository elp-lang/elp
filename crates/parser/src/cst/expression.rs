use super::{
    block::Block,
    elp_type::ElpType,
    export::Export,
    function::{FunctionDef, FunctionHeaderDef, FunctionReturnValue},
    ident::Ident,
    import::Import,
    interface::Interface,
    number_value::Number,
    object::Object,
    r#enum::Enum,
    r#match::MatchTree,
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
pub enum CSTExpression {
    Block(Box<Block>),
    ElpType(Box<ElpType>),
    Enum(Box<Enum>),
    Export(Box<Export>),
    FunctionDef(Box<FunctionDef>),
    FunctionHeaderDef(Box<FunctionHeaderDef>),
    FunctionReturnValue(Box<FunctionReturnValue>),
    Ident(Box<Ident>),
    Import(Box<Import>),
    Interface(Box<Interface>),
    Match(Box<MatchTree>),
    Number(Box<Number>),
    Object(Box<Object>),
    PointerSemantics(Box<PointerSemantics>),
    String(Box<StringValue>),
    ValueAssignment(Box<ValueAssignment>),
    VariableAccess(Box<VariableAccess>),
    VariableAssignment(Box<VariableAssignment>),
    VariableDeclaration(Box<VariableDeclaration>),
}
