use super::{
    block::CSTBlock,
    component::CSTComponentDef,
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
    value_assignment::{CSTOperand, CSTValueAssignment},
    variable_access::{CSTPointerSemantics, CSTVariableAccess},
    variable_assignment::CSTVariableAssignment,
    variable_declaration::CSTVariableDeclaration,
};
use crate::parser::Rule;
use pest_ast::FromPest;

#[derive(Debug, FromPest, PartialEq, Eq, Clone)]
#[pest_ast(rule(Rule::expression))]
pub enum CSTExpression<'a> {
    Block(Box<CSTBlock<'a>>),
    Component(Box<CSTComponentDef<'a>>),
    ElpType(Box<CSTElpType<'a>>),
    Enum(Box<CSTEnum<'a>>),
    Export(Box<CSTExport<'a>>),
    FunctionCall(Box<CSTFunctionCall<'a>>),
    FunctionDef(Box<CSTFunctionDef<'a>>),
    FunctionHeaderDef(Box<CSTFunctionHeaderDef<'a>>),
    FunctionReturnValue(Box<CSTFunctionReturnValue<'a>>),
    Ident(Box<CSTIdent<'a>>),
    Import(Box<CSTImport<'a>>),
    Interface(Box<CSTInterface<'a>>),
    Match(Box<CSTMatchTree<'a>>),
    Number(Box<CSTNumber<'a>>),
    Object(Box<CSTObject<'a>>),
    Operand(Box<CSTOperand<'a>>),
    PointerSemantics(Box<CSTPointerSemantics<'a>>),
    String(Box<CSTString<'a>>),
    Unary(Box<CSTUnaryOperator<'a>>),
    ValueAssignment(Box<CSTValueAssignment<'a>>),
    VariableAccess(Box<CSTVariableAccess<'a>>),
    VariableAssignment(Box<CSTVariableAssignment<'a>>),
    VariableDeclaration(Box<CSTVariableDeclaration<'a>>),
}

// Hey! Where are my tests?
// There aren't any here because testing here as well as in the code that produces the values for each member
// would effectively be testing Rust itself and there's no need for that. Each of the value types above are
// thouroughly tested in their respective files.
