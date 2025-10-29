use lexer::span::{CodeSpan, SpannedToken};
use std::{collections::HashMap, fmt::Debug, path::PathBuf};

use crate::ast::{declaration::Decl, expression::Expr};

pub mod ast;
pub mod parser;

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

pub enum ElpScriptType {
    Script,
    ViewScript,
    Stylesheet,
}

impl ElpScriptType {
    pub fn file_ext(&self) -> &str {
        match self {
            Self::Stylesheet => "selp",
            Self::ViewScript => "velp",
            Self::Script => "elp",
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ElpScript {
    pub items: Vec<Spanned<ElpItem>>,
    pub macros: HashMap<String, ElpItem>,
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
