use std::{collections::HashMap, fmt::Debug, path::PathBuf};

use lexer::span::CodeSpan;

use crate::ast::{declaration::Decl, expression::Expr};

pub mod ast;

#[derive(Debug, PartialEq)]
pub struct Spanned<T: Debug + PartialEq> {
    pub node: T,
    pub span: CodeSpan,
}

#[derive(Debug, PartialEq)]
pub enum ElpItem {
    Declaration(Decl),
    Expression(Expr),
}

#[derive(Debug, PartialEq)]
pub struct ElpModule {
    pub path: PathBuf,
    pub items: Vec<Spanned<ElpItem>>,
    pub macros: HashMap<String, ElpItem>,
}

#[derive(Debug, PartialEq)]
pub struct ElpParser {
    pub modules: ElpModule,
}
