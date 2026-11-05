use cst::{Checkpoint, SyntaxKind, SyntaxNode, SyntaxTreeBuilder};
use lexer::{
    span::{CodeSpan, SpannedToken},
    tokens::{LexerSymbol, LexerTokens},
};
use std::{collections::HashMap, fmt::Debug};

use crate::{
    ast::{declaration::Decl, expression::Expr},
    error::ElpParserError,
    span::Spanned,
};

pub mod ast;
pub mod error;
pub mod parser;
pub mod span;

#[derive(Debug, PartialEq)]
pub enum ElpItem {
    Declaration(Spanned<Decl>),
    Expression(Spanned<Expr>),
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
    pub items: Vec<ElpItem>,
    pub macros: HashMap<String, ElpItem>,
}

impl ElpScript {
    fn from_tokens(tokens: &mut Vec<SpannedToken>) -> Result<Self, ElpParserError> {
        let mut cursor = TokenCursor::new(tokens);
        let mut script = Self::default();

        // I spent too long being stupid trying to work out why i didn't get the first token,
        // it was because i was consuming instead of peeking so we just peek at the next token.
        while let Some(token) = cursor.peek() {
            match &token.token {
                LexerTokens::Identifier(_) => {
                    // first check for declaration keywords.
                    // TODO I wonder if Stylesheet counts as a declaration, it's basically a
                    // language level macro.
                    if Decl::starts_at(&cursor) {
                        let parse_result = Decl::from_cursor(&mut cursor)?;

                        script.items.push(ElpItem::Declaration(parse_result));
                    } else {
                        // Not a declaration keyword, so this identifier starts an
                        // expression (a call, a bare reference, an assignment lvalue, ...).
                        // Previously this arm did nothing, which left the cursor
                        // unadvanced and looped forever on any non-keyword identifier.
                        let parse_result = Expr::from_cursor(&mut cursor)?;
                        script.items.push(ElpItem::Expression(parse_result));
                    }
                }
                LexerTokens::Number(_)
                | LexerTokens::String(_)
                | LexerTokens::Boolean(_)
                | LexerTokens::Symbol(_) => {
                    let parse_result = Expr::from_cursor(&mut cursor)?;
                    script.items.push(ElpItem::Expression(parse_result));
                }
                LexerTokens::Error(msg) => {
                    return Err(ElpParserError::ScriptLexingError(
                        token.span.clone(),
                        msg.clone(),
                    ));
                }
                LexerTokens::Illegal(text) => {
                    return Err(ElpParserError::ScriptLexingError(
                        token.span.clone(),
                        format!("illegal character '{text}'"),
                    ));
                }
                // Trivia. No CST exists yet to attach these to, so for now
                // they're dropped rather than informing the AST — revisit
                // once the CST layer lands.
                LexerTokens::Newline
                | LexerTokens::SOI
                | LexerTokens::EOI
                | LexerTokens::LineComment(_)
                | LexerTokens::DocComment(_)
                | LexerTokens::BlockComment(_) => {
                    cursor.next_token();
                }
            };
        }

        Ok(script)
    }
}

#[derive(Debug, PartialEq)]
pub struct TokenCursor<'t> {
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

    pub fn next_token(&mut self) -> Option<&SpannedToken> {
        self.position += 1;

        self.tokens.get(self.position)
    }

    pub fn peek(&self) -> Option<&SpannedToken> {
        self.tokens.get(self.position + 1)
    }

    /// Lookahead beyond `peek`'s single token, without consuming.
    /// `peek_at(1)` is equivalent to `peek()`; `peek_at(2)` is the token
    /// after that. Needed to disambiguate things a single token of
    /// lookahead can't: a named call argument (`name = expr`) vs. a bare
    /// identifier expression, or `Type { ... }` vs. a plain identifier.
    pub fn peek_at(&self, n: usize) -> Option<&SpannedToken> {
        self.tokens.get(self.position + n)
    }

    fn consume_span_into(&mut self, spans: &mut Vec<CodeSpan>) -> Option<&SpannedToken> {
        let token = self.next_token();

        if let Some(tok) = token {
            spans.push(tok.span.clone());
        }
        token
    }
}

/// Drives a [`TokenCursor`] and a [`SyntaxTreeBuilder`] together: every
/// meaningful token consumed via [`TreeParser::bump`] is both recorded (for
/// `spans_extent`-based error/span reporting, unchanged from before the CST
/// existed) and pushed into the tree under the [`SyntaxKind`] the caller
/// says it is. This is how a `Decl`/`Expr` parsing function becomes a typed
/// view over a CST instead of hand-accumulating fields as it goes.
///
/// Newlines and other not-yet-modelled trivia are skipped via
/// [`TreeParser::skip_trivia`] rather than `bump`, so they're still recorded
/// for span purposes but don't appear in the tree yet — full trivia
/// preservation (needed for exact source round-tripping) is follow-up work,
/// gated on the lexer emitting whitespace as real tokens.
pub struct TreeParser<'c, 't> {
    cursor: &'c mut TokenCursor<'t>,
    builder: SyntaxTreeBuilder,
}

impl<'c, 't> TreeParser<'c, 't> {
    pub fn new(cursor: &'c mut TokenCursor<'t>) -> Self {
        Self {
            cursor,
            builder: SyntaxTreeBuilder::new(),
        }
    }

    pub fn peek(&self) -> Option<&SpannedToken> {
        self.cursor.peek()
    }

    pub fn peek_at(&self, n: usize) -> Option<&SpannedToken> {
        self.cursor.peek_at(n)
    }

    /// True at true end-of-input *or* at the `EOI` token — the lexer always
    /// emits a real `EOI` token at the end of the stream (even for empty
    /// input), so `peek().is_none()` alone never actually detects "out of
    /// input" inside a loop; it only fires if something already walked
    /// past `EOI`, which nothing should do. Loops that scan for a closing
    /// delimiter (`'}'`, `')'`, ...) and want to fail cleanly on truncated
    /// input rather than falling through to a confusing "expected an
    /// expression: EOI" should check this instead of `peek().is_none()`.
    pub fn at_end(&self) -> bool {
        matches!(self.peek().map(|t| &t.token), None | Some(LexerTokens::EOI))
    }

    pub fn start_node(&mut self, kind: SyntaxKind) {
        self.builder.start_node(kind);
    }

    pub fn finish_node(&mut self) {
        self.builder.finish_node();
    }

    /// See [`SyntaxTreeBuilder::checkpoint`] — lets a binary expression's
    /// LHS be parsed before deciding whether it needs wrapping.
    pub fn checkpoint(&mut self) -> Checkpoint {
        self.builder.checkpoint()
    }

    pub fn start_node_at(&mut self, checkpoint: Checkpoint, kind: SyntaxKind) {
        self.builder.start_node_at(checkpoint, kind);
    }

    /// Consumes the next token as `kind`: records its span and pushes its
    /// canonical text into the tree. No-op if there's no token left.
    pub fn bump(&mut self, kind: SyntaxKind, spans: &mut Vec<CodeSpan>) {
        let Some(tok) = self.cursor.peek() else {
            return;
        };
        let text = token_text(&tok.token);
        let span = tok.span.clone();
        self.cursor.next_token();
        spans.push(span);
        self.builder.token(kind, &text);
    }

    /// Consumes the next token without adding it to the tree, recording
    /// only its span. For trivia the tree doesn't model yet (see struct docs).
    pub fn skip_trivia(&mut self, spans: &mut Vec<CodeSpan>) {
        self.cursor.consume_span_into(spans);
    }

    /// Finishes the tree and returns its root. The root's `SyntaxKind` is
    /// whichever `start_node` call is still open when the matching
    /// `finish_node` closes it off — callers `AstNode::cast` this into the
    /// specific node type they expect.
    pub fn finish(self) -> SyntaxNode {
        self.builder.finish()
    }
}

/// The exact source text a token owns, or its canonical fixed text for
/// symbols/keywords. Used to feed [`SyntaxTreeBuilder::token`], which needs
/// real text rather than a `Debug`-style tag (`"="`, not `"Equal"`).
fn token_text(tok: &LexerTokens) -> String {
    match tok {
        LexerTokens::Identifier(s)
        | LexerTokens::Number(s)
        | LexerTokens::String(s)
        | LexerTokens::Error(s)
        | LexerTokens::Illegal(s) => s.clone(),
        LexerTokens::Boolean(b) => b.to_string(),
        LexerTokens::Symbol(sym) => symbol_text(sym).to_string(),
        LexerTokens::LineComment(s) => format!("//{s}"),
        LexerTokens::DocComment(s) => format!("///{s}"),
        LexerTokens::BlockComment(s) => format!("/*{s}*/"),
        // Known fidelity gap: the lexer doesn't record whether a newline was
        // originally "\n", "\r", or "\r\n", so this always renders as "\n".
        LexerTokens::Newline => "\n".to_string(),
        LexerTokens::SOI | LexerTokens::EOI => String::new(),
    }
}

fn symbol_text(sym: &LexerSymbol) -> &'static str {
    match sym {
        LexerSymbol::Equal => "=",
        LexerSymbol::Plus => "+",
        LexerSymbol::Minus => "-",
        LexerSymbol::Multiply => "*",
        LexerSymbol::Divide => "/",
        LexerSymbol::SemiColon => ";",
        LexerSymbol::EqualEqual => "==",
        LexerSymbol::Arrow => "->",
        LexerSymbol::PlusEqual => "+=",
        LexerSymbol::MinusEqual => "-=",
        LexerSymbol::MultiplyEqual => "*=",
        LexerSymbol::DivideEqual => "/=",
        LexerSymbol::AmpersandEqual => "&=",
        LexerSymbol::PipeEqual => "|=",
        LexerSymbol::NotEqual => "!=",
        LexerSymbol::LCurley => "{",
        LexerSymbol::RCurley => "}",
        LexerSymbol::Comma => ",",
        LexerSymbol::Period => ".",
        LexerSymbol::Range => "..",
        LexerSymbol::Colon => ":",
        LexerSymbol::At => "@",
        LexerSymbol::LessThan => "<",
        LexerSymbol::GreaterThan => ">",
        LexerSymbol::LessEqual => "<=",
        LexerSymbol::GreaterEqual => ">=",
        LexerSymbol::AndAnd => "&&",
        LexerSymbol::OrOr => "||",
        LexerSymbol::Percent => "%",
        LexerSymbol::Ampersand => "&",
        LexerSymbol::Bang => "!",
        LexerSymbol::Question => "?",
        LexerSymbol::LParen => "(",
        LexerSymbol::RParen => ")",
        LexerSymbol::LBracket => "[",
        LexerSymbol::RBracket => "]",
        LexerSymbol::Error => "<error>",
    }
}

#[cfg(test)]
mod dispatch_tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn parse(src: &str) -> Result<ElpScript, ElpParserError> {
        let mut lexer = lexer::ElpLexer::new(src);
        let mut tokens = lexer.scan();
        ElpScript::from_tokens(&mut tokens)
    }

    #[test]
    fn file_ext_maps_every_script_type() {
        assert_eq!(ElpScriptType::Script.file_ext(), "elp");
        assert_eq!(ElpScriptType::ViewScript.file_ext(), "velp");
        assert_eq!(ElpScriptType::Stylesheet.file_ext(), "selp");
    }

    // Regression: this arm used to do nothing on a non-keyword identifier,
    // which left the cursor unadvanced and looped `while let Some(token) =
    // cursor.peek()` forever — a real hang, not just a wrong result. If
    // this regresses, this test hangs rather than failing cleanly; that's
    // the nature of testing an infinite-loop fix, but a hang here is
    // itself the signal something's badly wrong.
    #[test]
    fn non_keyword_identifier_at_top_level_terminates_as_an_expression() {
        let script = parse("println(\"hi\")").expect("bare call should parse as an expression");
        assert_eq!(script.items.len(), 1);
        assert!(matches!(script.items[0], ElpItem::Expression(_)));
    }

    #[test]
    fn multiple_non_keyword_expression_statements_all_advance() {
        // Same regression, but proves the *loop* keeps making progress
        // across several such items, not just that one call parses.
        let script = parse("a\nb\nc").expect("bare identifiers should each parse");
        assert_eq!(script.items.len(), 3);
        assert!(
            script
                .items
                .iter()
                .all(|item| matches!(item, ElpItem::Expression(_)))
        );
    }

    // `TypeName implementing A + B` is the only declaration whose first
    // token is an ordinary identifier, so it's the only one that can be
    // *silently* misrouted: if the two-token gate in `Decl::starts_at`
    // regressed, this would parse as three unrelated bare-identifier
    // expressions and raise no error at all. Asserting on the item kind
    // here (rather than just "it parsed") is the whole point — the corpus
    // test can't catch this, since a wrong split still parses cleanly.
    #[test]
    fn implementing_declaration_dispatches_as_a_declaration_not_expressions() {
        use crate::ast::declaration::Decl;

        let script = parse("ChatState implementing State + Copy")
            .expect("an implementing declaration should parse");
        assert_eq!(script.items.len(), 1);
        let ElpItem::Declaration(decl) = &script.items[0] else {
            panic!("expected a declaration, got {:?}", script.items[0])
        };
        let Decl::Implementing(i) = &decl.node else {
            panic!("expected Decl::Implementing, got {:?}", decl.node)
        };
        assert_eq!(i.type_name(), "ChatState");
        assert_eq!(i.interfaces().len(), 2);
    }

    #[test]
    fn a_bare_identifier_is_still_an_expression_not_a_declaration() {
        // The other side of the same gate: two-token lookahead must not
        // claim an identifier that simply happens to be followed by
        // another one on the next line.
        let script = parse("ChatState\nCopy").expect("bare identifiers should parse");
        assert_eq!(script.items.len(), 2);
        assert!(
            script
                .items
                .iter()
                .all(|item| matches!(item, ElpItem::Expression(_)))
        );
    }

    // Regression: `LexerTokens::Illegal`/`Error` used to hit `todo!()` in
    // this dispatch, panicking instead of surfacing a normal parse error.
    #[test]
    fn illegal_character_is_a_clean_error_not_a_panic() {
        let err = parse("§").expect_err("an illegal character should error, not panic");
        assert!(
            matches!(err, ElpParserError::ScriptLexingError(_, _)),
            "unexpected error variant: {err}"
        );
    }

    #[test]
    fn illegal_character_after_valid_content_is_still_a_clean_error() {
        let err =
            parse("const x = 1\n§").expect_err("an illegal character should error, not panic");
        assert!(matches!(err, ElpParserError::ScriptLexingError(_, _)));
    }

    // `LexerTokens::Error` isn't ever actually constructed by today's
    // lexer (nothing in `ElpLexer::scan` produces it), so exercising this
    // dispatch arm means building the token stream by hand rather than
    // lexing real source.
    #[test]
    fn lexer_error_token_is_a_clean_error_not_a_panic() {
        let zero_span = || CodeSpan {
            start: 0,
            end: 0,
            display_column: 1,
            line: 1,
        };
        let mut tokens = vec![
            SpannedToken {
                span: zero_span(),
                token: LexerTokens::SOI,
            },
            SpannedToken {
                span: zero_span(),
                token: LexerTokens::Error("lexer is confused".to_string()),
            },
            SpannedToken {
                span: zero_span(),
                token: LexerTokens::EOI,
            },
        ];
        let err = ElpScript::from_tokens(&mut tokens)
            .expect_err("a lexer Error token should error, not panic");
        assert!(
            matches!(err, ElpParserError::ScriptLexingError(_, msg) if msg == "lexer is confused")
        );
    }
}

#[cfg(test)]
mod corpus_tests {
    use super::*;
    use std::{fs, path::Path};

    // Phase 0 exit criteria per the front-end roadmap: every .elp file in
    // examples/ should parse without panicking. This is deliberately NOT a
    // hard pass/fail gate — several files are known to need features that
    // are explicitly deferred (macro attributes like `@external`/`@CodeGen`,
    // generic args, match/if expressions, struct tags). The point of this
    // test is to fail loudly with a clear file+error report if something
    // regresses, and to make today's real coverage visible rather than
    // implicit. Update the `expect_success` list as more of the grammar
    // lands — do NOT loosen it just to make the test pass.
    #[test]
    fn parses_expected_files_in_example_corpus() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples");
        let mut results = vec![];

        for entry in glob::glob(&format!("{}/**/*.elp", root.display())).unwrap() {
            let path = entry.unwrap();
            let source = fs::read_to_string(&path).unwrap();
            let mut lexer = lexer::ElpLexer::new(&source);
            let mut tokens = lexer.scan();
            let result = ElpScript::from_tokens(&mut tokens);
            let rel = path.strip_prefix(&root).unwrap_or(&path).to_path_buf();
            results.push((rel, result));
        }

        assert!(
            !results.is_empty(),
            "expected to find .elp files under examples/"
        );

        // Known-good today: plain declarations/imports only, no macro
        // attributes, generics, match/if expressions, or struct tags.
        let expect_success = [
            "BlogStyle/constants.elp",
            "ChatRoom/types/chat.elp",
            "ChatRoom/state/app.elp",
        ];

        let mut failures = vec![];
        for (path, result) in &results {
            let path_str = path.to_string_lossy();
            let should_succeed = expect_success.iter().any(|p| path_str == *p);
            match (should_succeed, result) {
                (true, Err(e)) => {
                    failures.push(format!("{path_str}: expected to parse, got error: {e}"))
                }
                (false, Ok(_)) => failures.push(format!(
                    "{path_str}: parsed successfully — move it into `expect_success` \
                     (a deferred-feature gap just got closed, which is good news, \
                     but the test needs updating to reflect it)"
                )),
                _ => {}
            }
        }

        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }
}
