use super::{
    block::CSTBlock,
    elp_type::CSTElpType,
    export::CSTExport,
    function::{CSTFunctionCall, CSTFunctionDef, CSTFunctionHeaderDef, CSTFunctionReturnValue},
    ident::CSTIdent,
    import::CSTImport,
    interface::CSTInterface,
    number_value::CSTNumber,
    object::CSTObject,
    r#enum::CSTEnum,
    r#match::CSTMatchTree,
    string::CSTString,
    unary::CSTUnaryOperator,
    value_assignment::CSTValueAssignment,
    variable_access::{CSTPointerSemantics, CSTVariableAccess},
    variable_assignment::CSTVariableAssignment,
    variable_declaration::CSTVariableDeclaration,
};
use crate::parser::Rule;
use from_pest::FromPest;
use pest_ast::FromPest;

pub struct Spanned<'pest, T: FromPest<'pest>> {
    pub value: T,
    pub span: pest::Span<'pest>,
}

#[derive(Debug, FromPest, PartialEq, Eq)]
#[pest_ast(rule(Rule::expression))]
pub enum CSTExpression {
    Block(Box<CSTBlock>),
    ElpType(Box<CSTElpType>),
    Enum(Box<CSTEnum>),
    Export(Box<CSTExport>),
    FunctionDef(Box<CSTFunctionDef>),
    FunctionCall(Box<CSTFunctionCall>),
    FunctionHeaderDef(Box<CSTFunctionHeaderDef>),
    FunctionReturnValue(Box<CSTFunctionReturnValue>),
    Ident(Box<CSTIdent>),
    Import(Box<CSTImport>),
    Interface(Box<CSTInterface>),
    Match(Box<CSTMatchTree>),
    Number(Box<CSTNumber>),
    Object(Box<CSTObject>),
    PointerSemantics(Box<CSTPointerSemantics>),
    String(Box<CSTString>),
    ValueAssignment(Box<CSTValueAssignment>),
    VariableAccess(Box<CSTVariableAccess>),
    VariableAssignment(Box<CSTVariableAssignment>),
    VariableDeclaration(Box<CSTVariableDeclaration>),
    Unary(Box<CSTUnaryOperator>),
}
