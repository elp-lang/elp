// An AST expression is different to an CST expression in the way that a CST expression is a wider set of possible expressions that compute further down to a known AST node. There are a lot of similarities between the two but it must be considered that the CST is an incredibly brief state of the code pipeline and the AST is the first "visible" part as once we have made a pass here we will execute all precomps to refine the AST from userland to satisfy some form of Homoiconicity.

use crate::cst::expression::CSTExpression;

use super::{block::Block, traits::FromCST};

pub enum ASTExpression {
    Block(Box<Block>),
    //    ElpType(Box<ElpType>),
    //    Enum(Box<Enum>),
    //    Export(Box<Export>),
    //    FunctionDef(Box<FunctionDef>),
    //    FunctionHeaderDef(Box<FunctionHeaderDef>),
    //    FunctionReturnValue(Box<FunctionReturnValue>),
    //    Ident(Box<Ident>),
    //    Import(Box<Import>),
    //    Interface(Box<Interface>),
    //    Match(Box<MatchTree>),
    //    Number(Box<Number>),
    //    Object(Box<Object>),
    //    PointerSemantics(Box<PointerSemantics>),
    //    String(Box<StringValue>),
    //    ValueAssignment(Box<ValueAssignment>),
    //    VariableAccess(Box<VariableAccess>),
    //    VariableAssignment(Box<VariableAssignment>),
    //    VariableDeclaration(Box<VariableDeclaration>),
}

impl FromCST for ASTExpression {
    fn from_cst(cst: &CSTExpression) -> Self {
        match cst {
            CSTExpression::Block(block) => ASTExpression::Block(Box::new(Block::from_cst(block))),
            CSTExpression::ElpType(elp_type) => todo!(),
            CSTExpression::Enum(_) => todo!(),
            CSTExpression::Export(export) => todo!(),
            CSTExpression::FunctionDef(function_def) => todo!(),
            CSTExpression::FunctionHeaderDef(function_header_def) => todo!(),
            CSTExpression::FunctionReturnValue(function_return_value) => todo!(),
            CSTExpression::Ident(ident) => todo!(),
            CSTExpression::Import(import) => todo!(),
            CSTExpression::Interface(interface) => todo!(),
            CSTExpression::Match(match_tree) => todo!(),
            CSTExpression::Number(number) => todo!(),
            CSTExpression::Object(object) => todo!(),
            CSTExpression::PointerSemantics(pointer_semantics) => todo!(),
            CSTExpression::String(string_value) => todo!(),
            CSTExpression::ValueAssignment(value_assignment) => todo!(),
            CSTExpression::VariableAccess(variable_access) => todo!(),
            CSTExpression::VariableAssignment(variable_assignment) => todo!(),
            CSTExpression::VariableDeclaration(variable_declaration) => todo!(),
        }
    }
}
