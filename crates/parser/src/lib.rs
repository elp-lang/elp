use lexer::{
    span::{CodeSpan, SpannedToken},
    tokens::LexerTokens,
};
use std::{collections::HashMap, fmt::Debug};

use crate::{
    ast::{declaration::Decl, expression::Expr},
    parser::ElpParserConfig,
};

pub mod ast;
pub mod error;
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

#[derive(Debug, PartialEq, Default)]
pub struct ElpScript {
    pub items: Vec<Spanned<ElpItem>>,
    pub macros: HashMap<String, ElpItem>,
}

impl ElpScript {
    fn from_tokens(
        parser_config: &ElpParserConfig,
        tokens: &mut Vec<SpannedToken>,
    ) -> Result<Self> {
        let mut cursor = TokenCursor::new(&mut tokens);
        let mut script = Self::default();

        // I spent too long being stupid trying to work out why i didn't get the first token,
        // it was because i was consuming instead of peeking so we just peek at the next token.
        while let Some(token) = cursor.peek() {
            let item = match token.token {
                LexerTokens::Identifier(ident) => {
                    // first check for declaration keywords.
                    // TODO I wonder if Stylesheet counts as a declaration, it's basically a
                    // language level macro.
                    match ident.as_str() {
                        "fn" | "var" | "const" | "import" | "object" | "interface" | "export" => {
                            ElpItem::Declaration(Decl::from(&mut cursor))
                        }
                    }
                }
                LexerTokens::Number(_)
                | LexerTokens::String(_)
                | LexerTokens::Boolean(_)
                | LexerTokens::Symbol(_) => ElpItem::Expression(Expr::from(&mut cursor)),
                LexerTokens::Error(_) => todo!(),
                LexerTokens::Illegal(_) => todo!(),
                LexerTokens::SOI | LexerTokens::EOI => cursor.next(),
            };
        }

        Ok(script)
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct TokenCursor<'t> {
    pub tokens: &'t Vec<SpannedToken>,
    pub position: usize,
}

impl<'t> TokenCursor<'t> {
    pub fn new(tokens: &'t Vec<SpannedToken>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    pub fn next(&mut self) -> Option<&SpannedToken> {
        self.position += 1;

        self.tokens.get(self.position)
    }

    pub fn peek(&self) -> Option<&SpannedToken> {
        self.tokens.get(self.position + 1)
    }
}
