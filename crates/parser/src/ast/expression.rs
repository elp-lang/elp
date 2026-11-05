use cst::{AstNode, SyntaxKind, SyntaxNode};
use lexer::{
    span::{CodeSpan, SpannedToken},
    tokens::{LexerSymbol, LexerTokens},
};

use crate::{
    Spanned, TokenCursor, TreeParser,
    ast::{
        declaration, statement,
        ty::{self, Type},
    },
    error::{ElpParserError, expected_got, unexpected},
    span::spans_extent,
};

// ============================================================================
// Scope of this pass:
//
//   Covered: literals, identifiers, contextual `.Variant`, parenthesized
//   exprs, unary `! - &`, the full binary precedence table, postfix member
//   access / call / index, array/tuple/object literals, lambdaExpr,
//   matchExpr/ifExpr (in both statement and expression position, via
//   statement.rs), and the trailing builderBlock after a call.
//
//   NOT covered yet: generic args in calls/types (`Http.post<T, E>(...)`)
//   and dotted/generic type names in object literals (`Foo.Bar { ... }`) —
//   both need generics support in the `type` grammar rule, which nothing
//   anywhere has implemented yet.
// ============================================================================

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

ast_node!(Literal, SyntaxKind::LITERAL_EXPR);
ast_node!(IdentExpr, SyntaxKind::IDENT_EXPR);
ast_node!(ContextualVariantExpr, SyntaxKind::CONTEXTUAL_VARIANT_EXPR);
ast_node!(ParenExpr, SyntaxKind::PAREN_EXPR);
ast_node!(UnaryExpr, SyntaxKind::UNARY_EXPR);
ast_node!(BinaryExpr, SyntaxKind::BINARY_EXPR);
ast_node!(MemberExpr, SyntaxKind::MEMBER_EXPR);
ast_node!(CallExpr, SyntaxKind::CALL_EXPR);
ast_node!(IndexExpr, SyntaxKind::INDEX_EXPR);
ast_node!(ArrayLiteralExpr, SyntaxKind::ARRAY_LITERAL_EXPR);
ast_node!(TupleLiteralExpr, SyntaxKind::TUPLE_LITERAL_EXPR);
ast_node!(ObjectLiteralExpr, SyntaxKind::OBJECT_LITERAL_EXPR);
ast_node!(FieldInit, SyntaxKind::FIELD_INIT);
ast_node!(Arg, SyntaxKind::ARG);
ast_node!(LambdaExpr, SyntaxKind::LAMBDA_EXPR);
ast_node!(IfExpr, SyntaxKind::IF_EXPR);
ast_node!(MatchExpr, SyntaxKind::MATCH_EXPR);
ast_node!(BuilderBlockExpr, SyntaxKind::BUILDER_BLOCK);

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum LiteralKind {
    Integer,
    Float,
    String,
    True,
    False,
    Nil,
}

impl Literal {
    pub fn kind(&self) -> Option<LiteralKind> {
        self.0.first_token().and_then(|t| {
            Some(match t.kind() {
                SyntaxKind::INTEGER => LiteralKind::Integer,
                SyntaxKind::FLOAT => LiteralKind::Float,
                SyntaxKind::STRING => LiteralKind::String,
                SyntaxKind::TRUE_KW => LiteralKind::True,
                SyntaxKind::FALSE_KW => LiteralKind::False,
                SyntaxKind::NIL_KW => LiteralKind::Nil,
                _ => return None,
            })
        })
    }

    pub fn text(&self) -> String {
        self.0
            .first_token()
            .map(|t| t.text().to_string())
            .unwrap_or_default()
    }
}

impl IdentExpr {
    pub fn name(&self) -> String {
        self.0
            .first_token()
            .map(|t| t.text().to_string())
            .unwrap_or_default()
    }
}

impl ContextualVariantExpr {
    pub fn name(&self) -> String {
        self.0
            .children_with_tokens()
            .filter_map(|el| el.into_token())
            .find(|t| t.kind() == SyntaxKind::IDENT)
            .map(|t| t.text().to_string())
            .unwrap_or_default()
    }
}

impl ParenExpr {
    pub fn inner(&self) -> Option<Expr> {
        self.0.children().find_map(Expr::cast)
    }
}

impl UnaryExpr {
    pub fn op_text(&self) -> String {
        self.0
            .children_with_tokens()
            .filter_map(|el| el.into_token())
            .next()
            .map(|t| t.text().to_string())
            .unwrap_or_default()
    }

    pub fn operand(&self) -> Option<Expr> {
        self.0.children().find_map(Expr::cast)
    }
}

impl BinaryExpr {
    pub fn lhs(&self) -> Option<Expr> {
        self.0.children().filter_map(Expr::cast).next()
    }

    pub fn rhs(&self) -> Option<Expr> {
        self.0.children().filter_map(Expr::cast).nth(1)
    }

    pub fn op_text(&self) -> String {
        self.0
            .children_with_tokens()
            .filter_map(|el| el.into_token())
            .find(|t| binary_binding_power_for_kind(t.kind()).is_some())
            .map(|t| t.text().to_string())
            .unwrap_or_default()
    }
}

impl MemberExpr {
    pub fn receiver(&self) -> Option<Expr> {
        self.0.children().find_map(Expr::cast)
    }

    pub fn member_name(&self) -> String {
        // The member name is the *last* IDENT token direct child (the
        // first, if the receiver itself happens to be a plain identifier,
        // belongs to the receiver's own IDENT_EXPR node, not this one).
        self.0
            .children_with_tokens()
            .filter_map(|el| el.into_token())
            .filter(|t| t.kind() == SyntaxKind::IDENT)
            .last()
            .map(|t| t.text().to_string())
            .unwrap_or_default()
    }
}

impl CallExpr {
    pub fn callee(&self) -> Option<Expr> {
        self.0.children().find_map(Expr::cast)
    }

    pub fn args(&self) -> Vec<Arg> {
        self.0
            .children()
            .find(|n| n.kind() == SyntaxKind::ARG_LIST)
            .map(|list| list.children().filter_map(Arg::cast).collect())
            .unwrap_or_default()
    }
}

impl IndexExpr {
    pub fn receiver(&self) -> Option<Expr> {
        self.0.children().filter_map(Expr::cast).next()
    }

    pub fn index(&self) -> Option<Expr> {
        self.0.children().filter_map(Expr::cast).nth(1)
    }
}

impl ArrayLiteralExpr {
    pub fn elements(&self) -> Vec<Expr> {
        self.0.children().filter_map(Expr::cast).collect()
    }
}

impl TupleLiteralExpr {
    pub fn elements(&self) -> Vec<Expr> {
        self.0.children().filter_map(Expr::cast).collect()
    }
}

impl ObjectLiteralExpr {
    /// `None` for the bare `{ field = expr }` form with no leading type name.
    pub fn type_name(&self) -> Option<String> {
        self.0
            .children_with_tokens()
            .filter_map(|el| el.into_token())
            .find(|t| t.kind() == SyntaxKind::IDENT)
            .map(|t| t.text().to_string())
    }

    pub fn fields(&self) -> Vec<FieldInit> {
        self.0.children().filter_map(FieldInit::cast).collect()
    }
}

impl FieldInit {
    pub fn name(&self) -> String {
        self.0
            .children_with_tokens()
            .filter_map(|el| el.into_token())
            .find(|t| t.kind() == SyntaxKind::IDENT)
            .map(|t| t.text().to_string())
            .unwrap_or_default()
    }

    /// `None` for the shorthand `IDENT` form (value inferred from a
    /// same-named binding — a semantic-pass concern, not this layer's).
    pub fn value(&self) -> Option<Expr> {
        self.0.children().find_map(Expr::cast)
    }
}

impl Arg {
    /// `Some` only if this arg has an `IDENT '='` prefix — distinguished
    /// from a bare identifier-expression argument by the parser checking
    /// for a following `=` before committing to the named-arg shape.
    pub fn name(&self) -> Option<String> {
        let mut tokens = self
            .0
            .children_with_tokens()
            .filter_map(|el| el.into_token());
        let first = tokens.next()?;
        let second = tokens.next()?;
        (first.kind() == SyntaxKind::IDENT && second.kind() == SyntaxKind::EQ)
            .then(|| first.text().to_string())
    }

    pub fn value(&self) -> Option<Expr> {
        self.0.children().find_map(Expr::cast)
    }
}

impl LambdaExpr {
    pub fn params(&self) -> Vec<declaration::Param> {
        self.0
            .children()
            .find(|n| n.kind() == SyntaxKind::PARAM_LIST)
            .map(|list| {
                list.children()
                    .filter_map(declaration::Param::cast)
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn return_type(&self) -> Option<Type> {
        self.0
            .children()
            .filter(|n| !matches!(n.kind(), SyntaxKind::PARAM_LIST | SyntaxKind::BLOCK))
            .find_map(Type::cast)
    }

    pub fn body(&self) -> Option<statement::Block> {
        self.0.children().find_map(statement::Block::cast)
    }
}

impl IfExpr {
    pub fn condition(&self) -> Option<Expr> {
        self.0.children().find_map(Expr::cast)
    }

    pub fn then_block(&self) -> Option<statement::Block> {
        self.0.children().find_map(statement::Block::cast)
    }

    /// `None` when there's no `else` at all, or when the else branch is
    /// itself an `else if` (see `else_if`, not this).
    pub fn else_block(&self) -> Option<statement::Block> {
        self.0.children().filter_map(statement::Block::cast).nth(1)
    }

    pub fn else_if(&self) -> Option<IfExpr> {
        self.0.children().find_map(IfExpr::cast)
    }
}

impl MatchExpr {
    pub fn scrutinee(&self) -> Option<Expr> {
        self.0.children().find_map(Expr::cast)
    }

    pub fn arms(&self) -> Vec<statement::MatchArm> {
        self.0
            .children()
            .find(|n| n.kind() == SyntaxKind::MATCH_ARMS)
            .map(|arms| {
                arms.children()
                    .filter_map(statement::MatchArm::cast)
                    .collect()
            })
            .unwrap_or_default()
    }
}

impl BuilderBlockExpr {
    pub fn receiver(&self) -> Option<Expr> {
        self.0.children().find_map(Expr::cast)
    }

    pub fn body(&self) -> Option<statement::Block> {
        self.0.children().find_map(statement::Block::cast)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Literal(Literal),
    Ident(IdentExpr),
    ContextualVariant(ContextualVariantExpr),
    Paren(ParenExpr),
    Unary(UnaryExpr),
    Binary(BinaryExpr),
    Member(MemberExpr),
    Call(CallExpr),
    Index(IndexExpr),
    ArrayLiteral(ArrayLiteralExpr),
    TupleLiteral(TupleLiteralExpr),
    ObjectLiteral(ObjectLiteralExpr),
    Lambda(LambdaExpr),
    If(IfExpr),
    Match(MatchExpr),
    BuilderBlock(BuilderBlockExpr),
}

impl AstNode for Expr {
    fn can_cast(kind: SyntaxKind) -> bool {
        matches!(
            kind,
            SyntaxKind::LITERAL_EXPR
                | SyntaxKind::IDENT_EXPR
                | SyntaxKind::CONTEXTUAL_VARIANT_EXPR
                | SyntaxKind::PAREN_EXPR
                | SyntaxKind::UNARY_EXPR
                | SyntaxKind::BINARY_EXPR
                | SyntaxKind::MEMBER_EXPR
                | SyntaxKind::CALL_EXPR
                | SyntaxKind::INDEX_EXPR
                | SyntaxKind::ARRAY_LITERAL_EXPR
                | SyntaxKind::TUPLE_LITERAL_EXPR
                | SyntaxKind::OBJECT_LITERAL_EXPR
                | SyntaxKind::LAMBDA_EXPR
                | SyntaxKind::IF_EXPR
                | SyntaxKind::MATCH_EXPR
                | SyntaxKind::BUILDER_BLOCK
        )
    }

    fn cast(node: SyntaxNode) -> Option<Self> {
        let kind = node.kind();
        if !Self::can_cast(kind) {
            return None;
        }
        Some(match kind {
            SyntaxKind::LITERAL_EXPR => Expr::Literal(Literal::cast(node)?),
            SyntaxKind::IDENT_EXPR => Expr::Ident(IdentExpr::cast(node)?),
            SyntaxKind::CONTEXTUAL_VARIANT_EXPR => {
                Expr::ContextualVariant(ContextualVariantExpr::cast(node)?)
            }
            SyntaxKind::PAREN_EXPR => Expr::Paren(ParenExpr::cast(node)?),
            SyntaxKind::UNARY_EXPR => Expr::Unary(UnaryExpr::cast(node)?),
            SyntaxKind::BINARY_EXPR => Expr::Binary(BinaryExpr::cast(node)?),
            SyntaxKind::MEMBER_EXPR => Expr::Member(MemberExpr::cast(node)?),
            SyntaxKind::CALL_EXPR => Expr::Call(CallExpr::cast(node)?),
            SyntaxKind::INDEX_EXPR => Expr::Index(IndexExpr::cast(node)?),
            SyntaxKind::ARRAY_LITERAL_EXPR => Expr::ArrayLiteral(ArrayLiteralExpr::cast(node)?),
            SyntaxKind::TUPLE_LITERAL_EXPR => Expr::TupleLiteral(TupleLiteralExpr::cast(node)?),
            SyntaxKind::OBJECT_LITERAL_EXPR => Expr::ObjectLiteral(ObjectLiteralExpr::cast(node)?),
            SyntaxKind::LAMBDA_EXPR => Expr::Lambda(LambdaExpr::cast(node)?),
            SyntaxKind::IF_EXPR => Expr::If(IfExpr::cast(node)?),
            SyntaxKind::MATCH_EXPR => Expr::Match(MatchExpr::cast(node)?),
            SyntaxKind::BUILDER_BLOCK => Expr::BuilderBlock(BuilderBlockExpr::cast(node)?),
            _ => unreachable!("can_cast already filtered to a known kind"),
        })
    }

    fn syntax(&self) -> &SyntaxNode {
        match self {
            Expr::Literal(n) => n.syntax(),
            Expr::Ident(n) => n.syntax(),
            Expr::ContextualVariant(n) => n.syntax(),
            Expr::Paren(n) => n.syntax(),
            Expr::Unary(n) => n.syntax(),
            Expr::Binary(n) => n.syntax(),
            Expr::Member(n) => n.syntax(),
            Expr::Call(n) => n.syntax(),
            Expr::Index(n) => n.syntax(),
            Expr::Lambda(n) => n.syntax(),
            Expr::If(n) => n.syntax(),
            Expr::Match(n) => n.syntax(),
            Expr::BuilderBlock(n) => n.syntax(),
            Expr::ArrayLiteral(n) => n.syntax(),
            Expr::TupleLiteral(n) => n.syntax(),
            Expr::ObjectLiteral(n) => n.syntax(),
        }
    }
}

impl Expr {
    pub fn from_cursor(cursor: &mut TokenCursor) -> Result<Spanned<Self>, ElpParserError> {
        let mut spans = vec![];
        let mut p = TreeParser::new(cursor);
        parse_binary(&mut p, &mut spans, 0)?;
        let root = p.finish();
        let expr =
            Expr::cast(root).expect("parse_binary always builds exactly one expression-kind root");
        Ok(Spanned {
            node: expr,
            span: spans_extent(spans)?,
        })
    }
}

fn binary_binding_power(sym: &LexerSymbol) -> Option<(u8, SyntaxKind)> {
    use LexerSymbol::*;
    Some(match sym {
        Multiply => (5, SyntaxKind::STAR),
        Divide => (5, SyntaxKind::SLASH),
        Percent => (5, SyntaxKind::PERCENT),
        Plus => (4, SyntaxKind::PLUS),
        Minus => (4, SyntaxKind::MINUS),
        LessThan => (3, SyntaxKind::LT),
        GreaterThan => (3, SyntaxKind::GT),
        LessEqual => (3, SyntaxKind::LEQ),
        GreaterEqual => (3, SyntaxKind::GEQ),
        EqualEqual => (2, SyntaxKind::EQEQ),
        NotEqual => (2, SyntaxKind::NEQ),
        AndAnd => (1, SyntaxKind::ANDAND),
        OrOr => (0, SyntaxKind::OROR),
        _ => return None,
    })
}

fn binary_binding_power_for_kind(kind: SyntaxKind) -> Option<u8> {
    Some(match kind {
        SyntaxKind::STAR | SyntaxKind::SLASH | SyntaxKind::PERCENT => 5,
        SyntaxKind::PLUS | SyntaxKind::MINUS => 4,
        SyntaxKind::LT | SyntaxKind::GT | SyntaxKind::LEQ | SyntaxKind::GEQ => 3,
        SyntaxKind::EQEQ | SyntaxKind::NEQ => 2,
        SyntaxKind::ANDAND => 1,
        SyntaxKind::OROR => 0,
        _ => return None,
    })
}

// Pratt/precedence-climbing binary-expression parser. Uses a checkpoint
// (see TreeParser::checkpoint docs) to retroactively wrap the LHS — already
// parsed without knowing whether it'd turn out to need wrapping — in a
// BINARY_EXPR once an operator at or above `min_bp` actually shows up.
pub(crate) fn parse_binary(
    p: &mut TreeParser,
    spans: &mut Vec<CodeSpan>,
    min_bp: u8,
) -> Result<(), ElpParserError> {
    parse_binary_ctx(p, spans, min_bp, true)
}

/// The if-condition ambiguity `grammar-script-notes.md` tracks as an open
/// question at the language level, resolved here at the parser level:
/// `if x { ... }` is ambiguous between `x` alone as the condition (with
/// `{ ... }` as the if-body) and `x { ... }` as an object literal *or*
/// builder-block-wrapped condition (swallowing the if-body entirely).
/// Resolved the same way Rust/Go/Swift resolve it: object/builder-block
/// literals are disallowed while parsing an `if`/`match`/`for` condition,
/// and allowed again as soon as we're inside an unambiguous bracketed
/// sub-context (call args, array/tuple elements, parens, index brackets —
/// all of which call the regular `parse_binary` directly, not this one, so
/// the restriction doesn't propagate into them).
pub(crate) fn parse_binary_no_struct_literal(
    p: &mut TreeParser,
    spans: &mut Vec<CodeSpan>,
    min_bp: u8,
) -> Result<(), ElpParserError> {
    parse_binary_ctx(p, spans, min_bp, false)
}

fn parse_binary_ctx(
    p: &mut TreeParser,
    spans: &mut Vec<CodeSpan>,
    min_bp: u8,
    allow_struct_literal: bool,
) -> Result<(), ElpParserError> {
    let checkpoint = p.checkpoint();
    parse_unary(p, spans, allow_struct_literal)?;

    while let Some(LexerTokens::Symbol(sym)) = p.peek().map(|t| t.token.clone()) {
        let Some((bp, kind)) = binary_binding_power(&sym) else {
            break;
        };
        if bp < min_bp {
            break;
        }

        p.start_node_at(checkpoint, SyntaxKind::BINARY_EXPR);
        p.bump(kind, spans);
        parse_binary_ctx(p, spans, bp + 1, allow_struct_literal)?; // +1: left-associative
        p.finish_node(); // BINARY_EXPR
    }
    Ok(())
}

fn unary_op_kind(sym: &LexerSymbol) -> Option<SyntaxKind> {
    match sym {
        LexerSymbol::Bang => Some(SyntaxKind::BANG),
        LexerSymbol::Minus => Some(SyntaxKind::MINUS),
        LexerSymbol::Ampersand => Some(SyntaxKind::AMP),
        // Deref: `*x` copies the referent out from behind a reference
        // (docs/memory-model.md Finding 6). Same token kind `*` already
        // uses as binary Multiply — grammatical position, not token
        // content, disambiguates them, same as Rust/C.
        LexerSymbol::Multiply => Some(SyntaxKind::STAR),
        _ => None,
    }
}

fn parse_unary(
    p: &mut TreeParser,
    spans: &mut Vec<CodeSpan>,
    allow_struct_literal: bool,
) -> Result<(), ElpParserError> {
    if let Some(LexerTokens::Symbol(sym)) = p.peek().map(|t| t.token.clone())
        && let Some(kind) = unary_op_kind(&sym)
    {
        p.start_node(SyntaxKind::UNARY_EXPR);
        p.bump(kind, spans);
        parse_unary(p, spans, allow_struct_literal)?; // right-recursive: `!!x`, `--x`
        p.finish_node();
        return Ok(());
    }
    parse_postfix(p, spans, allow_struct_literal)
}

fn parse_postfix(
    p: &mut TreeParser,
    spans: &mut Vec<CodeSpan>,
    allow_struct_literal: bool,
) -> Result<(), ElpParserError> {
    let checkpoint = p.checkpoint();
    parse_primary(p, spans, allow_struct_literal)?;

    loop {
        match p.peek().map(|t| t.token.clone()) {
            Some(LexerTokens::Symbol(LexerSymbol::Period)) => {
                p.start_node_at(checkpoint, SyntaxKind::MEMBER_EXPR);
                p.bump(SyntaxKind::DOT, spans);
                let name_tok = p
                    .peek()
                    .cloned()
                    .ok_or_else(|| expected_got(None, "member name after '.'"))?;
                if matches!(name_tok.token, LexerTokens::Identifier(_)) {
                    p.bump(SyntaxKind::IDENT, spans);
                } else {
                    return Err(expected_got(Some(&name_tok), "member name after '.'"));
                }
                p.finish_node(); // MEMBER_EXPR
            }
            Some(LexerTokens::Symbol(LexerSymbol::LParen)) => {
                p.start_node_at(checkpoint, SyntaxKind::CALL_EXPR);
                parse_arg_list(p, spans)?;
                p.finish_node(); // CALL_EXPR
            }
            Some(LexerTokens::Symbol(LexerSymbol::LBracket)) => {
                p.start_node_at(checkpoint, SyntaxKind::INDEX_EXPR);
                p.bump(SyntaxKind::LBRACK, spans);
                parse_binary(p, spans, 0)?;
                let close = p.peek().cloned().ok_or_else(|| expected_got(None, "']'"))?;
                if matches!(close.token, LexerTokens::Symbol(LexerSymbol::RBracket)) {
                    p.bump(SyntaxKind::RBRACK, spans);
                } else {
                    return Err(expected_got(Some(&close), "']'"));
                }
                p.finish_node(); // INDEX_EXPR
            }
            // `App(...) { Surface { ... } }` — a block trailing any
            // primary/postfix expression. Only reachable here for
            // *non*-bare-identifier receivers (a call, index, member
            // access, ...); the bare-identifier case (`Surface { ... }`)
            // is resolved earlier, in `parse_primary`, since there it's
            // ambiguous with an object literal (see that function's docs).
            // Suppressed when `allow_struct_literal` is false (if/match/for
            // condition position) — see `parse_binary_no_struct_literal`.
            Some(LexerTokens::Symbol(LexerSymbol::LCurley)) if allow_struct_literal => {
                p.start_node_at(checkpoint, SyntaxKind::BUILDER_BLOCK);
                statement::parse_block(p, spans)?;
                p.finish_node(); // BUILDER_BLOCK
            }
            _ => break,
        }
    }
    Ok(())
}

fn parse_arg_list(p: &mut TreeParser, spans: &mut Vec<CodeSpan>) -> Result<(), ElpParserError> {
    p.bump(SyntaxKind::LPAREN, spans);
    p.start_node(SyntaxKind::ARG_LIST);

    let mut expect_arg = true;
    loop {
        let Some(tok) = p.peek().cloned() else {
            return Err(expected_got(None, "')' to close call"));
        };
        match &tok.token {
            LexerTokens::Symbol(LexerSymbol::RParen) => break,
            LexerTokens::Symbol(LexerSymbol::Comma) => {
                if expect_arg {
                    return Err(unexpected(&tok, "unexpected ','"));
                }
                p.bump(SyntaxKind::COMMA, spans);
                expect_arg = true;
            }
            LexerTokens::Newline => p.skip_trivia(spans),
            _ => {
                if !expect_arg {
                    return Err(unexpected(&tok, "expected ',' or ')'"));
                }
                parse_arg(p, spans)?;
                expect_arg = false;
            }
        }
    }
    p.finish_node(); // ARG_LIST
    p.bump(SyntaxKind::RPAREN, spans);
    Ok(())
}

fn parse_arg(p: &mut TreeParser, spans: &mut Vec<CodeSpan>) -> Result<(), ElpParserError> {
    p.start_node(SyntaxKind::ARG);

    // `IDENT '='` named-arg prefix needs 2-token lookahead: a bare IDENT is
    // equally valid as the start of an ordinary expression argument.
    let is_named = matches!(p.peek().map(|t| &t.token), Some(LexerTokens::Identifier(_)))
        && matches!(
            p.peek_at(2).map(|t| &t.token),
            Some(LexerTokens::Symbol(LexerSymbol::Equal))
        );
    if is_named {
        p.bump(SyntaxKind::IDENT, spans);
        p.bump(SyntaxKind::EQ, spans);
    }

    parse_binary(p, spans, 0)?;
    p.finish_node(); // ARG
    Ok(())
}

fn parse_primary(
    p: &mut TreeParser,
    spans: &mut Vec<CodeSpan>,
    allow_struct_literal: bool,
) -> Result<(), ElpParserError> {
    let Some(tok) = p.peek().cloned() else {
        return Err(expected_got(None, "an expression"));
    };

    match &tok.token {
        LexerTokens::Number(text) => {
            let kind = if text.contains('.') {
                SyntaxKind::FLOAT
            } else {
                SyntaxKind::INTEGER
            };
            p.start_node(SyntaxKind::LITERAL_EXPR);
            p.bump(kind, spans);
            p.finish_node();
        }
        LexerTokens::String(_) => {
            p.start_node(SyntaxKind::LITERAL_EXPR);
            p.bump(SyntaxKind::STRING, spans);
            p.finish_node();
        }
        LexerTokens::Identifier(name) => match name.as_str() {
            "true" => {
                p.start_node(SyntaxKind::LITERAL_EXPR);
                p.bump(SyntaxKind::TRUE_KW, spans);
                p.finish_node();
            }
            "false" => {
                p.start_node(SyntaxKind::LITERAL_EXPR);
                p.bump(SyntaxKind::FALSE_KW, spans);
                p.finish_node();
            }
            "nil" => {
                p.start_node(SyntaxKind::LITERAL_EXPR);
                p.bump(SyntaxKind::NIL_KW, spans);
                p.finish_node();
            }
            "self" => {
                p.start_node(SyntaxKind::IDENT_EXPR);
                p.bump(SyntaxKind::SELF_KW, spans);
                p.finish_node();
            }
            "fn" => {
                p.start_node(SyntaxKind::LAMBDA_EXPR);
                p.bump(SyntaxKind::FN_KW, spans);
                if matches!(
                    p.peek().map(|t| &t.token),
                    Some(LexerTokens::Symbol(LexerSymbol::LParen))
                ) {
                    declaration::parse_param_list(p, spans)?;
                }
                if matches!(
                    p.peek().map(|t| &t.token),
                    Some(LexerTokens::Symbol(LexerSymbol::Arrow))
                ) {
                    p.bump(SyntaxKind::ARROW, spans);
                    ty::parse_type(p, spans)?;
                }
                statement::parse_block(p, spans)?;
                p.finish_node(); // LAMBDA_EXPR
            }
            "if" => statement::parse_if_expr(p, spans)?,
            "match" => statement::parse_match_expr(p, spans)?,
            _ => {
                // `Name { field = expr }` object literal vs. a bare
                // identifier reference (possibly followed by a
                // `BUILDER_BLOCK` in the postfix loop instead, e.g.
                // `Surface { Content() }`) — see `looks_like_object_literal_body`'s
                // docs for how those two are told apart, and this
                // function's `allow_struct_literal` param for why both are
                // suppressed in if/match/for condition position.
                let starts_object_literal = allow_struct_literal
                    && matches!(
                        p.peek_at(2).map(|t| &t.token),
                        Some(LexerTokens::Symbol(LexerSymbol::LCurley))
                    )
                    && looks_like_object_literal_body(p);
                if starts_object_literal {
                    p.start_node(SyntaxKind::OBJECT_LITERAL_EXPR);
                    p.bump(SyntaxKind::IDENT, spans);
                    parse_object_literal_body(p, spans)?;
                    p.finish_node();
                } else {
                    p.start_node(SyntaxKind::IDENT_EXPR);
                    p.bump(SyntaxKind::IDENT, spans);
                    p.finish_node();
                }
            }
        },
        LexerTokens::Symbol(LexerSymbol::Period) => {
            p.start_node(SyntaxKind::CONTEXTUAL_VARIANT_EXPR);
            p.bump(SyntaxKind::DOT, spans);
            let name_tok = p
                .peek()
                .cloned()
                .ok_or_else(|| expected_got(None, "identifier after '.'"))?;
            if matches!(name_tok.token, LexerTokens::Identifier(_)) {
                p.bump(SyntaxKind::IDENT, spans);
            } else {
                return Err(expected_got(Some(&name_tok), "identifier after '.'"));
            }
            p.finish_node();
        }
        LexerTokens::Symbol(LexerSymbol::LParen) => parse_paren_or_tuple(p, spans)?,
        LexerTokens::Symbol(LexerSymbol::LBracket) => parse_array_literal(p, spans)?,
        LexerTokens::Symbol(LexerSymbol::LCurley) if allow_struct_literal => {
            p.start_node(SyntaxKind::OBJECT_LITERAL_EXPR);
            parse_object_literal_body(p, spans)?;
            p.finish_node();
        }
        _ => return Err(unexpected(&tok, "expected an expression")),
    }
    Ok(())
}

/// Called when `peek()`=IDENT, `peek_at(2)`='{'. Peeks further to see
/// whether the body looks like `fieldInit (',' fieldInit)*` (comma-, `=`-,
/// or `}`-shaped) rather than an arbitrary statement — the signal that
/// distinguishes an object literal (`Foo { x = 1 }`) from a builder block
/// on a bare-identifier receiver (`Surface { Content() }`, which reads as
/// `IDENT_EXPR("Surface")` followed by a `BUILDER_BLOCK` in the postfix
/// loop, once this returns false here). Heuristic, not a formal
/// disambiguation — `object literal` and `builder block` are the same
/// `{ ... }` shape, and the language has no rule that separates them, so
/// this is a lookahead guess standing in for a decision nobody has made
/// (tracked in `grammar-script-notes.md`). Only looks at the
/// *first* entry, so a builder block whose first statement happens to look
/// like a field-init (`Foo { x = 1; someCall() }`) would misfire; real
/// example code doesn't do this.
fn looks_like_object_literal_body(p: &TreeParser) -> bool {
    // Real object literals routinely put a newline right after `{` (see
    // `object_decl_fields_newline_separated_no_commas`'s style, which
    // applies here too) — `peek_at` alone would count that newline as
    // "the first token inside the braces" and misfire, so skip newlines
    // when counting positions.
    match nth_non_newline(p, 3).map(|t| &t.token) {
        Some(LexerTokens::Symbol(LexerSymbol::RCurley)) => true, // `Foo {}`
        Some(LexerTokens::Identifier(_)) => matches!(
            nth_non_newline(p, 4).map(|t| &t.token),
            Some(LexerTokens::Symbol(LexerSymbol::Equal))
                | Some(LexerTokens::Symbol(LexerSymbol::Comma))
                | Some(LexerTokens::Symbol(LexerSymbol::RCurley))
        ),
        _ => false,
    }
}

/// The `n`th token from the current position (1-indexed, same convention
/// as `peek_at`) counting only non-`Newline` tokens — i.e. `peek_at` as if
/// newlines weren't there. Lookahead-only, doesn't consume anything.
fn nth_non_newline<'p>(p: &'p TreeParser<'_, '_>, n: usize) -> Option<&'p SpannedToken> {
    let mut seen = 0;
    let mut k = 1;
    loop {
        let tok = p.peek_at(k)?;
        if !matches!(tok.token, LexerTokens::Newline) {
            seen += 1;
            if seen == n {
                return Some(tok);
            }
        }
        k += 1;
    }
}

fn parse_paren_or_tuple(
    p: &mut TreeParser,
    spans: &mut Vec<CodeSpan>,
) -> Result<(), ElpParserError> {
    let checkpoint = p.checkpoint();
    p.bump(SyntaxKind::LPAREN, spans);
    parse_binary(p, spans, 0)?;

    let mut is_tuple = false;
    loop {
        let Some(tok) = p.peek().cloned() else {
            return Err(expected_got(None, "')'"));
        };
        match &tok.token {
            LexerTokens::Symbol(LexerSymbol::Comma) => {
                is_tuple = true;
                p.bump(SyntaxKind::COMMA, spans);
                if matches!(
                    p.peek().map(|t| &t.token),
                    Some(LexerTokens::Symbol(LexerSymbol::RParen))
                ) {
                    break;
                }
                parse_binary(p, spans, 0)?;
            }
            LexerTokens::Symbol(LexerSymbol::RParen) => break,
            LexerTokens::Newline => p.skip_trivia(spans),
            _ => return Err(unexpected(&tok, "expected ',' or ')'")),
        }
    }

    p.start_node_at(
        checkpoint,
        if is_tuple {
            SyntaxKind::TUPLE_LITERAL_EXPR
        } else {
            SyntaxKind::PAREN_EXPR
        },
    );
    p.bump(SyntaxKind::RPAREN, spans);
    p.finish_node();
    Ok(())
}

fn parse_array_literal(
    p: &mut TreeParser,
    spans: &mut Vec<CodeSpan>,
) -> Result<(), ElpParserError> {
    p.start_node(SyntaxKind::ARRAY_LITERAL_EXPR);
    p.bump(SyntaxKind::LBRACK, spans);

    let mut expect_element = true;
    loop {
        let Some(tok) = p.peek().cloned() else {
            return Err(expected_got(None, "']'"));
        };
        match &tok.token {
            LexerTokens::Symbol(LexerSymbol::RBracket) => break,
            LexerTokens::Symbol(LexerSymbol::Comma) => {
                if expect_element {
                    return Err(unexpected(&tok, "unexpected ','"));
                }
                p.bump(SyntaxKind::COMMA, spans);
                expect_element = true;
            }
            LexerTokens::Newline => p.skip_trivia(spans),
            _ => {
                if !expect_element {
                    return Err(unexpected(&tok, "expected ',' or ']'"));
                }
                parse_binary(p, spans, 0)?;
                expect_element = false;
            }
        }
    }
    p.bump(SyntaxKind::RBRACK, spans);
    p.finish_node(); // ARRAY_LITERAL_EXPR
    Ok(())
}

fn parse_object_literal_body(
    p: &mut TreeParser,
    spans: &mut Vec<CodeSpan>,
) -> Result<(), ElpParserError> {
    p.bump(SyntaxKind::LCURLY, spans);

    let mut expect_field = true;
    loop {
        let Some(tok) = p.peek().cloned() else {
            return Err(expected_got(None, "'}'"));
        };
        match &tok.token {
            LexerTokens::Symbol(LexerSymbol::RCurley) => break,
            LexerTokens::Symbol(LexerSymbol::Comma) => {
                if expect_field {
                    return Err(unexpected(&tok, "unexpected ','"));
                }
                p.bump(SyntaxKind::COMMA, spans);
                expect_field = true;
            }
            LexerTokens::Newline => p.skip_trivia(spans),
            LexerTokens::Identifier(_) => {
                if !expect_field {
                    return Err(unexpected(&tok, "expected ',' or '}'"));
                }
                parse_field_init(p, spans)?;
                expect_field = false;
            }
            _ => return Err(unexpected(&tok, "expected a field")),
        }
    }
    p.bump(SyntaxKind::RCURLY, spans);
    Ok(())
}

fn parse_field_init(p: &mut TreeParser, spans: &mut Vec<CodeSpan>) -> Result<(), ElpParserError> {
    p.start_node(SyntaxKind::FIELD_INIT);
    p.bump(SyntaxKind::IDENT, spans);

    if matches!(
        p.peek().map(|t| &t.token),
        Some(LexerTokens::Symbol(LexerSymbol::Equal))
    ) {
        p.bump(SyntaxKind::EQ, spans);
        parse_binary(p, spans, 0)?;
    }
    p.finish_node(); // FIELD_INIT
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use lexer::ElpLexer;
    use pretty_assertions::assert_eq;

    fn parse(src: &str) -> Expr {
        let mut lexer = ElpLexer::new(src);
        let tokens = lexer.scan();
        let mut cursor = TokenCursor::new(&tokens);
        Expr::from_cursor(&mut cursor).unwrap().node
    }

    fn parse_err(src: &str) -> ElpParserError {
        let mut lexer = ElpLexer::new(src);
        let tokens = lexer.scan();
        let mut cursor = TokenCursor::new(&tokens);
        Expr::from_cursor(&mut cursor).unwrap_err()
    }

    #[test]
    fn integer_literal() {
        let Expr::Literal(lit) = parse("42") else {
            panic!("expected literal")
        };
        assert_eq!(lit.kind(), Some(LiteralKind::Integer));
        assert_eq!(lit.text(), "42");
    }

    #[test]
    fn float_literal() {
        let Expr::Literal(lit) = parse("3.14") else {
            panic!("expected literal")
        };
        assert_eq!(lit.kind(), Some(LiteralKind::Float));
        assert_eq!(lit.text(), "3.14");
    }

    #[test]
    fn string_literal() {
        let Expr::Literal(lit) = parse("\"hello\"") else {
            panic!("expected literal")
        };
        assert_eq!(lit.kind(), Some(LiteralKind::String));
        assert_eq!(lit.text(), "\"hello\"");
    }

    #[test]
    fn boolean_and_nil_literals() {
        assert!(matches!(
            parse("true"),
            Expr::Literal(l) if l.kind() == Some(LiteralKind::True)
        ));
        assert!(matches!(
            parse("false"),
            Expr::Literal(l) if l.kind() == Some(LiteralKind::False)
        ));
        assert!(matches!(
            parse("nil"),
            Expr::Literal(l) if l.kind() == Some(LiteralKind::Nil)
        ));
    }

    #[test]
    fn identifier_expr() {
        let Expr::Ident(ident) = parse("myVar") else {
            panic!("expected ident")
        };
        assert_eq!(ident.name(), "myVar");
    }

    #[test]
    fn self_expr() {
        let Expr::Ident(ident) = parse("self") else {
            panic!("expected ident")
        };
        assert_eq!(ident.name(), "self");
    }

    #[test]
    fn contextual_variant_expr() {
        let Expr::ContextualVariant(v) = parse(".SERVER") else {
            panic!("expected contextual variant")
        };
        assert_eq!(v.name(), "SERVER");
    }

    #[test]
    fn paren_expr_unwraps() {
        let Expr::Paren(paren) = parse("(42)") else {
            panic!("expected paren")
        };
        let Some(Expr::Literal(lit)) = paren.inner() else {
            panic!("expected literal inside parens")
        };
        assert_eq!(lit.text(), "42");
    }

    #[test]
    fn unary_expr() {
        let Expr::Unary(u) = parse("!ready") else {
            panic!("expected unary")
        };
        assert_eq!(u.op_text(), "!");
        let Some(Expr::Ident(operand)) = u.operand() else {
            panic!("expected ident operand")
        };
        assert_eq!(operand.name(), "ready");
    }

    #[test]
    fn double_unary_is_right_recursive() {
        let Expr::Unary(outer) = parse("--x") else {
            panic!("expected unary")
        };
        assert_eq!(outer.op_text(), "-");
        let Some(Expr::Unary(inner)) = outer.operand() else {
            panic!("expected nested unary")
        };
        assert_eq!(inner.op_text(), "-");
    }

    #[test]
    fn deref_expr() {
        let Expr::Unary(u) = parse("*dereffed") else {
            panic!("expected unary")
        };
        assert_eq!(u.op_text(), "*");
        let Some(Expr::Ident(operand)) = u.operand() else {
            panic!("expected ident operand")
        };
        assert_eq!(operand.name(), "dereffed");
    }

    #[test]
    fn deref_is_distinguished_from_multiply_by_position() {
        // Same token (`*`), different grammatical position: prefix at the
        // start of an operand is deref, infix between two operands is
        // multiply. See docs/memory-model.md Finding 6.
        let Expr::Binary(bin) = parse("1 * *x") else {
            panic!("expected the outer expr to be a multiply");
        };
        assert_eq!(bin.op_text(), "*");
        assert!(matches!(bin.lhs(), Some(Expr::Literal(l)) if l.text() == "1"));
        let Some(Expr::Unary(rhs)) = bin.rhs() else {
            panic!("expected rhs to be a deref unary");
        };
        assert_eq!(rhs.op_text(), "*");
        let Some(Expr::Ident(operand)) = rhs.operand() else {
            panic!("expected ident operand")
        };
        assert_eq!(operand.name(), "x");
    }

    #[test]
    fn simple_binary_expr() {
        let Expr::Binary(bin) = parse("1 + 2") else {
            panic!("expected binary")
        };
        assert_eq!(bin.op_text(), "+");
        assert!(matches!(bin.lhs(), Some(Expr::Literal(l)) if l.text() == "1"));
        assert!(matches!(bin.rhs(), Some(Expr::Literal(l)) if l.text() == "2"));
    }

    #[test]
    fn precedence_multiplication_binds_tighter_than_addition() {
        // 1 + 2 * 3 should parse as 1 + (2 * 3), i.e. the outer node is '+'.
        let Expr::Binary(outer) = parse("1 + 2 * 3") else {
            panic!("expected binary")
        };
        assert_eq!(outer.op_text(), "+");
        assert!(matches!(outer.lhs(), Some(Expr::Literal(l)) if l.text() == "1"));
        let Some(Expr::Binary(inner)) = outer.rhs() else {
            panic!("expected nested multiplication on the rhs")
        };
        assert_eq!(inner.op_text(), "*");
    }

    #[test]
    fn left_associative_chain() {
        // 1 - 2 - 3 should parse as (1 - 2) - 3, i.e. the outer node's LHS
        // is itself a binary expression.
        let Expr::Binary(outer) = parse("1 - 2 - 3") else {
            panic!("expected binary")
        };
        assert!(matches!(outer.rhs(), Some(Expr::Literal(l)) if l.text() == "3"));
        let Some(Expr::Binary(inner)) = outer.lhs() else {
            panic!("expected nested subtraction on the lhs")
        };
        assert!(matches!(inner.lhs(), Some(Expr::Literal(l)) if l.text() == "1"));
        assert!(matches!(inner.rhs(), Some(Expr::Literal(l)) if l.text() == "2"));
    }

    #[test]
    fn logical_operators_bind_looser_than_equality() {
        // a == b && c == d -> && at the top, == on each side.
        let Expr::Binary(outer) = parse("a == b && c == d") else {
            panic!("expected binary")
        };
        assert_eq!(outer.op_text(), "&&");
        assert!(matches!(outer.lhs(), Some(Expr::Binary(b)) if b.op_text() == "=="));
        assert!(matches!(outer.rhs(), Some(Expr::Binary(b)) if b.op_text() == "=="));
    }

    #[test]
    fn member_access() {
        let Expr::Member(m) = parse("user.name") else {
            panic!("expected member")
        };
        assert_eq!(m.member_name(), "name");
        assert!(matches!(m.receiver(), Some(Expr::Ident(i)) if i.name() == "user"));
    }

    #[test]
    fn chained_member_access() {
        let Expr::Member(outer) = parse("a.b.c") else {
            panic!("expected member")
        };
        assert_eq!(outer.member_name(), "c");
        let Some(Expr::Member(inner)) = outer.receiver() else {
            panic!("expected nested member access")
        };
        assert_eq!(inner.member_name(), "b");
    }

    #[test]
    fn call_with_no_args() {
        let Expr::Call(call) = parse("println()") else {
            panic!("expected call")
        };
        assert!(matches!(call.callee(), Some(Expr::Ident(i)) if i.name() == "println"));
        assert_eq!(call.args().len(), 0);
    }

    #[test]
    fn call_with_positional_args() {
        let Expr::Call(call) = parse("println(\"hello\", 42)") else {
            panic!("expected call")
        };
        let args = call.args();
        assert_eq!(args.len(), 2);
        assert_eq!(args[0].name(), None);
        assert!(matches!(&args[0].value(), Some(Expr::Literal(l)) if l.text() == "\"hello\""));
        assert!(matches!(&args[1].value(), Some(Expr::Literal(l)) if l.text() == "42"));
    }

    #[test]
    fn call_with_named_arg() {
        let Expr::Call(call) = parse("App(globalStyles = Styles())") else {
            panic!("expected call")
        };
        let args = call.args();
        assert_eq!(args.len(), 1);
        assert_eq!(args[0].name(), Some("globalStyles".to_string()));
        assert!(
            matches!(&args[0].value(), Some(Expr::Call(inner)) if matches!(inner.callee(), Some(Expr::Ident(i)) if i.name() == "Styles"))
        );
    }

    #[test]
    fn method_call_chain() {
        let Expr::Call(call) = parse("Http.post(body)") else {
            panic!("expected call")
        };
        let Some(Expr::Member(callee)) = call.callee() else {
            panic!("expected member-access callee")
        };
        assert_eq!(callee.member_name(), "post");
    }

    #[test]
    fn index_expr() {
        let Expr::Index(idx) = parse("items[0]") else {
            panic!("expected index")
        };
        assert!(matches!(idx.receiver(), Some(Expr::Ident(i)) if i.name() == "items"));
        assert!(matches!(idx.index(), Some(Expr::Literal(l)) if l.text() == "0"));
    }

    #[test]
    fn empty_array_literal() {
        let Expr::ArrayLiteral(arr) = parse("[]") else {
            panic!("expected array")
        };
        assert_eq!(arr.elements().len(), 0);
    }

    #[test]
    fn array_literal_with_elements() {
        let Expr::ArrayLiteral(arr) = parse("[1, 2, 3]") else {
            panic!("expected array")
        };
        let elements = arr.elements();
        assert_eq!(elements.len(), 3);
        assert!(matches!(&elements[2], Expr::Literal(l) if l.text() == "3"));
    }

    #[test]
    fn array_literal_trailing_comma_allowed() {
        let Expr::ArrayLiteral(arr) = parse("[1, 2,]") else {
            panic!("expected array")
        };
        assert_eq!(arr.elements().len(), 2);
    }

    #[test]
    fn tuple_literal_requires_comma() {
        let Expr::TupleLiteral(tuple) = parse("(1, 2)") else {
            panic!("expected tuple")
        };
        assert_eq!(tuple.elements().len(), 2);
    }

    #[test]
    fn object_literal_with_type_name() {
        let Expr::ObjectLiteral(obj) = parse("Self { nodes = 1, edges = 2 }") else {
            panic!("expected object literal")
        };
        assert_eq!(obj.type_name(), Some("Self".to_string()));
        let fields = obj.fields();
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].name(), "nodes");
        assert!(matches!(&fields[0].value(), Some(Expr::Literal(l)) if l.text() == "1"));
    }

    #[test]
    fn object_literal_without_type_name() {
        let Expr::ObjectLiteral(obj) = parse("{ success = true }") else {
            panic!("expected object literal")
        };
        assert_eq!(obj.type_name(), None);
        assert_eq!(obj.fields().len(), 1);
    }

    #[test]
    fn object_literal_shorthand_field() {
        let Expr::ObjectLiteral(obj) = parse("Self { nodes, edges }") else {
            panic!("expected object literal")
        };
        let fields = obj.fields();
        assert_eq!(fields[0].name(), "nodes");
        assert_eq!(fields[0].value(), None);
    }

    #[test]
    fn object_literal_with_type_name_newline_after_brace() {
        // Regression: `looks_like_object_literal_body`'s lookahead used
        // raw `peek_at` positions, so a newline right after `{` (the
        // common real-world style — see e.g. ChatRoom/state/app.elp) was
        // counted as "the first token inside the braces", never matched
        // IDENT/RCurley, and made this misfire as a builder block —
        // which then broke immediately since a builder block parses its
        // contents as *statements*, not comma-separated field inits.
        let src = "Self {\n\tusersInRoom = [user],\n\tmessages = []\n}";
        let Expr::ObjectLiteral(obj) = parse(src) else {
            panic!("expected object literal, not a builder block")
        };
        let fields = obj.fields();
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].name(), "usersInRoom");
        assert_eq!(fields[1].name(), "messages");
    }

    #[test]
    fn real_world_shape_reducer_call() {
        // Lifted from examples/ChatRoom/main.velp's AppReducer body.
        let Expr::Call(call) = parse("state.copyWith(authed)") else {
            panic!("expected call")
        };
        let Some(Expr::Member(callee)) = call.callee() else {
            panic!("expected member-access callee")
        };
        assert_eq!(callee.member_name(), "copyWith");
        assert!(matches!(callee.receiver(), Some(Expr::Ident(i)) if i.name() == "state"));
    }

    #[test]
    fn error_unclosed_call() {
        let err = parse_err("println(\"hi\"");
        let msg = format!("{err}");
        assert!(msg.contains("')'"), "unexpected error: {msg}");
    }

    #[test]
    fn error_unclosed_array() {
        let err = parse_err("[1, 2");
        let msg = format!("{err}");
        assert!(msg.contains("']'"), "unexpected error: {msg}");
    }

    #[test]
    fn error_dangling_operator() {
        let err = parse_err("1 +");
        let msg = format!("{err}");
        assert!(msg.contains("expression"), "unexpected error: {msg}");
    }

    #[test]
    fn error_dot_without_identifier() {
        let err = parse_err(".)");
        let msg = format!("{err}");
        assert!(msg.contains("identifier"), "unexpected error: {msg}");
    }

    #[test]
    fn syntax_tree_records_meaningful_tokens_in_order() {
        let node = parse("1 + 2 * 3");
        let tokens: Vec<String> = node
            .syntax()
            .descendants_with_tokens()
            .filter_map(|el| el.into_token())
            .map(|t| t.text().to_string())
            .collect();
        assert_eq!(tokens, vec!["1", "+", "2", "*", "3"]);
    }
}
