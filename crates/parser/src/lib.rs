use lexer::span::{CodeSpan, SpannedToken};
use std::{collections::HashMap, fmt::Debug, path::PathBuf};

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
    pub items: Vec<Spanned<ElpItem>>,
    pub macros: HashMap<String, ElpItem>,
}

#[derive(Debug, PartialEq)]
pub struct ElpParser {
    pub scripts: HashMap<String, ElpScript>,
}

impl ElpParser {
    pub fn new(entry_path: String) -> Self {
        let mut scripts: HashMap<String, ElpScript> = HashMap::new();

        Self { scripts }
    }
}

pub trait ItemFromTokens: PartialEq {
    fn from_tokens(tokens: &mut Vec<SpannedToken>) -> anyhow::Result<ElpItem>;
}

#[derive(Debug, PartialEq)]
pub(crate) struct TokenCursor {
    pub position: usize,
    pub length: usize,
}

impl TokenCursor {
    pub fn new(tokens: &Vec<SpannedToken>) -> Self {
        Self {
            position: 0,
            length: tokens.len(),
        }
    }
}
