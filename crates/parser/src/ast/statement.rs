//! `block`/`statement`/`pattern` grammar rules. This is what
//! task #9 ("block-dependent expression forms") was waiting on: `ifExpr`,
//! `matchExpr`, `lambdaExpr`, and the trailing `builderBlock` after a call
//! all need real statement parsing, not the opaque brace-balancing skip
//! `fn` bodies used before this module existed.
//!
//! `parse_if_expr`/`parse_match_expr` live here (not in `expression.rs`)
//! because they're reachable from both `statement` and `primary` in the
//! grammar — a bare `if cond { ... }` is a statement, but `const x = if
//! cond { 1 } else { 2 }` uses the exact same rule in expression position.
//! `expression.rs`'s `parse_primary` calls into these directly.

use cst::{AstNode, SyntaxKind, SyntaxNode};
use lexer::{
    span::CodeSpan,
    tokens::{LexerSymbol, LexerTokens},
};

use crate::{
    TreeParser,
    ast::{
        declaration,
        expression::{self, Expr},
    },
    error::{ElpParserError, expected_got, unexpected},
};

macro_rules! ast_node {
    ($name:ident, $kind:path) => {
        #[derive(Debug, PartialEq, Clone)]
        pub struct $name(SyntaxNode);

        impl AstNode for $name {
            fn can_cast(kind: SyntaxKind) -> bool {
                kind == $kind
            }

            fn cast(node: SyntaxNode) -> Option<Self> {
                Self::can_cast(node.kind()).then(|| Self(node))
            }

            fn syntax(&self) -> &SyntaxNode {
                &self.0
            }
        }
    };
}

ast_node!(Block, SyntaxKind::BLOCK);
ast_node!(ReturnStmt, SyntaxKind::RETURN_STMT);
ast_node!(Assignment, SyntaxKind::ASSIGNMENT);
ast_node!(ForStmt, SyntaxKind::FOR_STMT);
ast_node!(MatchArm, SyntaxKind::MATCH_ARM);

impl Block {
    pub fn statements(&self) -> Vec<Statement> {
        self.0.children().filter_map(Statement::cast).collect()
    }
}

impl ReturnStmt {
    pub fn value(&self) -> Option<Expr> {
        self.0.children().find_map(Expr::cast)
    }
}

fn assign_op_text(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::EQ
            | SyntaxKind::PLUSEQ
            | SyntaxKind::MINUSEQ
            | SyntaxKind::STAREQ
            | SyntaxKind::SLASHEQ
            | SyntaxKind::AMPEQ
            | SyntaxKind::PIPEEQ
    )
}

impl Assignment {
    pub fn target(&self) -> Option<Expr> {
        self.0.children().filter_map(Expr::cast).next()
    }

    pub fn op_text(&self) -> String {
        self.0
            .children_with_tokens()
            .filter_map(|el| el.into_token())
            .find(|t| assign_op_text(t.kind()))
            .map(|t| t.text().to_string())
            .unwrap_or_default()
    }

    pub fn value(&self) -> Option<Expr> {
        self.0.children().filter_map(Expr::cast).nth(1)
    }
}

impl ForStmt {
    pub fn binding(&self) -> String {
        self.0
            .children_with_tokens()
            .filter_map(|el| el.into_token())
            .find(|t| t.kind() == SyntaxKind::IDENT)
            .map(|t| t.text().to_string())
            .unwrap_or_default()
    }

    pub fn iterable(&self) -> Option<Expr> {
        self.0.children().find_map(Expr::cast)
    }

    pub fn body(&self) -> Option<Block> {
        self.0.children().find_map(Block::cast)
    }
}

impl MatchArm {
    pub fn body(&self) -> Option<Expr> {
        self.0.children().find_map(Expr::cast)
    }

    pub fn body_block(&self) -> Option<Block> {
        self.0.children().find_map(Block::cast)
    }

    /// Raw pattern-node kind (`WILDCARD_PATTERN`, `BINDING_PATTERN`,
    /// `LITERAL_EXPR`, `VARIANT_PATTERN`, `RANGE_PATTERN`,
    /// `RANGE_CATCHALL_PATTERN`, `TUPLE_PATTERN`) — every pattern form
    /// parses to a node (even the bare `..` catch-all), so this is just
    /// the arm's first child. No typed `Pattern` enum yet; this is enough
    /// for tests and for a future typechecker to dispatch on.
    pub fn pattern_kind(&self) -> Option<SyntaxKind> {
        self.0.first_child().map(|n| n.kind())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Var(declaration::VarDecl),
    Const(declaration::ConstDecl),
    /// `*fn name(...) -> T` — a callback slot. `export`ed, it's a prop.
    Callback(declaration::CallbackDecl),
    Assignment(Assignment),
    Return(ReturnStmt),
    If(expression::IfExpr),
    Match(expression::MatchExpr),
    For(ForStmt),
    Expr(Expr),
}

impl AstNode for Statement {
    fn can_cast(kind: SyntaxKind) -> bool {
        matches!(
            kind,
            SyntaxKind::VAR_DECL
                | SyntaxKind::CONST_DECL
                | SyntaxKind::CALLBACK_DECL
                | SyntaxKind::ASSIGNMENT
                | SyntaxKind::RETURN_STMT
                | SyntaxKind::IF_EXPR
                | SyntaxKind::MATCH_EXPR
                | SyntaxKind::FOR_STMT
        ) || Expr::can_cast(kind)
    }

    fn cast(node: SyntaxNode) -> Option<Self> {
        match node.kind() {
            SyntaxKind::VAR_DECL => Some(Statement::Var(declaration::VarDecl::cast(node)?)),
            SyntaxKind::CONST_DECL => Some(Statement::Const(declaration::ConstDecl::cast(node)?)),
            SyntaxKind::CALLBACK_DECL => {
                Some(Statement::Callback(declaration::CallbackDecl::cast(node)?))
            }
            SyntaxKind::ASSIGNMENT => Some(Statement::Assignment(Assignment::cast(node)?)),
            SyntaxKind::RETURN_STMT => Some(Statement::Return(ReturnStmt::cast(node)?)),
            SyntaxKind::IF_EXPR => Some(Statement::If(expression::IfExpr::cast(node)?)),
            SyntaxKind::MATCH_EXPR => Some(Statement::Match(expression::MatchExpr::cast(node)?)),
            SyntaxKind::FOR_STMT => Some(Statement::For(ForStmt::cast(node)?)),
            _ => Expr::cast(node).map(Statement::Expr),
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        match self {
            Statement::Var(n) => n.syntax(),
            Statement::Const(n) => n.syntax(),
            Statement::Callback(n) => n.syntax(),
            Statement::Assignment(n) => n.syntax(),
            Statement::Return(n) => n.syntax(),
            Statement::If(n) => n.syntax(),
            Statement::Match(n) => n.syntax(),
            Statement::For(n) => n.syntax(),
            Statement::Expr(n) => n.syntax(),
        }
    }
}

pub(crate) fn parse_block(
    p: &mut TreeParser,
    spans: &mut Vec<CodeSpan>,
) -> Result<(), ElpParserError> {
    p.start_node(SyntaxKind::BLOCK);
    p.bump(SyntaxKind::LCURLY, spans);

    loop {
        declaration::skip_newlines(p, spans);
        if matches!(
            p.peek().map(|t| &t.token),
            Some(LexerTokens::Symbol(LexerSymbol::RCurley))
        ) {
            break;
        }
        if p.at_end() {
            return Err(expected_got(None, "'}' to close block"));
        }
        parse_statement(p, spans)?;
    }

    p.bump(SyntaxKind::RCURLY, spans);
    p.finish_node(); // BLOCK
    Ok(())
}

fn parse_statement(p: &mut TreeParser, spans: &mut Vec<CodeSpan>) -> Result<(), ElpParserError> {
    match p.peek().map(|t| &t.token) {
        // `export` in statement position marks a component prop — a
        // reactive `var`/`const`, or a `*fn` callback slot. `export` is
        // pure visibility (see memory-model.md); what follows it decides
        // the form, so dispatch on the *second* token.
        Some(LexerTokens::Identifier(kw)) if kw == "export" => {
            match p.peek_at(2).map(|t| &t.token) {
                Some(LexerTokens::Identifier(k)) if k == "var" => {
                    declaration::parse_var_or_const_body(
                        p,
                        spans,
                        SyntaxKind::VAR_DECL,
                        SyntaxKind::VAR_KW,
                    )
                }
                Some(LexerTokens::Identifier(k)) if k == "const" => {
                    declaration::parse_var_or_const_body(
                        p,
                        spans,
                        SyntaxKind::CONST_DECL,
                        SyntaxKind::CONST_KW,
                    )
                }
                Some(LexerTokens::Symbol(LexerSymbol::Multiply)) => {
                    declaration::parse_callback_decl(p, spans)
                }
                _ => Err(expected_got(
                    p.peek_at(2),
                    "'var', 'const' or '*fn' after 'export' — only props can be exported \
                     from inside a block",
                )),
            }
        }
        // A bare `*fn` is an error — callbacks must be exported — but the
        // arm still claims it so the message comes from the callback parser
        // rather than the expression parser choking on a leading `*`.
        Some(LexerTokens::Symbol(LexerSymbol::Multiply)) => {
            declaration::parse_callback_decl(p, spans)
        }
        Some(LexerTokens::Identifier(kw)) if kw == "var" => {
            declaration::parse_var_or_const_body(p, spans, SyntaxKind::VAR_DECL, SyntaxKind::VAR_KW)
        }
        Some(LexerTokens::Identifier(kw)) if kw == "const" => declaration::parse_var_or_const_body(
            p,
            spans,
            SyntaxKind::CONST_DECL,
            SyntaxKind::CONST_KW,
        ),
        Some(LexerTokens::Identifier(kw)) if kw == "return" => parse_return_stmt(p, spans),
        Some(LexerTokens::Identifier(kw)) if kw == "if" => parse_if_expr(p, spans),
        Some(LexerTokens::Identifier(kw)) if kw == "match" => parse_match_expr(p, spans),
        Some(LexerTokens::Identifier(kw)) if kw == "for" => parse_for_stmt(p, spans),
        _ => parse_assignment_or_expr_stmt(p, spans),
    }
}

fn parse_return_stmt(p: &mut TreeParser, spans: &mut Vec<CodeSpan>) -> Result<(), ElpParserError> {
    p.start_node(SyntaxKind::RETURN_STMT);
    p.bump(SyntaxKind::RETURN_KW, spans);

    // `expr?` — deliberately checked *without* skipping trivia first: a
    // newline immediately after `return` means no value (Go-style ASI),
    // not "the value is on the next line". Skipping it here would risk
    // swallowing the start of an unrelated following statement.
    let starts_expr = !matches!(
        p.peek().map(|t| &t.token),
        None | Some(LexerTokens::Newline) | Some(LexerTokens::Symbol(LexerSymbol::RCurley))
    );
    if starts_expr {
        expression::parse_binary(p, spans, 0)?;
    }
    p.finish_node(); // RETURN_STMT
    Ok(())
}

fn parse_for_stmt(p: &mut TreeParser, spans: &mut Vec<CodeSpan>) -> Result<(), ElpParserError> {
    p.start_node(SyntaxKind::FOR_STMT);
    p.bump(SyntaxKind::FOR_KW, spans);

    let name_tok = p
        .peek()
        .cloned()
        .ok_or_else(|| expected_got(None, "a loop variable"))?;
    if !matches!(name_tok.token, LexerTokens::Identifier(_)) {
        return Err(expected_got(Some(&name_tok), "a loop variable"));
    }
    p.bump(SyntaxKind::IDENT, spans);

    let in_tok = p
        .peek()
        .cloned()
        .ok_or_else(|| expected_got(None, "'in'"))?;
    if matches!(&in_tok.token, LexerTokens::Identifier(i) if i == "in") {
        p.bump(SyntaxKind::IN_KW, spans);
    } else {
        return Err(expected_got(Some(&in_tok), "'in'"));
    }

    expression::parse_binary_no_struct_literal(p, spans, 0)?;

    let brace = p
        .peek()
        .cloned()
        .ok_or_else(|| expected_got(None, "'{' to start loop body"))?;
    if !matches!(brace.token, LexerTokens::Symbol(LexerSymbol::LCurley)) {
        return Err(expected_got(Some(&brace), "'{' to start loop body"));
    }
    parse_block(p, spans)?;

    p.finish_node(); // FOR_STMT
    Ok(())
}

fn assign_op_kind(sym: &LexerSymbol) -> Option<SyntaxKind> {
    use LexerSymbol::*;
    Some(match sym {
        Equal => SyntaxKind::EQ,
        PlusEqual => SyntaxKind::PLUSEQ,
        MinusEqual => SyntaxKind::MINUSEQ,
        MultiplyEqual => SyntaxKind::STAREQ,
        DivideEqual => SyntaxKind::SLASHEQ,
        AmpersandEqual => SyntaxKind::AMPEQ,
        PipeEqual => SyntaxKind::PIPEEQ,
        _ => return None,
    })
}

/// `lvalue assignOp expr` vs. a bare `exprStmt` — both start with an
/// expression, so parse the full (postfix-chain-capable) expression first
/// via a checkpoint, then retroactively wrap in `ASSIGNMENT` only if an
/// assignment operator actually follows. No validation that the parsed
/// expression is actually a valid `lvalue` (`IDENT (memberAccess | index)*`)
/// — that's a semantic-pass concern once one exists, not this layer's.
fn parse_assignment_or_expr_stmt(
    p: &mut TreeParser,
    spans: &mut Vec<CodeSpan>,
) -> Result<(), ElpParserError> {
    let checkpoint = p.checkpoint();
    expression::parse_binary(p, spans, 0)?;

    let assign_kind = match p.peek().map(|t| &t.token) {
        Some(LexerTokens::Symbol(sym)) => assign_op_kind(sym),
        _ => None,
    };

    if let Some(kind) = assign_kind {
        p.start_node_at(checkpoint, SyntaxKind::ASSIGNMENT);
        p.bump(kind, spans);
        expression::parse_binary(p, spans, 0)?;
        p.finish_node(); // ASSIGNMENT
    }
    Ok(())
}

// ---------- if / match (shared between statement and expression position) ----------

pub(crate) fn parse_if_expr(
    p: &mut TreeParser,
    spans: &mut Vec<CodeSpan>,
) -> Result<(), ElpParserError> {
    p.start_node(SyntaxKind::IF_EXPR);
    p.bump(SyntaxKind::IF_KW, spans);
    expression::parse_binary_no_struct_literal(p, spans, 0)?;

    let brace = p
        .peek()
        .cloned()
        .ok_or_else(|| expected_got(None, "'{' to start if body"))?;
    if !matches!(brace.token, LexerTokens::Symbol(LexerSymbol::LCurley)) {
        return Err(expected_got(Some(&brace), "'{' to start if body"));
    }
    parse_block(p, spans)?;

    if matches!(p.peek().map(|t| &t.token), Some(LexerTokens::Identifier(i)) if i == "else") {
        p.bump(SyntaxKind::ELSE_KW, spans);
        if matches!(p.peek().map(|t| &t.token), Some(LexerTokens::Identifier(i)) if i == "if") {
            parse_if_expr(p, spans)?;
        } else {
            let next = p
                .peek()
                .cloned()
                .ok_or_else(|| expected_got(None, "'{' or 'if' after else"))?;
            if matches!(next.token, LexerTokens::Symbol(LexerSymbol::LCurley)) {
                parse_block(p, spans)?;
            } else {
                return Err(expected_got(Some(&next), "'{' or 'if' after else"));
            }
        }
    }

    p.finish_node(); // IF_EXPR
    Ok(())
}

pub(crate) fn parse_match_expr(
    p: &mut TreeParser,
    spans: &mut Vec<CodeSpan>,
) -> Result<(), ElpParserError> {
    p.start_node(SyntaxKind::MATCH_EXPR);
    p.bump(SyntaxKind::MATCH_KW, spans);
    expression::parse_binary_no_struct_literal(p, spans, 0)?;

    let brace = p
        .peek()
        .cloned()
        .ok_or_else(|| expected_got(None, "'{' to start match body"))?;
    if !matches!(brace.token, LexerTokens::Symbol(LexerSymbol::LCurley)) {
        return Err(expected_got(Some(&brace), "'{' to start match body"));
    }
    p.bump(SyntaxKind::LCURLY, spans);

    p.start_node(SyntaxKind::MATCH_ARMS);
    loop {
        declaration::skip_newlines(p, spans);
        if matches!(
            p.peek().map(|t| &t.token),
            Some(LexerTokens::Symbol(LexerSymbol::RCurley))
        ) {
            break;
        }
        if p.at_end() {
            return Err(expected_got(None, "a match arm or '}'"));
        }
        parse_match_arm(p, spans)?;
        declaration::skip_newlines(p, spans);
        if matches!(
            p.peek().map(|t| &t.token),
            Some(LexerTokens::Symbol(LexerSymbol::Comma))
        ) {
            p.bump(SyntaxKind::COMMA, spans);
        }
    }
    p.finish_node(); // MATCH_ARMS

    p.bump(SyntaxKind::RCURLY, spans);
    p.finish_node(); // MATCH_EXPR
    Ok(())
}

fn parse_match_arm(p: &mut TreeParser, spans: &mut Vec<CodeSpan>) -> Result<(), ElpParserError> {
    p.start_node(SyntaxKind::MATCH_ARM);
    parse_pattern(p, spans)?;

    let arrow = p
        .peek()
        .cloned()
        .ok_or_else(|| expected_got(None, "'->'"))?;
    if !matches!(arrow.token, LexerTokens::Symbol(LexerSymbol::Arrow)) {
        return Err(expected_got(Some(&arrow), "'->'"));
    }
    p.bump(SyntaxKind::ARROW, spans);

    // armBody: block if next is '{', else a bare expr.
    if matches!(
        p.peek().map(|t| &t.token),
        Some(LexerTokens::Symbol(LexerSymbol::LCurley))
    ) {
        parse_block(p, spans)?;
    } else {
        expression::parse_binary(p, spans, 0)?;
    }

    p.finish_node(); // MATCH_ARM
    Ok(())
}

// ---------- patterns ----------

/// `rangePat` (`expr '..' expr`) is genuinely ambiguous with every other
/// pattern form at the start (`400..499` starts exactly like the literal
/// pattern `400`) — same checkpoint-retroactive-wrap technique as binary
/// expressions and `type`'s postfix `?`: parse one atom, then check for a
/// trailing `..` and wrap only if it's there.
fn parse_pattern(p: &mut TreeParser, spans: &mut Vec<CodeSpan>) -> Result<(), ElpParserError> {
    let checkpoint = p.checkpoint();
    parse_pattern_atom(p, spans)?;

    if matches!(
        p.peek().map(|t| &t.token),
        Some(LexerTokens::Symbol(LexerSymbol::Range))
    ) {
        p.start_node_at(checkpoint, SyntaxKind::RANGE_PATTERN);
        p.bump(SyntaxKind::RANGE, spans);
        parse_pattern_atom(p, spans)?;
        p.finish_node(); // RANGE_PATTERN
    }
    Ok(())
}

fn parse_pattern_atom(p: &mut TreeParser, spans: &mut Vec<CodeSpan>) -> Result<(), ElpParserError> {
    let Some(tok) = p.peek().cloned() else {
        return Err(expected_got(None, "a pattern"));
    };

    match &tok.token {
        // Bare `..` catch-all (`.. -> ...`) — distinct from a range
        // pattern, which needs an atom *before* the `..` too.
        LexerTokens::Symbol(LexerSymbol::Range) => {
            p.start_node(SyntaxKind::RANGE_CATCHALL_PATTERN);
            p.bump(SyntaxKind::RANGE, spans);
            p.finish_node();
            Ok(())
        }
        LexerTokens::Number(_) | LexerTokens::String(_) => parse_literal_pattern(p, spans),
        LexerTokens::Identifier(name) if matches!(name.as_str(), "true" | "false" | "nil") => {
            parse_literal_pattern(p, spans)
        }
        LexerTokens::Identifier(name) if name == "_" => {
            p.start_node(SyntaxKind::WILDCARD_PATTERN);
            p.bump(SyntaxKind::IDENT, spans);
            p.finish_node();
            Ok(())
        }
        LexerTokens::Symbol(LexerSymbol::Period) => parse_contextual_variant_pattern(p, spans),
        LexerTokens::Symbol(LexerSymbol::LParen) => parse_tuple_pattern(p, spans),
        LexerTokens::Identifier(_) => {
            // `IDENT '(' patternList ')'` (qualified variant) vs. a bare
            // `IDENT` binding — needs 2-token lookahead for the `(`.
            if matches!(
                p.peek_at(2).map(|t| &t.token),
                Some(LexerTokens::Symbol(LexerSymbol::LParen))
            ) {
                parse_qualified_variant_pattern(p, spans)
            } else {
                p.start_node(SyntaxKind::BINDING_PATTERN);
                p.bump(SyntaxKind::IDENT, spans);
                p.finish_node();
                Ok(())
            }
        }
        _ => Err(unexpected(&tok, "expected a pattern")),
    }
}

/// Duplicates a handful of lines of `expression::parse_primary`'s literal
/// branch rather than sharing it — the two contexts diverge enough
/// (patterns have no unary/postfix/binary wrapping) that factoring it out
/// wasn't worth it for four match arms. Worth revisiting if it grows.
fn parse_literal_pattern(
    p: &mut TreeParser,
    spans: &mut Vec<CodeSpan>,
) -> Result<(), ElpParserError> {
    let tok = p
        .peek()
        .cloned()
        .ok_or_else(|| expected_got(None, "a literal pattern"))?;
    p.start_node(SyntaxKind::LITERAL_EXPR);
    match &tok.token {
        LexerTokens::Number(text) => {
            let kind = if text.contains('.') {
                SyntaxKind::FLOAT
            } else {
                SyntaxKind::INTEGER
            };
            p.bump(kind, spans);
        }
        LexerTokens::String(_) => p.bump(SyntaxKind::STRING, spans),
        LexerTokens::Identifier(name) if name == "true" => p.bump(SyntaxKind::TRUE_KW, spans),
        LexerTokens::Identifier(name) if name == "false" => p.bump(SyntaxKind::FALSE_KW, spans),
        LexerTokens::Identifier(name) if name == "nil" => p.bump(SyntaxKind::NIL_KW, spans),
        _ => return Err(unexpected(&tok, "expected a literal pattern")),
    }
    p.finish_node(); // LITERAL_EXPR
    Ok(())
}

fn parse_contextual_variant_pattern(
    p: &mut TreeParser,
    spans: &mut Vec<CodeSpan>,
) -> Result<(), ElpParserError> {
    p.start_node(SyntaxKind::VARIANT_PATTERN);
    p.bump(SyntaxKind::DOT, spans);
    let name = p
        .peek()
        .cloned()
        .ok_or_else(|| expected_got(None, "a variant name after '.'"))?;
    if !matches!(name.token, LexerTokens::Identifier(_)) {
        return Err(expected_got(Some(&name), "a variant name after '.'"));
    }
    p.bump(SyntaxKind::IDENT, spans);
    parse_optional_pattern_payload(p, spans)?;
    p.finish_node(); // VARIANT_PATTERN
    Ok(())
}

fn parse_qualified_variant_pattern(
    p: &mut TreeParser,
    spans: &mut Vec<CodeSpan>,
) -> Result<(), ElpParserError> {
    p.start_node(SyntaxKind::VARIANT_PATTERN);
    p.bump(SyntaxKind::IDENT, spans);
    parse_optional_pattern_payload(p, spans)?;
    p.finish_node(); // VARIANT_PATTERN
    Ok(())
}

fn parse_optional_pattern_payload(
    p: &mut TreeParser,
    spans: &mut Vec<CodeSpan>,
) -> Result<(), ElpParserError> {
    if !matches!(
        p.peek().map(|t| &t.token),
        Some(LexerTokens::Symbol(LexerSymbol::LParen))
    ) {
        return Ok(());
    }
    p.bump(SyntaxKind::LPAREN, spans);
    parse_pattern_list(p, spans)?;
    let close = p
        .peek()
        .cloned()
        .ok_or_else(|| expected_got(None, "')' to close variant pattern"))?;
    if matches!(close.token, LexerTokens::Symbol(LexerSymbol::RParen)) {
        p.bump(SyntaxKind::RPAREN, spans);
    } else {
        return Err(expected_got(Some(&close), "')' to close variant pattern"));
    }
    Ok(())
}

fn parse_pattern_list(p: &mut TreeParser, spans: &mut Vec<CodeSpan>) -> Result<(), ElpParserError> {
    p.start_node(SyntaxKind::PATTERN_LIST);
    parse_pattern(p, spans)?;
    while matches!(
        p.peek().map(|t| &t.token),
        Some(LexerTokens::Symbol(LexerSymbol::Comma))
    ) {
        p.bump(SyntaxKind::COMMA, spans);
        parse_pattern(p, spans)?;
    }
    p.finish_node(); // PATTERN_LIST
    Ok(())
}

fn parse_tuple_pattern(
    p: &mut TreeParser,
    spans: &mut Vec<CodeSpan>,
) -> Result<(), ElpParserError> {
    p.start_node(SyntaxKind::TUPLE_PATTERN);
    p.bump(SyntaxKind::LPAREN, spans);
    parse_pattern(p, spans)?;
    while matches!(
        p.peek().map(|t| &t.token),
        Some(LexerTokens::Symbol(LexerSymbol::Comma))
    ) {
        p.bump(SyntaxKind::COMMA, spans);
        if matches!(
            p.peek().map(|t| &t.token),
            Some(LexerTokens::Symbol(LexerSymbol::RParen))
        ) {
            break;
        }
        parse_pattern(p, spans)?;
    }
    let close = p
        .peek()
        .cloned()
        .ok_or_else(|| expected_got(None, "')' to close tuple pattern"))?;
    if matches!(close.token, LexerTokens::Symbol(LexerSymbol::RParen)) {
        p.bump(SyntaxKind::RPAREN, spans);
    } else {
        return Err(expected_got(Some(&close), "')' to close tuple pattern"));
    }
    p.finish_node(); // TUPLE_PATTERN
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TokenCursor;
    use lexer::ElpLexer;
    use pretty_assertions::assert_eq;

    fn parse_block_src(src: &str) -> Block {
        let mut lexer = ElpLexer::new(src);
        let tokens = lexer.scan();
        let mut cursor = TokenCursor::new(&tokens);
        let mut spans = vec![];
        let mut p = TreeParser::new(&mut cursor);
        parse_block(&mut p, &mut spans).unwrap_or_else(|e| panic!("{src:?}: {e}"));
        Block::cast(p.finish()).unwrap()
    }

    fn parse_block_err(src: &str) -> ElpParserError {
        let mut lexer = ElpLexer::new(src);
        let tokens = lexer.scan();
        let mut cursor = TokenCursor::new(&tokens);
        let mut spans = vec![];
        let mut p = TreeParser::new(&mut cursor);
        parse_block(&mut p, &mut spans).unwrap_err()
    }

    #[test]
    fn empty_block() {
        let block = parse_block_src("{}");
        assert_eq!(block.statements().len(), 0);
    }

    #[test]
    fn var_and_const_statements() {
        let block = parse_block_src("{ var x = 1\nconst y = 2 }");
        let stmts = block.statements();
        assert_eq!(stmts.len(), 2);
        assert!(matches!(stmts[0], Statement::Var(_)));
        assert!(matches!(stmts[1], Statement::Const(_)));
    }

    #[test]
    fn exported_var_and_const_statements_are_props() {
        // `export` in statement position is the component-prop marker. This
        // had no dispatch arm at all before — `export var email` fell
        // through to the expression parser, so the prop syntax the corpus
        // and memory-model.md both use didn't parse anywhere.
        let block = parse_block_src("{ export var email string?\nexport const id int }");
        let stmts = block.statements();
        assert_eq!(stmts.len(), 2);
        let Statement::Var(v) = &stmts[0] else {
            panic!("expected an exported var, got {:?}", stmts[0])
        };
        assert!(v.is_exported());
        assert_eq!(v.pattern(), vec!["email".to_string()]);
        let Statement::Const(c) = &stmts[1] else {
            panic!("expected an exported const, got {:?}", stmts[1])
        };
        assert!(c.is_exported());
    }

    #[test]
    fn exported_callback_prop() {
        // The real line from `examples/ChatRoom/components/login-form.velp`.
        let block =
            parse_block_src("{ export *fn onSubmit(username string, password Protected) -> None }");
        let stmts = block.statements();
        assert_eq!(stmts.len(), 1);
        let Statement::Callback(c) = &stmts[0] else {
            panic!("expected a callback, got {:?}", stmts[0])
        };
        assert_eq!(c.name(), "onSubmit");
        assert_eq!(c.params().len(), 2);
        assert!(c.return_type().is_some());
    }

    #[test]
    fn callback_with_no_params_or_return_type() {
        let block = parse_block_src("{ export *fn onTick }");
        let stmts = block.statements();
        assert_eq!(stmts.len(), 1);
        let Statement::Callback(c) = &stmts[0] else {
            panic!("expected a callback, got {:?}", stmts[0])
        };
        assert_eq!(c.name(), "onTick");
        assert!(c.params().is_empty());
        assert!(c.return_type().is_none());
    }

    #[test]
    fn error_callback_must_be_exported() {
        // A private callback slot can only ever be nil — nothing outside
        // can fill it — so it's rejected rather than parsed into a node
        // that provably holds nothing.
        let err = parse_block_err("{ *fn onTick() }");
        assert!(
            format!("{err}").contains("every callback is a prop"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn error_export_in_a_block_must_be_followed_by_a_prop_form() {
        let err = parse_block_err("{ export foo() }");
        assert!(
            format!("{err}").contains("only props can be exported"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn error_star_must_be_followed_by_fn() {
        // `export` is present so this gets past the must-be-exported check
        // and actually reaches the `*`-then-`fn` check.
        let err = parse_block_err("{ export *notFn() }");
        assert!(
            format!("{err}").contains("'fn' after '*'"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn bare_call_is_an_expr_statement() {
        let block = parse_block_src("{ println(\"hi\") }");
        let stmts = block.statements();
        assert_eq!(stmts.len(), 1);
        assert!(matches!(stmts[0], Statement::Expr(Expr::Call(_))));
    }

    #[test]
    fn return_with_and_without_value() {
        let block = parse_block_src("{ return 1\nreturn }");
        let stmts = block.statements();
        assert_eq!(stmts.len(), 2);
        let Statement::Return(r1) = &stmts[0] else {
            panic!("expected return")
        };
        assert!(matches!(r1.value(), Some(Expr::Literal(_))));
        let Statement::Return(r2) = &stmts[1] else {
            panic!("expected return")
        };
        assert_eq!(r2.value(), None);
    }

    #[test]
    fn assignment_statement() {
        let block = parse_block_src("{ attempts += 1 }");
        let stmts = block.statements();
        let Statement::Assignment(a) = &stmts[0] else {
            panic!("expected assignment, got {:?}", stmts[0])
        };
        assert_eq!(a.op_text(), "+=");
        assert!(matches!(a.target(), Some(Expr::Ident(_))));
        assert!(matches!(a.value(), Some(Expr::Literal(_))));
    }

    #[test]
    fn member_assignment() {
        let block = parse_block_src("{ self.count = 0 }");
        let stmts = block.statements();
        let Statement::Assignment(a) = &stmts[0] else {
            panic!("expected assignment")
        };
        assert!(matches!(a.target(), Some(Expr::Member(_))));
    }

    #[test]
    fn for_statement() {
        let block = parse_block_src("{ for message in state.messages { render(message) } }");
        let stmts = block.statements();
        let Statement::For(f) = &stmts[0] else {
            panic!("expected for, got {:?}", stmts[0])
        };
        assert_eq!(f.binding(), "message");
        assert!(matches!(f.iterable(), Some(Expr::Member(_))));
        assert_eq!(f.body().unwrap().statements().len(), 1);
    }

    #[test]
    fn if_else_as_statement() {
        let block = parse_block_src("{ if ready { go() } else { wait() } }");
        let stmts = block.statements();
        let Statement::If(i) = &stmts[0] else {
            panic!("expected if, got {:?}", stmts[0])
        };
        assert!(matches!(i.condition(), Some(Expr::Ident(_))));
        assert_eq!(i.then_block().unwrap().statements().len(), 1);
        assert_eq!(i.else_block().unwrap().statements().len(), 1);
        assert!(i.else_if().is_none());
    }

    #[test]
    fn if_else_if_chain() {
        let block = parse_block_src("{ if a { 1 } else if b { 2 } else { 3 } }");
        let stmts = block.statements();
        let Statement::If(outer) = &stmts[0] else {
            panic!("expected if")
        };
        let else_if = outer.else_if().expect("expected else-if");
        assert_eq!(else_if.then_block().unwrap().statements().len(), 1);
        assert!(else_if.else_block().is_some());
    }

    #[test]
    fn if_condition_does_not_swallow_the_body() {
        // The `if`-condition-vs-builder-block ambiguity that
        // `grammar-script-notes.md` tracks as open: without the
        // struct-literal restriction on condition parsing, `if x { go() }`
        // would try to read `x { go() }` as a builder-block-wrapped
        // condition, leaving nothing for the if's own body.
        let block = parse_block_src("{ if x { go() } }");
        let stmts = block.statements();
        let Statement::If(i) = &stmts[0] else {
            panic!("expected if, got {:?}", stmts[0])
        };
        assert!(matches!(i.condition(), Some(Expr::Ident(_))));
        let then_stmts = i.then_block().unwrap().statements();
        assert_eq!(then_stmts.len(), 1);
        assert!(matches!(then_stmts[0], Statement::Expr(Expr::Call(_))));
    }

    #[test]
    fn match_with_literal_and_wildcard_patterns() {
        let block = parse_block_src(
            "{ match n {
    0 -> zero()
    _ -> other()
} }",
        );
        let stmts = block.statements();
        let Statement::Match(m) = &stmts[0] else {
            panic!(
                "expected a bare match expression statement, got {:?}",
                stmts[0]
            )
        };
        assert!(matches!(m.scrutinee(), Some(Expr::Ident(_))));
        let arms = m.arms();
        assert_eq!(arms.len(), 2);
        assert_eq!(arms[0].pattern_kind(), Some(SyntaxKind::LITERAL_EXPR));
        assert_eq!(arms[1].pattern_kind(), Some(SyntaxKind::WILDCARD_PATTERN));
    }

    #[test]
    fn match_with_range_and_catchall_patterns() {
        // Lifted from api.elp's ApiError.from body.
        let block = parse_block_src(
            "{ match err.responseCode {
    400..499 -> .USER(x)
    500..599 -> .SERVER(x)
    .. -> .UNKNOWN
} }",
        );
        let stmts = block.statements();
        let Statement::Match(m) = &stmts[0] else {
            panic!("expected match")
        };
        let arms = m.arms();
        assert_eq!(arms.len(), 3);
        assert_eq!(arms[0].pattern_kind(), Some(SyntaxKind::RANGE_PATTERN));
        assert_eq!(
            arms[2].pattern_kind(),
            Some(SyntaxKind::RANGE_CATCHALL_PATTERN)
        );
        // `.USER(x)` is a call whose callee is the contextual variant —
        // the call wrapping happens in the postfix loop, same as any
        // other `expr(...)`.
        assert!(matches!(
            arms[0].body(),
            Some(Expr::Call(call)) if matches!(call.callee(), Some(Expr::ContextualVariant(_)))
        ));
    }

    #[test]
    fn match_with_variant_and_tuple_patterns() {
        let block = parse_block_src(
            "{ match pair {
    .SERVER(msg) -> handle(msg)
    Ok(value) -> value
    (a, b) -> combine(a, b)
    name -> name
} }",
        );
        let stmts = block.statements();
        let Statement::Match(m) = &stmts[0] else {
            panic!("expected match")
        };
        let arms = m.arms();
        assert_eq!(arms.len(), 4);
        assert_eq!(arms[0].pattern_kind(), Some(SyntaxKind::VARIANT_PATTERN)); // .SERVER(msg)
        assert_eq!(arms[1].pattern_kind(), Some(SyntaxKind::VARIANT_PATTERN)); // Ok(value)
        assert_eq!(arms[2].pattern_kind(), Some(SyntaxKind::TUPLE_PATTERN));
        assert_eq!(arms[3].pattern_kind(), Some(SyntaxKind::BINDING_PATTERN));
    }

    #[test]
    fn match_arm_with_block_body() {
        let block = parse_block_src(
            "{ match x {
    0 -> {
        setup()
        return done()
    }
} }",
        );
        let stmts = block.statements();
        let Statement::Match(m) = &stmts[0] else {
            panic!("expected match")
        };
        let body_block = m.arms()[0].body_block().expect("expected a block arm body");
        assert_eq!(body_block.statements().len(), 2);
    }

    #[test]
    fn match_condition_restriction_does_not_swallow_arms() {
        // Same ambiguity as the if-condition case, for match's scrutinee.
        let block = parse_block_src("{ match x { 0 -> a } }");
        let stmts = block.statements();
        assert!(matches!(&stmts[0], Statement::Match(_)));
    }

    #[test]
    fn lambda_expression() {
        let block = parse_block_src("{ const f = fn(x int) -> int { return x } }");
        let Statement::Const(c) = &block.statements()[0] else {
            panic!("expected const")
        };
        let Some(Expr::Lambda(lambda)) = c.value() else {
            panic!("expected lambda value")
        };
        assert_eq!(lambda.params().len(), 1);
        assert!(lambda.return_type().is_some());
        assert_eq!(lambda.body().unwrap().statements().len(), 1);
    }

    #[test]
    fn builder_block_after_call() {
        // `App(...) { Surface { Content() } }` — the exact shape from
        // examples/BlogStyle/main.velp, minus macro/view-dialect syntax.
        let block = parse_block_src(
            "{ App(globalStyles = Styles()) {
    Surface {
        Content()
    }
} }",
        );
        let Statement::Expr(Expr::BuilderBlock(outer)) = &block.statements()[0] else {
            panic!("expected a builder block, got {:?}", block.statements()[0])
        };
        assert!(matches!(outer.receiver(), Some(Expr::Call(_))));
        let outer_stmts = outer.body().unwrap().statements();
        assert_eq!(outer_stmts.len(), 1);
        // `Surface { Content() }` — a bare identifier receiver, which is
        // disambiguated from an object literal in `parse_primary` itself
        // (not the postfix loop) since `Content()` doesn't look like a
        // field-init.
        let Statement::Expr(Expr::BuilderBlock(inner)) = &outer_stmts[0] else {
            panic!("expected nested builder block, got {:?}", outer_stmts[0])
        };
        assert!(matches!(inner.receiver(), Some(Expr::Ident(_))));
        assert_eq!(inner.body().unwrap().statements().len(), 1);
    }

    #[test]
    fn error_unclosed_block() {
        let err = parse_block_err("{ var x = 1");
        assert!(format!("{err}").contains("'}'"), "unexpected error: {err}");
    }

    #[test]
    fn error_bad_pattern() {
        let err = parse_block_err("{ match x { ) -> 1 } }");
        assert!(
            format!("{err}").contains("pattern"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn error_for_missing_loop_variable() {
        // Not `for in xs { }` — `in` lexes as a plain identifier (it's
        // contextual, not reserved), so that would bind `in` itself as
        // the loop variable and fail later, on the *next* `in` check,
        // with a different message. A token that can't be an identifier
        // at all is what actually exercises "a loop variable".
        let err = parse_block_err("{ for 5 in xs { } }");
        assert!(
            format!("{err}").contains("loop variable"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn error_for_missing_in() {
        let err = parse_block_err("{ for x xs { } }");
        assert!(format!("{err}").contains("'in'"), "unexpected error: {err}");
    }

    #[test]
    fn error_for_missing_body() {
        let err = parse_block_err("{ for x in xs }");
        assert!(
            format!("{err}").contains("loop body"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn error_if_missing_body() {
        let err = parse_block_err("{ if x }");
        assert!(
            format!("{err}").contains("if body"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn error_if_else_missing_body_or_if() {
        let err = parse_block_err("{ if x { 1 } else }");
        assert!(
            format!("{err}").contains("after else"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn error_match_missing_body() {
        let err = parse_block_err("{ match x }");
        assert!(
            format!("{err}").contains("match body"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn error_match_unclosed_arms() {
        // No closing brace *at all* — if the match had its own closing
        // `}` but the outer block didn't, the outer block's "'}' to close
        // block" would fire first instead of match's own arms-loop check.
        let err = parse_block_err("{ match x { 0 -> 1");
        assert!(
            format!("{err}").contains("match arm"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn error_match_arm_missing_arrow() {
        let err = parse_block_err("{ match x { 0 1 } }");
        assert!(format!("{err}").contains("'->'"), "unexpected error: {err}");
    }

    #[test]
    fn error_variant_pattern_unclosed_payload() {
        let err = parse_block_err("{ match x { .SERVER(msg -> 1 } }");
        assert!(format!("{err}").contains("')'"), "unexpected error: {err}");
    }

    #[test]
    fn error_tuple_pattern_unclosed() {
        let err = parse_block_err("{ match x { (a, b -> 1 } }");
        assert!(format!("{err}").contains("')'"), "unexpected error: {err}");
    }
}
