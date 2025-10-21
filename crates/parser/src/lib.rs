use std::{collections::HashMap, fmt::Debug, path::PathBuf};

use anyhow::Result;
use lexer::span::{CodeSpan, SpannedToken};

use crate::ast::{declaration::Decl, expression::Expr};

pub mod ast;

#[derive(Debug, PartialEq)]
pub struct Spanned<T: Debug + PartialEq> {
    pub node: T,
    pub span: Vec<CodeSpan>,
}

#[derive(Debug, PartialEq)]
pub enum ElpItem {
    Declaration(Decl),
    Expression(Expr),
}

#[derive(Debug, PartialEq)]
pub struct ElpScript {
    pub path: PathBuf,
    pub items: Vec<Spanned<ElpItem>>,
    pub macros: HashMap<String, ElpItem>,
}

#[derive(Debug, PartialEq)]
pub struct ElpParser {
    pub scripts: ElpScript,
}

pub trait ItemFromTokens: PartialEq {
    fn from_tokens(tokens: &mut Vec<SpannedToken>) -> Result<ElpItem>;
}
