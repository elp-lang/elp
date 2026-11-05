use cst::{AstNode, SyntaxKind, SyntaxNode};
use lexer::{
    span::CodeSpan,
    tokens::{LexerSymbol, LexerTokens},
};

use crate::{
    Spanned, TokenCursor, TreeParser,
    ast::{
        expression::{self, Expr},
        statement,
        ty::{self, Type},
    },
    error::{ElpParserError, ParsingError, expected_got, unexpected},
    span::spans_extent,
};

/// Plain, easily-asserted-against view of one `Name (as Alias)?` entry in an
/// import list — extracted from the tree by [`ImportStmt::names`] rather
/// than stored directly, so the tree stays the single source of truth.
#[derive(Debug, PartialEq, Clone)]
pub struct ImportNameData {
    pub name: String,
    pub alias: Option<String>,
}

impl ImportNameData {
    pub fn plain(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            alias: None,
        }
    }

    pub fn aliased(name: impl Into<String>, alias: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            alias: Some(alias.into()),
        }
    }
}

/// A parsed `import { ... } from ...` statement — a typed view over an
/// `IMPORT_STMT` [`SyntaxNode`]. Every accessor walks the wrapped tree on
/// demand; nothing here is cached at parse time.
#[derive(Debug, PartialEq, Clone)]
pub struct ImportStmt(SyntaxNode);

impl AstNode for ImportStmt {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::IMPORT_STMT
    }

    fn cast(node: SyntaxNode) -> Option<Self> {
        Self::can_cast(node.kind()).then(|| Self(node))
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.0
    }
}

impl ImportStmt {
    pub fn names(&self) -> Vec<ImportNameData> {
        self.0
            .descendants()
            .filter(|n| n.kind() == SyntaxKind::IMPORT_NAME)
            .map(|import_name| {
                let idents: Vec<String> = import_name
                    .children_with_tokens()
                    .filter_map(|el| el.into_token())
                    .filter(|t| t.kind() == SyntaxKind::IDENT)
                    .map(|t| t.text().to_string())
                    .collect();

                ImportNameData {
                    name: idents.first().cloned().unwrap_or_default(),
                    alias: idents.get(1).cloned(),
                }
            })
            .collect()
    }

    pub fn module_path(&self) -> Vec<String> {
        let Some(module_path) = self
            .0
            .children()
            .find(|n| n.kind() == SyntaxKind::MODULE_PATH)
        else {
            return vec![];
        };

        module_path
            .children_with_tokens()
            .filter_map(|el| el.into_token())
            .filter(|t| t.kind() == SyntaxKind::IDENT)
            .map(|t| t.text().to_string())
            .collect()
    }

    /// True for a leading-dot relative import like `from .components.paragraph`.
    pub fn is_relative(&self) -> bool {
        self.0
            .children()
            .find(|n| n.kind() == SyntaxKind::MODULE_PATH)
            .and_then(|module_path| module_path.first_token())
            .is_some_and(|t| t.kind() == SyntaxKind::DOT)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Decl {
    Import(ImportStmt),
    Var(VarDecl),
    Const(ConstDecl),
    Object(ObjectDecl),
    Interface(InterfaceDecl),
    Implementing(ImplementingDecl),
    Enum(EnumDecl),
    Fn(FnDecl),
}

impl Decl {
    pub fn from_cursor(cursor: &mut TokenCursor) -> Result<Spanned<Self>, ElpParserError> {
        if cursor.peek().is_none() {
            return Err(ElpParserError::ScriptParsingError(ParsingError::Expected(
                "No token to peek at, this is a compiler bug".into(),
            )));
        }

        // `export` and `pure` (in that order: `export pure fn ...`) prefix
        // another declaration rather than being one themselves — look past
        // both to find the keyword that actually decides which parser to
        // dispatch to. Each parser below independently re-checks for and
        // consumes its own leading `export`/`pure`, from the real current
        // cursor position — this lookahead only exists to pick the right
        // parser, it doesn't consume anything itself. `peek_at(1)` is the
        // current token (same as `peek()`).
        let mut n = 1;
        let keyword = loop {
            match cursor.peek_at(n).map(|t| &t.token) {
                Some(LexerTokens::Identifier(kw)) if kw == "export" || kw == "pure" => {
                    n += 1;
                }
                Some(LexerTokens::Identifier(kw)) => break kw.clone(),
                _ => return Err(expected_got(cursor.peek_at(n), "a declaration keyword")),
            }
        };

        match keyword.as_str() {
            "import" => import_from_cursor(cursor),
            // elp has no package-level variables — mutable state lives in an
            // object or a component, never at file scope. `const` stays: a
            // compile-time constant isn't state. Still dispatched here (via
            // `matches_ident`) rather than dropped from the keyword list, so
            // this says what's wrong instead of the expression parser
            // failing on a stray `var`.
            "var" => Err(ElpParserError::ScriptParsingError(
                ParsingError::Unexpected(
                    cursor.peek_at(n).map(|t| t.span.clone()),
                    "top-level `var` — elp has no package-level variables; use `const` for a \
                     file-scope constant, or hold mutable state in an object or component"
                        .into(),
                ),
            )),
            "const" => {
                var_or_const_from_cursor(cursor, SyntaxKind::CONST_DECL, SyntaxKind::CONST_KW)
            }
            "object" => object_from_cursor(cursor),
            "interface" => interface_from_cursor(cursor),
            "enum" => enum_from_cursor(cursor),
            "fn" => fn_from_cursor(cursor),
            // Not a declaration keyword — the only remaining declaration
            // shape is `TypeName implementing A + B`, which opens with the
            // conforming type's name rather than a keyword and so is
            // recognised by its *second* token. Checked here rather than
            // before the match so an object legitimately named
            // `implementing` still parses as an object.
            _ if is_implementing_kw(cursor.peek_at(n + 1)) => implementing_from_cursor(cursor),
            _ => Err(ElpParserError::ScriptParsingError(
                ParsingError::Unexpected(None, "Unexpected token".into()),
            )),
        }
    }

    pub fn matches_ident(ident: &str) -> bool {
        matches!(
            ident,
            "fn" | "var" | "const" | "import" | "object" | "interface" | "enum" | "export"
        )
    }

    /// Top-level dispatch gate. Wider than [`Decl::matches_ident`], which
    /// only ever sees one identifier: `TypeName implementing ...` can't be
    /// recognised from its first token alone, since that token is an
    /// ordinary type name. Anything this returns false for is handed to the
    /// expression parser instead.
    pub fn starts_at(cursor: &TokenCursor) -> bool {
        match cursor.peek().map(|t| &t.token) {
            Some(LexerTokens::Identifier(ident)) => {
                Self::matches_ident(ident) || is_implementing_kw(cursor.peek_at(2))
            }
            _ => false,
        }
    }
}

/// `implementing` is contextual, not reserved — it's an ordinary identifier
/// token everywhere except directly after a type name at declaration
/// position, same as every other "keyword" in this lexer.
fn is_implementing_kw(token: Option<&lexer::span::SpannedToken>) -> bool {
    matches!(token.map(|t| &t.token), Some(LexerTokens::Identifier(i)) if i == "implementing")
}

/// Every `*_DECL` node accepts the same optional leading `export` — called
/// at the very start of each declaration parser, before its own keyword.
fn bump_optional_export(p: &mut TreeParser, spans: &mut Vec<CodeSpan>) {
    if matches!(p.peek().map(|t| &t.token), Some(LexerTokens::Identifier(i)) if i == "export") {
        p.bump(SyntaxKind::EXPORT_KW, spans);
    }
}

fn is_exported(node: &SyntaxNode) -> bool {
    node.first_token()
        .is_some_and(|t| t.kind() == SyntaxKind::EXPORT_KW)
}

fn import_from_cursor(cursor: &mut TokenCursor) -> Result<Spanned<Decl>, ElpParserError> {
    let mut spans = vec![];
    let mut p = TreeParser::new(cursor);
    p.start_node(SyntaxKind::IMPORT_STMT);

    // Consume `import` and `{`.
    p.bump(SyntaxKind::IMPORT_KW, &mut spans);

    let brace_tok = p
        .peek()
        .cloned()
        .ok_or_else(|| expected_got(None, "'{' after import"))?;
    if !matches!(&brace_tok.token, LexerTokens::Symbol(LexerSymbol::LCurley)) {
        return Err(expected_got(Some(&brace_tok), "'{' after import"));
    }
    p.bump(SyntaxKind::LCURLY, &mut spans);

    // Collect `{ Name (as Alias)?, ... }`.
    // State: after a name we may see `as`, `,`, or `}`. After `,` or `{` we expect a name.
    p.start_node(SyntaxKind::IMPORT_LIST);
    let mut closed = false;
    let mut expect_name = true;
    while let Some(token) = p.peek().cloned() {
        match &token.token {
            LexerTokens::Identifier(_) => {
                if !expect_name {
                    return Err(unexpected(&token, "expected ',' or '}' between imports"));
                }

                p.start_node(SyntaxKind::IMPORT_NAME);
                p.bump(SyntaxKind::IDENT, &mut spans);

                // Peek past any newlines to see if there's an `as` alias.
                loop {
                    match p.peek().cloned() {
                        Some(next) if matches!(next.token, LexerTokens::Newline) => {
                            p.skip_trivia(&mut spans);
                            continue;
                        }
                        Some(next) => {
                            if let LexerTokens::Identifier(maybe_as) = &next.token
                                && maybe_as == "as"
                            {
                                p.bump(SyntaxKind::AS_KW, &mut spans);

                                // Skip newlines between `as` and the alias identifier.
                                while let Some(t) = p.peek() {
                                    if matches!(t.token, LexerTokens::Newline) {
                                        p.skip_trivia(&mut spans);
                                    } else {
                                        break;
                                    }
                                }

                                let alias_tok = p.peek().cloned().ok_or_else(|| {
                                    expected_got(None, "alias identifier after 'as'")
                                })?;
                                if matches!(alias_tok.token, LexerTokens::Identifier(_)) {
                                    p.bump(SyntaxKind::IDENT, &mut spans);
                                } else {
                                    return Err(expected_got(
                                        Some(&alias_tok),
                                        "alias identifier after 'as'",
                                    ));
                                }
                            }
                        }
                        None => {}
                    }
                    break;
                }

                p.finish_node(); // IMPORT_NAME
                expect_name = false;
            }
            LexerTokens::Symbol(symbol) => match symbol {
                LexerSymbol::RCurley => {
                    p.bump(SyntaxKind::RCURLY, &mut spans);
                    closed = true;
                    break;
                }
                LexerSymbol::Comma => {
                    if expect_name {
                        return Err(unexpected(&token, "unexpected ','"));
                    }
                    p.bump(SyntaxKind::COMMA, &mut spans);
                    expect_name = true;
                    continue;
                }
                _ => {
                    return Err(unexpected(&token, "unexpected token in import list"));
                }
            },
            LexerTokens::Newline => {
                p.skip_trivia(&mut spans);
                continue;
            }
            LexerTokens::EOI => {
                return Err(expected_got(Some(&token), "'}' to close import list"));
            }
            _ => {
                return Err(unexpected(&token, "unexpected token in import list"));
            }
        };
    }
    p.finish_node(); // IMPORT_LIST

    if !closed {
        return Err(expected_got(None, "'}' to close import list"));
    }

    // Skip any newlines between `}` and `from`.
    while let Some(t) = p.peek() {
        if matches!(t.token, LexerTokens::Newline) {
            p.skip_trivia(&mut spans);
        } else {
            break;
        }
    }

    // Expect the `from` keyword.
    let from_tok = p
        .peek()
        .cloned()
        .ok_or_else(|| expected_got(None, "'from' after import list"))?;
    match &from_tok.token {
        LexerTokens::Identifier(i) if i == "from" => {
            p.bump(SyntaxKind::FROM_KW, &mut spans);
        }
        _ => {
            return Err(expected_got(Some(&from_tok), "'from' after import list"));
        }
    }

    p.start_node(SyntaxKind::MODULE_PATH);

    // Optional leading `.` marks a relative import.
    if let Some(t) = p.peek()
        && matches!(t.token, LexerTokens::Symbol(LexerSymbol::Period))
    {
        p.bump(SyntaxKind::DOT, &mut spans);
    }

    // Collect the module path: ident (`.` ident)*
    let mut saw_segment = false;
    let mut expect_segment = true;
    while let Some(token) = p.peek().cloned() {
        match &token.token {
            LexerTokens::Identifier(_) => {
                if !expect_segment {
                    break;
                }
                p.bump(SyntaxKind::IDENT, &mut spans);
                saw_segment = true;
                expect_segment = false;
            }
            LexerTokens::Symbol(LexerSymbol::Period) => {
                if expect_segment {
                    return Err(unexpected(&token, "unexpected '.' in module path"));
                }
                p.bump(SyntaxKind::DOT, &mut spans);
                expect_segment = true;
                continue;
            }
            LexerTokens::Newline | LexerTokens::EOI => break,
            _ => break,
        }
    }
    p.finish_node(); // MODULE_PATH

    if !saw_segment {
        return Err(expected_got(None, "module path after 'from'"));
    }

    p.finish_node(); // IMPORT_STMT
    let root = p.finish();
    let import_stmt =
        ImportStmt::cast(root).expect("builder always roots this tree at IMPORT_STMT");

    Ok(Spanned {
        node: Decl::Import(import_stmt),
        span: spans_extent(spans)?,
    })
}

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

ast_node!(VarDecl, SyntaxKind::VAR_DECL);
ast_node!(ConstDecl, SyntaxKind::CONST_DECL);
ast_node!(ObjectDecl, SyntaxKind::OBJECT_DECL);
ast_node!(InterfaceDecl, SyntaxKind::INTERFACE_DECL);
ast_node!(ImplementingDecl, SyntaxKind::IMPLEMENTING_DECL);
ast_node!(EnumDecl, SyntaxKind::ENUM_DECL);
ast_node!(FnDecl, SyntaxKind::FN_DECL);
ast_node!(Field, SyntaxKind::FIELD);
ast_node!(InterfaceMethod, SyntaxKind::INTERFACE_METHOD);
ast_node!(CallbackDecl, SyntaxKind::CALLBACK_DECL);
ast_node!(Variant, SyntaxKind::VARIANT);
ast_node!(Param, SyntaxKind::PARAM);
ast_node!(SelfParam, SyntaxKind::SELF_PARAM);

fn var_pattern_names(node: &SyntaxNode) -> Vec<String> {
    node.children()
        .find(|n| n.kind() == SyntaxKind::VAR_PATTERN)
        .map(|pat| {
            pat.children_with_tokens()
                .filter_map(|el| el.into_token())
                .filter(|t| t.kind() == SyntaxKind::IDENT)
                .map(|t| t.text().to_string())
                .collect()
        })
        .unwrap_or_default()
}

fn declared_type(node: &SyntaxNode) -> Option<Type> {
    node.children().find_map(Type::cast)
}

fn initializer_value(node: &SyntaxNode) -> Option<Expr> {
    // The initializer is whichever expression-castable child comes right
    // after a bare `=` token — distinguishes `const x Type = default` from
    // types/patterns that might otherwise also contain expression-shaped
    // children in a future grammar extension. Today, walking children for
    // the first `Expr::cast` success is equivalent and simpler; kept as its
    // own function so callers read intent, not tree-walking mechanics.
    node.children().find_map(Expr::cast)
}

impl VarDecl {
    pub fn is_exported(&self) -> bool {
        is_exported(&self.0)
    }
    pub fn pattern(&self) -> Vec<String> {
        var_pattern_names(&self.0)
    }
    pub fn ty(&self) -> Option<Type> {
        declared_type(&self.0)
    }
    pub fn value(&self) -> Option<Expr> {
        initializer_value(&self.0)
    }
}

impl ConstDecl {
    pub fn is_exported(&self) -> bool {
        is_exported(&self.0)
    }
    pub fn pattern(&self) -> Vec<String> {
        var_pattern_names(&self.0)
    }
    pub fn ty(&self) -> Option<Type> {
        declared_type(&self.0)
    }
    pub fn value(&self) -> Option<Expr> {
        initializer_value(&self.0)
    }
}

impl Field {
    pub fn name(&self) -> String {
        self.0
            .children_with_tokens()
            .filter_map(|el| el.into_token())
            .find(|t| t.kind() == SyntaxKind::IDENT)
            .map(|t| t.text().to_string())
            .unwrap_or_default()
    }

    pub fn ty(&self) -> Option<Type> {
        self.0.children().find_map(Type::cast)
    }

    pub fn default_value(&self) -> Option<Expr> {
        self.0.children().find_map(Expr::cast)
    }
}

impl ObjectDecl {
    pub fn is_exported(&self) -> bool {
        is_exported(&self.0)
    }

    pub fn name(&self) -> String {
        self.0
            .children_with_tokens()
            .filter_map(|el| el.into_token())
            .find(|t| t.kind() == SyntaxKind::IDENT)
            .map(|t| t.text().to_string())
            .unwrap_or_default()
    }

    pub fn fields(&self) -> Vec<Field> {
        self.0
            .children()
            .find(|n| n.kind() == SyntaxKind::FIELD_LIST)
            .map(|list| list.children().filter_map(Field::cast).collect())
            .unwrap_or_default()
    }
}

impl ImplementingDecl {
    /// The conforming type's name. Just an identifier here — whether it
    /// names a real `object`/`enum` is name resolution's problem, not the
    /// parser's, exactly as with any other type reference.
    pub fn type_name(&self) -> String {
        self.0
            .children_with_tokens()
            .filter_map(|el| el.into_token())
            .find(|t| t.kind() == SyntaxKind::IDENT)
            .map(|t| t.text().to_string())
            .unwrap_or_default()
    }

    /// Every interface named after `implementing`, in source order. Always
    /// at least one — a bare `Foo implementing` is a parse error.
    pub fn interfaces(&self) -> Vec<Type> {
        self.0
            .children()
            .find(|n| n.kind() == SyntaxKind::INTERFACE_LIST)
            .map(|list| list.children().filter_map(Type::cast).collect())
            .unwrap_or_default()
    }
}

impl InterfaceMethod {
    pub fn name(&self) -> String {
        self.0
            .children_with_tokens()
            .filter_map(|el| el.into_token())
            .find(|t| t.kind() == SyntaxKind::IDENT)
            .map(|t| t.text().to_string())
            .unwrap_or_default()
    }

    pub fn params(&self) -> Vec<Param> {
        self.0
            .children()
            .find(|n| n.kind() == SyntaxKind::PARAM_LIST)
            .map(|list| list.children().filter_map(Param::cast).collect())
            .unwrap_or_default()
    }

    pub fn return_type(&self) -> Option<Type> {
        self.0
            .children()
            .filter(|n| n.kind() != SyntaxKind::PARAM_LIST)
            .find_map(Type::cast)
    }
}

/// A `*fn` callback slot. Deliberately has no `is_exported()`: `export` is
/// mandatory on this form (see [`parse_callback_decl`]), so the accessor
/// would be a constant `true` — every callback is a prop.
impl CallbackDecl {
    pub fn name(&self) -> String {
        self.0
            .children_with_tokens()
            .filter_map(|el| el.into_token())
            .find(|t| t.kind() == SyntaxKind::IDENT)
            .map(|t| t.text().to_string())
            .unwrap_or_default()
    }

    pub fn params(&self) -> Vec<Param> {
        self.0
            .children()
            .find(|n| n.kind() == SyntaxKind::PARAM_LIST)
            .map(|list| list.children().filter_map(Param::cast).collect())
            .unwrap_or_default()
    }

    pub fn return_type(&self) -> Option<Type> {
        self.0
            .children()
            .filter(|n| n.kind() != SyntaxKind::PARAM_LIST)
            .find_map(Type::cast)
    }
}

impl InterfaceDecl {
    pub fn is_exported(&self) -> bool {
        is_exported(&self.0)
    }

    pub fn name(&self) -> String {
        self.0
            .children_with_tokens()
            .filter_map(|el| el.into_token())
            .find(|t| t.kind() == SyntaxKind::IDENT)
            .map(|t| t.text().to_string())
            .unwrap_or_default()
    }

    pub fn methods(&self) -> Vec<InterfaceMethod> {
        self.0
            .children()
            .find(|n| n.kind() == SyntaxKind::INTERFACE_BODY)
            .map(|body| body.children().filter_map(InterfaceMethod::cast).collect())
            .unwrap_or_default()
    }
}

impl Variant {
    pub fn name(&self) -> String {
        self.0
            .children_with_tokens()
            .filter_map(|el| el.into_token())
            .find(|t| t.kind() == SyntaxKind::IDENT)
            .map(|t| t.text().to_string())
            .unwrap_or_default()
    }

    pub fn payload_types(&self) -> Vec<Type> {
        self.0
            .children()
            .find(|n| n.kind() == SyntaxKind::TYPE_LIST)
            .map(|list| list.children().filter_map(Type::cast).collect())
            .unwrap_or_default()
    }
}

impl EnumDecl {
    pub fn is_exported(&self) -> bool {
        is_exported(&self.0)
    }

    pub fn name(&self) -> String {
        self.0
            .children_with_tokens()
            .filter_map(|el| el.into_token())
            .find(|t| t.kind() == SyntaxKind::IDENT)
            .map(|t| t.text().to_string())
            .unwrap_or_default()
    }

    pub fn variants(&self) -> Vec<Variant> {
        self.0
            .children()
            .find(|n| n.kind() == SyntaxKind::ENUM_VARIANTS)
            .map(|list| list.children().filter_map(Variant::cast).collect())
            .unwrap_or_default()
    }
}

impl SelfParam {
    pub fn is_ref(&self) -> bool {
        self.0
            .first_token()
            .is_some_and(|t| t.kind() == SyntaxKind::AMP)
    }
}

impl Param {
    /// `None` for a bare unnamed-type param (FFI-style declarations like
    /// `fn OBJC_msgSend<T>(T, OBJC_Selector) -> T`).
    pub fn name(&self) -> Option<String> {
        let mut children = self.0.children_with_tokens();
        let mut next = children.next()?;
        // Skip an optional leading `const`.
        if matches!(
            next.as_token().map(|t| t.kind()),
            Some(SyntaxKind::CONST_KW)
        ) {
            next = children.next()?;
        }
        // A named param bumps `IDENT` for the name before parsing the type
        // as its own child *node*; a bare-type param has no such token —
        // its first child is the type node directly.
        let token = next.into_token()?;
        (token.kind() == SyntaxKind::IDENT).then(|| token.text().to_string())
    }

    pub fn ty(&self) -> Option<Type> {
        self.0.children().find_map(Type::cast)
    }

    pub fn default_value(&self) -> Option<Expr> {
        self.0.children().find_map(Expr::cast)
    }

    pub fn is_const(&self) -> bool {
        self.0
            .children_with_tokens()
            .filter_map(|el| el.into_token())
            .any(|t| t.kind() == SyntaxKind::CONST_KW)
    }
}

impl FnDecl {
    pub fn is_exported(&self) -> bool {
        is_exported(&self.0)
    }

    pub fn is_pure(&self) -> bool {
        self.0
            .children_with_tokens()
            .filter_map(|el| el.into_token())
            .any(|t| t.kind() == SyntaxKind::PURE_KW)
    }

    pub fn name(&self) -> String {
        self.0
            .children_with_tokens()
            .filter_map(|el| el.into_token())
            .find(|t| t.kind() == SyntaxKind::IDENT)
            .map(|t| t.text().to_string())
            .unwrap_or_default()
    }

    /// The `.method` segment of `fn Type.method(...)`, if present.
    pub fn method_name(&self) -> Option<String> {
        self.0
            .children_with_tokens()
            .filter_map(|el| el.into_token())
            .filter(|t| t.kind() == SyntaxKind::IDENT)
            .nth(1)
            .map(|t| t.text().to_string())
    }

    pub fn params(&self) -> Vec<Param> {
        self.0
            .children()
            .find(|n| n.kind() == SyntaxKind::PARAM_LIST)
            .map(|list| list.children().filter_map(Param::cast).collect())
            .unwrap_or_default()
    }

    pub fn return_type(&self) -> Option<Type> {
        self.0
            .children()
            .filter(|n| n.kind() != SyntaxKind::PARAM_LIST)
            .find_map(Type::cast)
    }

    /// Body-less declarations are legal (`@external` FFI forward
    /// declarations) — a semantic pass, once one exists, should reject
    /// body-less `fn`s that aren't marked `@external` or similar.
    pub fn has_body(&self) -> bool {
        self.0.children().any(|n| n.kind() == SyntaxKind::BLOCK)
    }

    pub fn body(&self) -> Option<statement::Block> {
        self.0.children().find_map(statement::Block::cast)
    }
}

// ---------- var / const ----------

fn var_or_const_from_cursor(
    cursor: &mut TokenCursor,
    decl_kind: SyntaxKind,
    keyword_kind: SyntaxKind,
) -> Result<Spanned<Decl>, ElpParserError> {
    let mut spans = vec![];
    let mut p = TreeParser::new(cursor);
    parse_var_or_const_body(&mut p, &mut spans, decl_kind, keyword_kind)?;
    let root = p.finish();

    let node = if decl_kind == SyntaxKind::VAR_DECL {
        Decl::Var(VarDecl::cast(root).expect("builder rooted this tree at VAR_DECL"))
    } else {
        Decl::Const(ConstDecl::cast(root).expect("builder rooted this tree at CONST_DECL"))
    };

    Ok(Spanned {
        node,
        span: spans_extent(spans)?,
    })
}

/// Core of `var`/`const` parsing, operating on an already-shared
/// `TreeParser` rather than creating its own — so it can be reused inside
/// `statement.rs`'s block parsing (`var x = 1` inside a fn body needs to
/// land in the *same* tree as the block around it, not a standalone one).
pub(crate) fn parse_var_or_const_body(
    p: &mut TreeParser,
    spans: &mut Vec<CodeSpan>,
    decl_kind: SyntaxKind,
    keyword_kind: SyntaxKind,
) -> Result<(), ElpParserError> {
    p.start_node(decl_kind);

    bump_optional_export(p, spans);
    p.bump(keyword_kind, spans);
    parse_var_pattern(p, spans)?;

    // Optional type: only if what follows looks like the start of one,
    // rather than `=` or the end of the declaration.
    if matches!(
        p.peek().map(|t| &t.token),
        Some(LexerTokens::Identifier(_))
            | Some(LexerTokens::Symbol(LexerSymbol::Ampersand))
            | Some(LexerTokens::Symbol(LexerSymbol::LBracket))
    ) {
        ty::parse_type(p, spans)?;
    }

    if matches!(
        p.peek().map(|t| &t.token),
        Some(LexerTokens::Symbol(LexerSymbol::Equal))
    ) {
        p.bump(SyntaxKind::EQ, spans);
        expression::parse_binary(p, spans, 0)?;
    }

    p.finish_node();
    Ok(())
}

fn parse_var_pattern(p: &mut TreeParser, spans: &mut Vec<CodeSpan>) -> Result<(), ElpParserError> {
    p.start_node(SyntaxKind::VAR_PATTERN);
    let first = p
        .peek()
        .cloned()
        .ok_or_else(|| expected_got(None, "a binding name"))?;
    if !matches!(first.token, LexerTokens::Identifier(_)) {
        return Err(expected_got(Some(&first), "a binding name"));
    }
    p.bump(SyntaxKind::IDENT, spans);

    while matches!(
        p.peek().map(|t| &t.token),
        Some(LexerTokens::Symbol(LexerSymbol::Comma))
    ) {
        p.bump(SyntaxKind::COMMA, spans);
        let next = p
            .peek()
            .cloned()
            .ok_or_else(|| expected_got(None, "a binding name after ','"))?;
        if !matches!(next.token, LexerTokens::Identifier(_)) {
            return Err(expected_got(Some(&next), "a binding name after ','"));
        }
        p.bump(SyntaxKind::IDENT, spans);
    }
    p.finish_node();
    Ok(())
}

// ---------- shared: type lists, field lists, param lists ----------

fn parse_type_list(p: &mut TreeParser, spans: &mut Vec<CodeSpan>) -> Result<(), ElpParserError> {
    p.start_node(SyntaxKind::TYPE_LIST);
    ty::parse_type(p, spans)?;
    while matches!(
        p.peek().map(|t| &t.token),
        Some(LexerTokens::Symbol(LexerSymbol::Comma))
    ) {
        p.bump(SyntaxKind::COMMA, spans);
        ty::parse_type(p, spans)?;
    }
    p.finish_node();
    Ok(())
}

pub(crate) fn skip_newlines(p: &mut TreeParser, spans: &mut Vec<CodeSpan>) {
    while matches!(p.peek().map(|t| &t.token), Some(LexerTokens::Newline)) {
        p.skip_trivia(spans);
    }
}

/// `(field ','?)*` — commas are optional since real-world field lists are
/// newline-separated throughout the example corpus.
fn parse_field_list(
    p: &mut TreeParser,
    spans: &mut Vec<CodeSpan>,
    close: LexerSymbol,
) -> Result<(), ElpParserError> {
    loop {
        skip_newlines(p, spans);
        match p.peek().map(|t| &t.token) {
            Some(LexerTokens::Symbol(s)) if *s == close => break,
            None => return Err(expected_got(None, "a field or the closing '}'")),
            _ => {}
        }
        parse_field(p, spans)?;
        skip_newlines(p, spans);
        if matches!(
            p.peek().map(|t| &t.token),
            Some(LexerTokens::Symbol(LexerSymbol::Comma))
        ) {
            p.bump(SyntaxKind::COMMA, spans);
        }
    }
    Ok(())
}

/// `ident type ('=' expr)? | ident '=' expr` — field names use the `ident`
/// meta-rule (IDENT, or the contextual keywords `from`/`as`) so a field can
/// be named `from` or `as`; the lexer doesn't distinguish those from a
/// plain identifier anyway, so no special-casing is needed here.
fn parse_field(p: &mut TreeParser, spans: &mut Vec<CodeSpan>) -> Result<(), ElpParserError> {
    p.start_node(SyntaxKind::FIELD);

    let name_tok = p
        .peek()
        .cloned()
        .ok_or_else(|| expected_got(None, "a field name"))?;
    if !matches!(name_tok.token, LexerTokens::Identifier(_)) {
        return Err(expected_got(Some(&name_tok), "a field name"));
    }
    p.bump(SyntaxKind::IDENT, spans);

    if matches!(
        p.peek().map(|t| &t.token),
        Some(LexerTokens::Symbol(LexerSymbol::Equal))
    ) {
        p.bump(SyntaxKind::EQ, spans);
        expression::parse_binary(p, spans, 0)?;
    } else {
        ty::parse_type(p, spans)?;
        if matches!(
            p.peek().map(|t| &t.token),
            Some(LexerTokens::Symbol(LexerSymbol::Equal))
        ) {
            p.bump(SyntaxKind::EQ, spans);
            expression::parse_binary(p, spans, 0)?;
        }
    }

    p.finish_node(); // FIELD
    Ok(())
}

fn is_self_ident(tok: Option<&lexer::span::SpannedToken>) -> bool {
    matches!(tok.map(|t| &t.token), Some(LexerTokens::Identifier(i)) if i == "self")
}

fn is_const_ident(tok: Option<&lexer::span::SpannedToken>) -> bool {
    matches!(tok.map(|t| &t.token), Some(LexerTokens::Identifier(i)) if i == "const")
}

/// `param (',' param)* ','?`, delimited by the caller-consumed `(`/`)`.
pub(crate) fn parse_param_list(
    p: &mut TreeParser,
    spans: &mut Vec<CodeSpan>,
) -> Result<(), ElpParserError> {
    p.bump(SyntaxKind::LPAREN, spans);
    p.start_node(SyntaxKind::PARAM_LIST);

    let mut expect_param = true;
    loop {
        let Some(tok) = p.peek().cloned() else {
            return Err(expected_got(None, "')' to close parameter list"));
        };
        match &tok.token {
            LexerTokens::Symbol(LexerSymbol::RParen) => break,
            LexerTokens::Symbol(LexerSymbol::Comma) => {
                if expect_param {
                    return Err(unexpected(&tok, "unexpected ','"));
                }
                p.bump(SyntaxKind::COMMA, spans);
                expect_param = true;
            }
            LexerTokens::Newline => p.skip_trivia(spans),
            _ => {
                if !expect_param {
                    return Err(unexpected(&tok, "expected ',' or ')'"));
                }
                parse_param(p, spans)?;
                expect_param = false;
            }
        }
    }
    p.finish_node(); // PARAM_LIST
    p.bump(SyntaxKind::RPAREN, spans);
    Ok(())
}

/// `selfParam | CONST? ident type ('=' expr)? | CONST? type`. Disambiguating
/// the named form from the bare-type form (used by FFI-style declarations
/// with no param names at all) needs 2-token lookahead: `x User` is a name
/// followed by a type (both IDENTs); a bare `User` alone is followed by
/// `,`/`)` instead.
fn parse_param(p: &mut TreeParser, spans: &mut Vec<CodeSpan>) -> Result<(), ElpParserError> {
    let has_amp = matches!(
        p.peek().map(|t| &t.token),
        Some(LexerTokens::Symbol(LexerSymbol::Ampersand))
    );
    let self_after_amp = has_amp && is_self_ident(p.peek_at(2));
    let bare_self = !has_amp && is_self_ident(p.peek());

    if self_after_amp || bare_self {
        p.start_node(SyntaxKind::SELF_PARAM);
        if self_after_amp {
            p.bump(SyntaxKind::AMP, spans);
        }
        p.bump(SyntaxKind::SELF_KW, spans);
        p.finish_node();
        return Ok(());
    }

    p.start_node(SyntaxKind::PARAM);
    if is_const_ident(p.peek()) {
        p.bump(SyntaxKind::CONST_KW, spans);
    }

    let is_named = matches!(p.peek().map(|t| &t.token), Some(LexerTokens::Identifier(_)))
        && matches!(
            p.peek_at(2).map(|t| &t.token),
            Some(LexerTokens::Identifier(_))
        );

    if is_named {
        p.bump(SyntaxKind::IDENT, spans);
        ty::parse_type(p, spans)?;
        if matches!(
            p.peek().map(|t| &t.token),
            Some(LexerTokens::Symbol(LexerSymbol::Equal))
        ) {
            p.bump(SyntaxKind::EQ, spans);
            expression::parse_binary(p, spans, 0)?;
        }
    } else {
        ty::parse_type(p, spans)?;
    }

    p.finish_node(); // PARAM
    Ok(())
}

// ---------- object ----------

fn object_from_cursor(cursor: &mut TokenCursor) -> Result<Spanned<Decl>, ElpParserError> {
    let mut spans = vec![];
    let mut p = TreeParser::new(cursor);
    p.start_node(SyntaxKind::OBJECT_DECL);

    bump_optional_export(&mut p, &mut spans);
    p.bump(SyntaxKind::OBJECT_KW, &mut spans);

    let name_tok = p
        .peek()
        .cloned()
        .ok_or_else(|| expected_got(None, "an object name"))?;
    if !matches!(name_tok.token, LexerTokens::Identifier(_)) {
        return Err(expected_got(Some(&name_tok), "an object name"));
    }
    p.bump(SyntaxKind::IDENT, &mut spans);

    // The combined `object Foo implements A, B { ... }` form is retired:
    // conformance is now a separate `Foo implementing A + B` declaration.
    // Caught explicitly so existing code gets told what to write instead of
    // a bare "expected '{'".
    if matches!(p.peek().map(|t| &t.token), Some(LexerTokens::Identifier(i)) if i == "implements") {
        let name = match &name_tok.token {
            LexerTokens::Identifier(i) => i.as_str(),
            // Unreachable: the check above already rejected a non-identifier.
            _ => "TypeName",
        };
        return Err(expected_got(
            p.peek(),
            &format!(
                "'{{' to start object body — the combined `implements` form was removed; \
                 declare conformance separately as `{name} implementing A + B`"
            ),
        ));
    }

    let open = p
        .peek()
        .cloned()
        .ok_or_else(|| expected_got(None, "'{' to start object body"))?;
    if !matches!(open.token, LexerTokens::Symbol(LexerSymbol::LCurley)) {
        return Err(expected_got(Some(&open), "'{' to start object body"));
    }
    p.bump(SyntaxKind::LCURLY, &mut spans);

    p.start_node(SyntaxKind::FIELD_LIST);
    parse_field_list(&mut p, &mut spans, LexerSymbol::RCurley)?;
    p.finish_node(); // FIELD_LIST

    p.bump(SyntaxKind::RCURLY, &mut spans);
    p.finish_node(); // OBJECT_DECL

    let root = p.finish();
    Ok(Spanned {
        node: Decl::Object(ObjectDecl::cast(root).expect("builder rooted this at OBJECT_DECL")),
        span: spans_extent(spans)?,
    })
}

// ---------- callback props (`*fn`) ----------

/// `EXPORT '*' FN IDENT '(' paramList? ')' ('->' type)?` — a callback slot,
/// signature only, no body. Statement-position only (component bodies are
/// the intended home), so it's dispatched from `parse_statement` rather
/// than from `Decl::from_cursor`.
///
/// The parser deliberately accepts this in *any* block. It can't tell a
/// component from a plain function — a component is just a `fn` returning
/// `Component` — so "props only exist in components" is a semantic check
/// for a later pass, not something the grammar can express. "Callbacks
/// must be exported" *is* expressible here, so it's enforced below.
pub(crate) fn parse_callback_decl(
    p: &mut TreeParser,
    spans: &mut Vec<CodeSpan>,
) -> Result<(), ElpParserError> {
    p.start_node(SyntaxKind::CALLBACK_DECL);

    // `export` is mandatory here, unlike everywhere else it appears. A
    // callback slot can only ever be filled from outside the component, so
    // a private one has nothing that could populate it — it can never hold
    // a value, which makes it both meaningless and trivially optimisable
    // away. Rejected at the door rather than parsed into a node a later
    // pass has to throw out.
    if !matches!(p.peek().map(|t| &t.token), Some(LexerTokens::Identifier(i)) if i == "export") {
        return Err(expected_got(
            p.peek(),
            "'export' before '*fn' — a callback can only be filled from outside, so a private \
             one could never hold a value; every callback is a prop",
        ));
    }
    p.bump(SyntaxKind::EXPORT_KW, spans);
    p.bump(SyntaxKind::STAR, spans);

    if !matches!(p.peek().map(|t| &t.token), Some(LexerTokens::Identifier(i)) if i == "fn") {
        return Err(expected_got(p.peek(), "'fn' after '*'"));
    }
    p.bump(SyntaxKind::FN_KW, spans);

    let name_tok = p
        .peek()
        .cloned()
        .ok_or_else(|| expected_got(None, "a callback name"))?;
    if !matches!(name_tok.token, LexerTokens::Identifier(_)) {
        return Err(expected_got(Some(&name_tok), "a callback name"));
    }
    p.bump(SyntaxKind::IDENT, spans);

    if matches!(
        p.peek().map(|t| &t.token),
        Some(LexerTokens::Symbol(LexerSymbol::LParen))
    ) {
        parse_param_list(p, spans)?;
    }
    if matches!(
        p.peek().map(|t| &t.token),
        Some(LexerTokens::Symbol(LexerSymbol::Arrow))
    ) {
        p.bump(SyntaxKind::ARROW, spans);
        ty::parse_type(p, spans)?;
    }

    p.finish_node(); // CALLBACK_DECL
    Ok(())
}

// ---------- implementing ----------

/// `TypeName implementing Interface ('+' Interface)*` — a standalone
/// conformance marker. No block: the method bodies that satisfy the
/// interfaces are ordinary dotted free functions elsewhere in the package
/// (`fn TypeName.method(...) -> ...`), and file location carries no
/// compiler meaning, so there is nothing for a block to usefully contain.
fn implementing_from_cursor(cursor: &mut TokenCursor) -> Result<Spanned<Decl>, ElpParserError> {
    let mut spans = vec![];
    let mut p = TreeParser::new(cursor);
    p.start_node(SyntaxKind::IMPLEMENTING_DECL);

    // Deliberately no `bump_optional_export` — conformance isn't separately
    // exportable, it travels with the type it's declared for. Rejected
    // explicitly rather than ignored: the dispatch lookahead skips `export`
    // when hunting for the keyword, so without this an
    // `export Foo implementing Bar` would arrive here still pointing at
    // `export` and silently build a tree claiming a type called `export`
    // implements `Foo`.
    if matches!(p.peek().map(|t| &t.token), Some(LexerTokens::Identifier(i)) if i == "export") {
        return Err(expected_got(
            p.peek(),
            "a type name — an `implementing` declaration can't be exported, \
             conformance travels with the type it's declared for",
        ));
    }

    let name_tok = p
        .peek()
        .cloned()
        .ok_or_else(|| expected_got(None, "a type name"))?;
    if !matches!(name_tok.token, LexerTokens::Identifier(_)) {
        return Err(expected_got(Some(&name_tok), "a type name"));
    }
    p.bump(SyntaxKind::IDENT, &mut spans);

    // `bump` doesn't validate, and this is only ever reached via a
    // two-token lookahead that already saw `implementing` — but that
    // lookahead counts from the pre-`export` position, so re-check here
    // rather than trusting it.
    if !is_implementing_kw(p.peek()) {
        return Err(expected_got(p.peek(), "'implementing'"));
    }
    p.bump(SyntaxKind::IMPLEMENTING_KW, &mut spans);

    parse_interface_list(&mut p, &mut spans)?;

    p.finish_node(); // IMPLEMENTING_DECL

    let root = p.finish();
    Ok(Spanned {
        node: Decl::Implementing(
            ImplementingDecl::cast(root).expect("builder rooted this at IMPLEMENTING_DECL"),
        ),
        span: spans_extent(spans)?,
    })
}

/// `type ('+' type)*` — the same conjunction shape generic bounds use.
/// Newlines are allowed after a `+` so a long conformance list can wrap;
/// unambiguous because a `+` always has to be followed by another interface.
fn parse_interface_list(
    p: &mut TreeParser,
    spans: &mut Vec<CodeSpan>,
) -> Result<(), ElpParserError> {
    p.start_node(SyntaxKind::INTERFACE_LIST);
    ty::parse_type(p, spans)?;
    while matches!(
        p.peek().map(|t| &t.token),
        Some(LexerTokens::Symbol(LexerSymbol::Plus))
    ) {
        p.bump(SyntaxKind::PLUS, spans);
        skip_newlines(p, spans);
        ty::parse_type(p, spans)?;
    }
    p.finish_node(); // INTERFACE_LIST
    Ok(())
}

// ---------- interface ----------

fn interface_from_cursor(cursor: &mut TokenCursor) -> Result<Spanned<Decl>, ElpParserError> {
    let mut spans = vec![];
    let mut p = TreeParser::new(cursor);
    p.start_node(SyntaxKind::INTERFACE_DECL);

    bump_optional_export(&mut p, &mut spans);
    p.bump(SyntaxKind::INTERFACE_KW, &mut spans);

    let name_tok = p
        .peek()
        .cloned()
        .ok_or_else(|| expected_got(None, "an interface name"))?;
    if !matches!(name_tok.token, LexerTokens::Identifier(_)) {
        return Err(expected_got(Some(&name_tok), "an interface name"));
    }
    p.bump(SyntaxKind::IDENT, &mut spans);

    let open = p
        .peek()
        .cloned()
        .ok_or_else(|| expected_got(None, "'{' to start interface body"))?;
    if !matches!(open.token, LexerTokens::Symbol(LexerSymbol::LCurley)) {
        return Err(expected_got(Some(&open), "'{' to start interface body"));
    }
    p.bump(SyntaxKind::LCURLY, &mut spans);

    p.start_node(SyntaxKind::INTERFACE_BODY);
    loop {
        skip_newlines(&mut p, &mut spans);
        if matches!(
            p.peek().map(|t| &t.token),
            Some(LexerTokens::Symbol(LexerSymbol::RCurley))
        ) {
            break;
        }
        if p.at_end() {
            return Err(expected_got(None, "a member or the closing '}'"));
        }

        // Interfaces declare methods only. Field requirements are retired:
        // anything that was a field becomes a required method, which the
        // implementing type satisfies however it likes internally. Caught
        // explicitly, with the rewrite spelled out, rather than letting a
        // field parse into a contract nothing can check.
        if !matches!(p.peek().map(|t| &t.token), Some(LexerTokens::Identifier(i)) if i == "fn") {
            let field_name = match p.peek().map(|t| &t.token) {
                Some(LexerTokens::Identifier(i)) => i.clone(),
                other => return Err(expected_got(p.peek(), &format!("a method, got {other:?}"))),
            };
            return Err(expected_got(
                p.peek(),
                &format!(
                    "a method — interfaces declare methods only, field requirements were \
                     removed; write `fn {field_name}() -> T` instead of `{field_name} T`"
                ),
            ));
        }
        parse_interface_method(&mut p, &mut spans)?;

        skip_newlines(&mut p, &mut spans);
        if matches!(
            p.peek().map(|t| &t.token),
            Some(LexerTokens::Symbol(LexerSymbol::Comma))
        ) {
            p.bump(SyntaxKind::COMMA, &mut spans);
        }
    }
    p.finish_node(); // INTERFACE_BODY

    p.bump(SyntaxKind::RCURLY, &mut spans);
    p.finish_node(); // INTERFACE_DECL

    let root = p.finish();
    Ok(Spanned {
        node: Decl::Interface(
            InterfaceDecl::cast(root).expect("builder rooted this at INTERFACE_DECL"),
        ),
        span: spans_extent(spans)?,
    })
}

fn parse_interface_method(
    p: &mut TreeParser,
    spans: &mut Vec<CodeSpan>,
) -> Result<(), ElpParserError> {
    p.start_node(SyntaxKind::INTERFACE_METHOD);
    p.bump(SyntaxKind::FN_KW, spans);

    let name_tok = p
        .peek()
        .cloned()
        .ok_or_else(|| expected_got(None, "a method name"))?;
    if !matches!(name_tok.token, LexerTokens::Identifier(_)) {
        return Err(expected_got(Some(&name_tok), "a method name"));
    }
    p.bump(SyntaxKind::IDENT, spans);

    if matches!(
        p.peek().map(|t| &t.token),
        Some(LexerTokens::Symbol(LexerSymbol::LParen))
    ) {
        parse_param_list(p, spans)?;
    }
    if matches!(
        p.peek().map(|t| &t.token),
        Some(LexerTokens::Symbol(LexerSymbol::Arrow))
    ) {
        p.bump(SyntaxKind::ARROW, spans);
        ty::parse_type(p, spans)?;
    }

    p.finish_node(); // INTERFACE_METHOD
    Ok(())
}

// ---------- enum ----------

fn enum_from_cursor(cursor: &mut TokenCursor) -> Result<Spanned<Decl>, ElpParserError> {
    let mut spans = vec![];
    let mut p = TreeParser::new(cursor);
    p.start_node(SyntaxKind::ENUM_DECL);

    bump_optional_export(&mut p, &mut spans);
    p.bump(SyntaxKind::ENUM_KW, &mut spans);

    let name_tok = p
        .peek()
        .cloned()
        .ok_or_else(|| expected_got(None, "an enum name"))?;
    if !matches!(name_tok.token, LexerTokens::Identifier(_)) {
        return Err(expected_got(Some(&name_tok), "an enum name"));
    }
    p.bump(SyntaxKind::IDENT, &mut spans);

    let open = p
        .peek()
        .cloned()
        .ok_or_else(|| expected_got(None, "'{' to start enum body"))?;
    if !matches!(open.token, LexerTokens::Symbol(LexerSymbol::LCurley)) {
        return Err(expected_got(Some(&open), "'{' to start enum body"));
    }
    p.bump(SyntaxKind::LCURLY, &mut spans);

    p.start_node(SyntaxKind::ENUM_VARIANTS);
    loop {
        skip_newlines(&mut p, &mut spans);
        if matches!(
            p.peek().map(|t| &t.token),
            Some(LexerTokens::Symbol(LexerSymbol::RCurley))
        ) {
            break;
        }
        if p.at_end() {
            return Err(expected_got(None, "a variant or the closing '}'"));
        }
        parse_variant(&mut p, &mut spans)?;

        skip_newlines(&mut p, &mut spans);
        if matches!(
            p.peek().map(|t| &t.token),
            Some(LexerTokens::Symbol(LexerSymbol::Comma))
        ) {
            p.bump(SyntaxKind::COMMA, &mut spans);
        }
    }
    p.finish_node(); // ENUM_VARIANTS

    p.bump(SyntaxKind::RCURLY, &mut spans);
    p.finish_node(); // ENUM_DECL

    let root = p.finish();
    Ok(Spanned {
        node: Decl::Enum(EnumDecl::cast(root).expect("builder rooted this at ENUM_DECL")),
        span: spans_extent(spans)?,
    })
}

fn parse_variant(p: &mut TreeParser, spans: &mut Vec<CodeSpan>) -> Result<(), ElpParserError> {
    p.start_node(SyntaxKind::VARIANT);

    let name_tok = p
        .peek()
        .cloned()
        .ok_or_else(|| expected_got(None, "a variant name"))?;
    if !matches!(name_tok.token, LexerTokens::Identifier(_)) {
        return Err(expected_got(Some(&name_tok), "a variant name"));
    }
    p.bump(SyntaxKind::IDENT, spans);

    if matches!(
        p.peek().map(|t| &t.token),
        Some(LexerTokens::Symbol(LexerSymbol::LParen))
    ) {
        p.bump(SyntaxKind::LPAREN, spans);
        parse_type_list(p, spans)?;
        let close = p
            .peek()
            .cloned()
            .ok_or_else(|| expected_got(None, "')' to close variant payload"))?;
        if matches!(close.token, LexerTokens::Symbol(LexerSymbol::RParen)) {
            p.bump(SyntaxKind::RPAREN, spans);
        } else {
            return Err(expected_got(Some(&close), "')' to close variant payload"));
        }
    }

    p.finish_node(); // VARIANT
    Ok(())
}

// ---------- fn ----------

fn fn_from_cursor(cursor: &mut TokenCursor) -> Result<Spanned<Decl>, ElpParserError> {
    let mut spans = vec![];
    let mut p = TreeParser::new(cursor);
    p.start_node(SyntaxKind::FN_DECL);

    bump_optional_export(&mut p, &mut spans);
    if matches!(p.peek().map(|t| &t.token), Some(LexerTokens::Identifier(i)) if i == "pure") {
        p.bump(SyntaxKind::PURE_KW, &mut spans);
    }
    p.bump(SyntaxKind::FN_KW, &mut spans);

    let name_tok = p
        .peek()
        .cloned()
        .ok_or_else(|| expected_got(None, "a function name"))?;
    if !matches!(name_tok.token, LexerTokens::Identifier(_)) {
        return Err(expected_got(Some(&name_tok), "a function name"));
    }
    p.bump(SyntaxKind::IDENT, &mut spans);

    // fnName: IDENT ('.' ident)? — second segment allows the contextual
    // keywords `from`/`as` as method names (`ApiError.from`).
    if matches!(
        p.peek().map(|t| &t.token),
        Some(LexerTokens::Symbol(LexerSymbol::Period))
    ) {
        p.bump(SyntaxKind::DOT, &mut spans);
        let method_tok = p
            .peek()
            .cloned()
            .ok_or_else(|| expected_got(None, "a method name after '.'"))?;
        if matches!(method_tok.token, LexerTokens::Identifier(_)) {
            p.bump(SyntaxKind::IDENT, &mut spans);
        } else {
            return Err(expected_got(Some(&method_tok), "a method name after '.'"));
        }
    }

    if matches!(
        p.peek().map(|t| &t.token),
        Some(LexerTokens::Symbol(LexerSymbol::LParen))
    ) {
        parse_param_list(&mut p, &mut spans)?;
    }

    if matches!(
        p.peek().map(|t| &t.token),
        Some(LexerTokens::Symbol(LexerSymbol::Arrow))
    ) {
        p.bump(SyntaxKind::ARROW, &mut spans);
        ty::parse_type(&mut p, &mut spans)?;
    }

    // block? — body-less is legal (FFI forward declarations).
    if matches!(
        p.peek().map(|t| &t.token),
        Some(LexerTokens::Symbol(LexerSymbol::LCurley))
    ) {
        statement::parse_block(&mut p, &mut spans)?;
    }

    p.finish_node(); // FN_DECL

    let root = p.finish();
    Ok(Spanned {
        node: Decl::Fn(FnDecl::cast(root).expect("builder rooted this at FN_DECL")),
        span: spans_extent(spans)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use lexer::ElpLexer;
    use pretty_assertions::assert_eq;

    #[test]
    fn from_cursor_on_exhausted_cursor_errors_cleanly() {
        // Defensive check for a case the real dispatch (gated by
        // `matches_ident`) never produces on its own — the cursor is
        // already past every token before `Decl::from_cursor` runs.
        let tokens = vec![];
        let mut cursor = TokenCursor::new(&tokens);
        let err = Decl::from_cursor(&mut cursor).unwrap_err();
        assert!(
            format!("{err}").contains("No token to peek at"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn from_cursor_with_a_non_keyword_identifier_errors_cleanly() {
        // `Decl::from_cursor` called directly (bypassing the
        // `matches_ident`-gated top-level dispatch that normally prevents
        // this) with an identifier that isn't a recognized declaration
        // keyword. `matches_ident` and this function's `match
        // keyword.as_str()` are meant to stay in sync — every keyword the
        // former accepts, the latter has an arm for — so the real
        // dispatch never reaches this fallback; only a direct,
        // out-of-band call does.
        let mut lexer = ElpLexer::new("notAKeyword");
        let tokens = lexer.scan();
        let mut cursor = TokenCursor::new(&tokens);
        let err = Decl::from_cursor(&mut cursor).unwrap_err();
        assert!(
            format!("{err}").contains("Unexpected token"),
            "unexpected error: {err}"
        );
    }

    fn parse_import(src: &str) -> Result<Spanned<Decl>, ElpParserError> {
        let mut lexer = ElpLexer::new(src);
        let tokens = lexer.scan();
        let mut cursor = TokenCursor::new(&tokens);
        import_from_cursor(&mut cursor)
    }

    fn unwrap_import(decl: Decl) -> (Vec<ImportNameData>, Vec<String>, bool) {
        match decl {
            Decl::Import(stmt) => (stmt.names(), stmt.module_path(), stmt.is_relative()),
            other => panic!("expected Decl::Import, got {other:?}"),
        }
    }

    #[test]
    fn single_named_import() {
        let result = parse_import("import { test } from my.module").unwrap();
        let (names, path, relative) = unwrap_import(result.node);
        assert_eq!(names, vec![ImportNameData::plain("test")]);
        assert_eq!(path, vec!["my".to_string(), "module".to_string()]);
        assert!(!relative);
        assert_eq!(result.span.start, 0);
        assert_eq!(result.span.end, 30);
        assert_eq!(result.span.line, 1);
    }

    #[test]
    fn multiple_named_imports() {
        let result = parse_import("import { User, Message, Actions } from my.module").unwrap();
        let (names, path, relative) = unwrap_import(result.node);
        assert_eq!(
            names,
            vec![
                ImportNameData::plain("User"),
                ImportNameData::plain("Message"),
                ImportNameData::plain("Actions"),
            ]
        );
        assert_eq!(path, vec!["my".to_string(), "module".to_string()]);
        assert!(!relative);
    }

    #[test]
    fn import_with_alias() {
        let result = parse_import("import { Message as MessageView } from my.module").unwrap();
        let (names, _, _) = unwrap_import(result.node);
        assert_eq!(
            names,
            vec![ImportNameData::aliased("Message", "MessageView")]
        );
    }

    #[test]
    fn import_mixed_aliases() {
        let result =
            parse_import("import { User, Message as MessageView, Actions } from my.module")
                .unwrap();
        let (names, _, _) = unwrap_import(result.node);
        assert_eq!(
            names,
            vec![
                ImportNameData::plain("User"),
                ImportNameData::aliased("Message", "MessageView"),
                ImportNameData::plain("Actions"),
            ]
        );
    }

    #[test]
    fn multiline_import_block() {
        let src = "import {
    OBJC,
    OBJC_Obj,
    Error,
    Either
} from elp.types";
        let result = parse_import(src).unwrap();
        let (names, path, _) = unwrap_import(result.node);
        assert_eq!(
            names,
            vec![
                ImportNameData::plain("OBJC"),
                ImportNameData::plain("OBJC_Obj"),
                ImportNameData::plain("Error"),
                ImportNameData::plain("Either"),
            ]
        );
        assert_eq!(path, vec!["elp".to_string(), "types".to_string()]);
    }

    #[test]
    fn relative_import_leading_dot() {
        let result = parse_import("import { Paragraph } from .components.paragraph").unwrap();
        let (names, path, relative) = unwrap_import(result.node);
        assert_eq!(names, vec![ImportNameData::plain("Paragraph")]);
        assert_eq!(
            path,
            vec!["components".to_string(), "paragraph".to_string()]
        );
        assert!(relative);
    }

    #[test]
    fn deep_dotted_module_path() {
        let result = parse_import("import { Text } from elp.app.ui.elements.Text").unwrap();
        let (_, path, relative) = unwrap_import(result.node);
        assert_eq!(
            path,
            vec![
                "elp".to_string(),
                "app".to_string(),
                "ui".to_string(),
                "elements".to_string(),
                "Text".to_string(),
            ]
        );
        assert!(!relative);
    }

    #[test]
    fn multiple_imports_with_newlines_between() {
        let src = "import { a } from x.y
import { b } from x.y
import { c } from x.y";
        let mut lexer = ElpLexer::new(src);
        let tokens = lexer.scan();
        let mut cursor = TokenCursor::new(&tokens);

        for expected in ["a", "b", "c"] {
            let result = import_from_cursor(&mut cursor).unwrap();
            let (names, path, _) = unwrap_import(result.node);
            assert_eq!(names, vec![ImportNameData::plain(expected)]);
            assert_eq!(path, vec!["x".to_string(), "y".to_string()]);

            // Advance past the trailing newline, if any.
            if let Some(tok) = cursor.peek()
                && matches!(tok.token, LexerTokens::Newline)
            {
                cursor.next_token();
            }
        }
    }

    #[test]
    fn top_level_dispatches_to_import() {
        let mut lexer = ElpLexer::new("import { a } from x.y");
        let tokens = lexer.scan();
        let mut cursor = TokenCursor::new(&tokens);
        let result = Decl::from_cursor(&mut cursor).unwrap();
        let (names, path, _) = unwrap_import(result.node);
        assert_eq!(names, vec![ImportNameData::plain("a")]);
        assert_eq!(path, vec!["x".to_string(), "y".to_string()]);
    }

    #[test]
    fn error_missing_from() {
        let err = parse_import("import { a } my.module").unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("from"), "unexpected error: {msg}");
    }

    #[test]
    fn error_missing_module_path() {
        let err = parse_import("import { a } from").unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("module path"), "unexpected error: {msg}");
    }

    #[test]
    fn error_unclosed_brace() {
        let err = parse_import("import { a, b").unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("'}'"), "unexpected error: {msg}");
    }

    #[test]
    fn error_leading_comma() {
        let err = parse_import("import { , a } from x.y").unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains(","), "unexpected error: {msg}");
    }

    #[test]
    fn error_two_names_without_comma() {
        let err = parse_import("import { a b } from x.y").unwrap_err();
        let msg = format!("{}", err);
        assert!(
            msg.contains("','") || msg.contains("}"),
            "unexpected error: {msg}"
        );
    }

    #[test]
    fn error_missing_alias_after_as() {
        let err = parse_import("import { a as } from x.y").unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("alias"), "unexpected error: {msg}");
    }

    #[test]
    fn trailing_comma_allowed() {
        // Permissive behaviour: `{ a, b, }` parses as two names.
        let result = parse_import("import { a, b, } from x.y").unwrap();
        let (names, _, _) = unwrap_import(result.node);
        assert_eq!(
            names,
            vec![ImportNameData::plain("a"), ImportNameData::plain("b")]
        );
    }

    #[test]
    fn syntax_tree_round_trips_source_text_except_undertyped_trivia() {
        // Whitespace/newline round-tripping is a known, deliberate gap
        // (see TreeParser docs) until the lexer emits whitespace tokens.
        // This test locks in what *is* preserved today: every meaningful
        // token's exact text, in order.
        let result = parse_import("import { test as alias } from my.module").unwrap();
        let Decl::Import(stmt) = result.node else {
            panic!("expected Decl::Import")
        };
        let tokens: Vec<String> = stmt
            .syntax()
            .descendants_with_tokens()
            .filter_map(|el| el.into_token())
            .map(|t| t.text().to_string())
            .collect();
        assert_eq!(
            tokens,
            vec![
                "import", "{", "test", "as", "alias", "}", "from", "my", ".", "module"
            ]
        );
    }

    fn parse_decl(src: &str) -> Result<Spanned<Decl>, ElpParserError> {
        let mut lexer = ElpLexer::new(src);
        let tokens = lexer.scan();
        let mut cursor = TokenCursor::new(&tokens);
        Decl::from_cursor(&mut cursor)
    }

    fn type_segments(ty: Option<Type>) -> Vec<String> {
        match ty {
            Some(Type::Named(n)) => n.segments(),
            other => panic!("expected a named type, got {other:?}"),
        }
    }

    // ---------- var / const ----------

    /// `var` is block-only now, so its tests parse through block context
    /// rather than `Decl::from_cursor`. Returns the block's first statement.
    fn parse_stmt_in_block(src: &str) -> statement::Statement {
        let mut lexer = ElpLexer::new(src);
        let tokens = lexer.scan();
        let mut cursor = TokenCursor::new(&tokens);
        let mut spans = vec![];
        let mut p = TreeParser::new(&mut cursor);
        statement::parse_block(&mut p, &mut spans).unwrap_or_else(|e| panic!("{src:?}: {e}"));
        let block = statement::Block::cast(p.finish()).expect("rooted at BLOCK");
        block
            .statements()
            .into_iter()
            .next()
            .unwrap_or_else(|| panic!("{src:?}: no statements"))
    }

    #[test]
    fn simple_var_decl() {
        let statement::Statement::Var(v) = parse_stmt_in_block("{ var x }") else {
            panic!("expected a var statement")
        };
        assert_eq!(v.pattern(), vec!["x".to_string()]);
        assert_eq!(v.ty(), None);
        assert_eq!(v.value(), None);
        assert!(!v.is_exported());
    }

    #[test]
    fn var_decl_with_type_and_initializer() {
        let statement::Statement::Var(v) = parse_stmt_in_block("{ var name string = \"bob\" }")
        else {
            panic!("expected a var statement")
        };
        assert_eq!(v.pattern(), vec!["name".to_string()]);
        assert_eq!(type_segments(v.ty()), vec!["string".to_string()]);
        assert!(matches!(v.value(), Some(Expr::Literal(_))));
    }

    #[test]
    fn const_decl_multi_binding_destructure() {
        // `const` is still a declaration at file scope, so this one keeps
        // going through `Decl::from_cursor`.
        let Decl::Const(c) = parse_decl("const result, err = getThing()").unwrap().node else {
            panic!("expected Decl::Const")
        };
        assert_eq!(c.pattern(), vec!["result".to_string(), "err".to_string()]);
        assert!(matches!(c.value(), Some(Expr::Call(_))));
    }

    #[test]
    fn error_var_decl_missing_name() {
        let mut lexer = ElpLexer::new("{ var = 1 }");
        let tokens = lexer.scan();
        let mut cursor = TokenCursor::new(&tokens);
        let mut spans = vec![];
        let mut p = TreeParser::new(&mut cursor);
        let err = statement::parse_block(&mut p, &mut spans).unwrap_err();
        assert!(
            format!("{err}").contains("binding name"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn error_top_level_var_is_rejected() {
        // elp has no package-level variables; the error should say so and
        // point at `const`, not just fail somewhere downstream.
        let err = parse_decl("var x = 1").unwrap_err();
        let msg = format!("{err}");
        assert!(
            msg.contains("no package-level variables"),
            "unexpected error: {msg}"
        );
        assert!(msg.contains("const"), "should point at const: {msg}");
    }

    #[test]
    fn top_level_const_is_still_a_declaration() {
        // The corpus depends on this — `BlogStyle/constants.elp` is a
        // file-scope `export const`, and a compile-time constant isn't the
        // package-level *state* the `var` rejection is aimed at.
        let Decl::Const(c) = parse_decl("export const FONT = \"./f.otf\"")
            .unwrap_or_else(|e| panic!("{e}"))
            .node
        else {
            panic!("expected Decl::Const")
        };
        assert!(c.is_exported());
        assert_eq!(c.pattern(), vec!["FONT".to_string()]);
    }

    // ---------- object ----------

    #[test]
    fn object_decl_with_fields() {
        let Decl::Object(o) = parse_decl(
            "object ChatAppState {
    messages [Message],
    members  [User],
    loggedIn User?
}",
        )
        .unwrap()
        .node
        else {
            panic!("expected Decl::Object")
        };
        assert_eq!(o.name(), "ChatAppState");
        let fields = o.fields();
        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0].name(), "messages");
        assert!(matches!(fields[0].ty(), Some(Type::Array(_))));
        assert_eq!(fields[2].name(), "loggedIn");
        assert!(matches!(fields[2].ty(), Some(Type::Optional(_))));
    }

    #[test]
    fn object_decl_with_field_default() {
        let Decl::Object(o) = parse_decl("export object ChatAppState { messages = [] }")
            .unwrap()
            .node
        else {
            panic!("expected Decl::Object")
        };
        assert!(o.is_exported());
        let fields = o.fields();
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].name(), "messages");
        assert!(matches!(
            fields[0].default_value(),
            Some(Expr::ArrayLiteral(_))
        ));
    }

    #[test]
    fn error_object_decl_rejects_the_retired_implements_form() {
        // The combined form is gone; the error should say what to write
        // instead rather than just complaining about a missing '{'.
        let err = parse_decl("object ChatAppState implements Default { }").unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("was removed"), "unexpected error: {msg}");
        assert!(
            msg.contains("ChatAppState implementing A + B"),
            "the suggestion should name the actual type: {msg}"
        );
    }

    #[test]
    fn object_decl_fields_newline_separated_no_commas() {
        let Decl::Object(o) = parse_decl(
            "object Point {
    x float
    y float
}",
        )
        .unwrap()
        .node
        else {
            panic!("expected Decl::Object")
        };
        assert_eq!(o.fields().len(), 2);
    }

    #[test]
    fn error_object_decl_missing_brace() {
        let err = parse_decl("object Foo").unwrap_err();
        assert!(format!("{err}").contains("'{'"), "unexpected error: {err}");
    }

    // ---------- implementing ----------

    fn parse_implementing(src: &str) -> ImplementingDecl {
        let Decl::Implementing(i) = parse_decl(src).unwrap_or_else(|e| panic!("{e}")).node else {
            panic!("expected Decl::Implementing for {src:?}")
        };
        i
    }

    #[test]
    fn implementing_decl_single_interface() {
        let i = parse_implementing("ChatState implementing State");
        assert_eq!(i.type_name(), "ChatState");
        let interfaces = i.interfaces();
        assert_eq!(interfaces.len(), 1);
        assert_eq!(
            type_segments(interfaces.into_iter().next()),
            vec!["State".to_string()]
        );
    }

    #[test]
    fn implementing_decl_plus_separated_list() {
        let i = parse_implementing("ChatState implementing State + Copy + Equatable");
        assert_eq!(i.type_name(), "ChatState");
        let names: Vec<Vec<String>> = i
            .interfaces()
            .into_iter()
            .map(|t| type_segments(Some(t)))
            .collect();
        assert_eq!(
            names,
            vec![
                vec!["State".to_string()],
                vec!["Copy".to_string()],
                vec!["Equatable".to_string()],
            ]
        );
    }

    #[test]
    fn implementing_decl_list_may_wrap_after_a_plus() {
        let i = parse_implementing("ApiError implementing Error +\n    Copy");
        assert_eq!(i.interfaces().len(), 2);
    }

    #[test]
    fn implementing_decl_accepts_a_dotted_interface_name() {
        // Interfaces are ordinary type references, so a namespaced one
        // (`elp.ui.Component`) has to work the same as a bare name.
        let i = parse_implementing("Login implementing elp.ui.Component");
        assert_eq!(
            type_segments(i.interfaces().into_iter().next()),
            vec!["elp".to_string(), "ui".to_string(), "Component".to_string()]
        );
    }

    #[test]
    fn error_implementing_decl_with_no_interface() {
        let err = parse_decl("ChatState implementing").unwrap_err();
        assert!(!format!("{err}").is_empty());
    }

    #[test]
    fn error_implementing_decl_trailing_plus() {
        let err = parse_decl("ChatState implementing State +").unwrap_err();
        assert!(!format!("{err}").is_empty());
    }

    #[test]
    fn error_implementing_decl_cannot_be_exported() {
        // Dispatch's keyword lookahead skips `export`, so without an
        // explicit guard this parsed as "a type named `export` implements
        // `ChatState`" — a silently wrong tree, not an error.
        let err = parse_decl("export ChatState implementing State").unwrap_err();
        assert!(
            format!("{err}").contains("can't be exported"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn object_named_implementing_still_parses_as_an_object() {
        // `implementing` is contextual, not reserved. Dispatch checks the
        // declaration keyword first precisely so this stays an object.
        let Decl::Object(o) = parse_decl("object implementing { x float }").unwrap().node else {
            panic!("expected Decl::Object")
        };
        assert_eq!(o.name(), "implementing");
    }

    // ---------- interface ----------

    #[test]
    fn interface_decl_with_methods() {
        let Decl::Interface(i) = parse_decl(
            "interface LoginProps {
    fn email() -> string?,
    fn validate() -> bool
}",
        )
        .unwrap()
        .node
        else {
            panic!("expected Decl::Interface")
        };
        assert_eq!(i.name(), "LoginProps");
        let methods = i.methods();
        assert_eq!(methods.len(), 2);
        assert_eq!(methods[0].name(), "email");
        assert_eq!(methods[1].name(), "validate");
        assert_eq!(
            type_segments(methods[1].return_type()),
            vec!["bool".to_string()]
        );
    }

    #[test]
    fn error_interface_decl_rejects_a_field_requirement() {
        // Interfaces are method-only; a field requirement should say what
        // to write instead rather than parsing into an uncheckable contract.
        let err = parse_decl("interface LoginProps { email string? }").unwrap_err();
        let msg = format!("{err}");
        assert!(
            msg.contains("field requirements were removed"),
            "unexpected error: {msg}"
        );
        assert!(
            msg.contains("fn email() -> T"),
            "the suggestion should name the actual field: {msg}"
        );
    }

    #[test]
    fn interface_decl_method_without_parens_is_still_a_method() {
        // `fn new -> Self` (no parameter list at all) has to keep working —
        // the field-requirement rejection keys off the `fn`, not the parens.
        let Decl::Interface(i) = parse_decl("interface State { fn new -> Self }")
            .unwrap()
            .node
        else {
            panic!("expected Decl::Interface")
        };
        assert_eq!(i.methods().len(), 1);
        assert_eq!(i.methods()[0].name(), "new");
    }

    // ---------- enum ----------

    #[test]
    fn enum_decl_with_payload_variants() {
        let Decl::Enum(e) = parse_decl(
            "export enum AppActions {
    AUTHENTICATED_ISH(bool),
    LOADING(bool),
    LOG_IN(string),
    NEW_MESSAGE(string, string),
}",
        )
        .unwrap()
        .node
        else {
            panic!("expected Decl::Enum")
        };
        assert!(e.is_exported());
        assert_eq!(e.name(), "AppActions");
        let variants = e.variants();
        assert_eq!(variants.len(), 4);
        assert_eq!(variants[0].name(), "AUTHENTICATED_ISH");
        assert_eq!(
            type_segments(variants[0].payload_types().into_iter().next()),
            vec!["bool".to_string()]
        );
        assert_eq!(variants[3].payload_types().len(), 2);
    }

    #[test]
    fn enum_decl_bare_variant_no_payload() {
        let Decl::Enum(e) = parse_decl("enum LoginError { NO_HANDLER, BAD_REQUEST(string) }")
            .unwrap()
            .node
        else {
            panic!("expected Decl::Enum")
        };
        let variants = e.variants();
        assert_eq!(variants[0].name(), "NO_HANDLER");
        assert_eq!(variants[0].payload_types().len(), 0);
        assert_eq!(variants[1].payload_types().len(), 1);
    }

    // ---------- fn ----------

    #[test]
    fn fn_decl_signature_only_no_body() {
        let Decl::Fn(f) = parse_decl("fn OBJC_msgSend(T, OBJC_Selector) -> T")
            .unwrap()
            .node
        else {
            panic!("expected Decl::Fn")
        };
        assert_eq!(f.name(), "OBJC_msgSend");
        assert!(!f.has_body());
        let params = f.params();
        assert_eq!(params.len(), 2);
        // Bare-type params (no name) — matches @external FFI declarations.
        assert_eq!(params[0].name(), None);
        assert_eq!(type_segments(params[0].ty()), vec!["T".to_string()]);
        assert_eq!(type_segments(f.return_type()), vec!["T".to_string()]);
    }

    #[test]
    fn fn_decl_named_params_and_body() {
        let Decl::Fn(f) = parse_decl(
            "export fn HelloWorld(name string, count int = 1) -> App {
    println(\"hello\")
}",
        )
        .unwrap()
        .node
        else {
            panic!("expected Decl::Fn")
        };
        assert!(f.is_exported());
        assert_eq!(f.name(), "HelloWorld");
        assert!(f.has_body());
        let params = f.params();
        assert_eq!(params.len(), 2);
        assert_eq!(params[0].name(), Some("name".to_string()));
        assert_eq!(type_segments(params[0].ty()), vec!["string".to_string()]);
        assert_eq!(params[1].name(), Some("count".to_string()));
        assert!(matches!(params[1].default_value(), Some(Expr::Literal(_))));
    }

    #[test]
    fn fn_decl_method_syntax_with_contextual_keyword_name() {
        let Decl::Fn(f) = parse_decl("fn ApiError.from(msg string) -> Self")
            .unwrap()
            .node
        else {
            panic!("expected Decl::Fn")
        };
        assert_eq!(f.name(), "ApiError");
        assert_eq!(f.method_name(), Some("from".to_string()));
    }

    #[test]
    fn fn_decl_pure_prefix() {
        let Decl::Fn(f) = parse_decl(
            "pure fn AppReducer(const state ChatAppState, const action AppActions) -> ChatAppState {
    return state
}",
        )
        .unwrap()
        .node
        else {
            panic!("expected Decl::Fn")
        };
        assert!(f.is_pure());
        let params = f.params();
        assert_eq!(params.len(), 2);
        assert!(params[0].is_const());
        assert_eq!(params[0].name(), Some("state".to_string()));
    }

    #[test]
    fn fn_decl_self_and_ref_self_params() {
        let Decl::Fn(f) = parse_decl("fn Component.render(&self) -> Component { return self }")
            .unwrap()
            .node
        else {
            panic!("expected Decl::Fn")
        };
        assert_eq!(f.params().len(), 0); // &self is a SelfParam, not a Param
        let self_params: Vec<_> = f
            .syntax()
            .descendants()
            .filter(|n| n.kind() == SyntaxKind::SELF_PARAM)
            .filter_map(SelfParam::cast)
            .collect();
        assert_eq!(self_params.len(), 1);
        assert!(self_params[0].is_ref());
    }

    #[test]
    fn fn_decl_body_with_nested_braces_stays_balanced() {
        // The opaque body scan must track brace depth so a nested `{ }`
        // (a builder block, in real code) doesn't prematurely close the
        // function's own body.
        let Decl::Fn(f) = parse_decl(
            "fn Outer() {
    if true {
        nested()
    }
}",
        )
        .unwrap()
        .node
        else {
            panic!("expected Decl::Fn")
        };
        assert!(f.has_body());
    }

    #[test]
    fn error_fn_decl_missing_name() {
        let err = parse_decl("fn (x string)").unwrap_err();
        assert!(
            format!("{err}").contains("function name"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn error_fn_decl_unclosed_body() {
        let err = parse_decl("fn Foo() { println(\"hi\")").unwrap_err();
        assert!(format!("{err}").contains("'}'"), "unexpected error: {err}");
    }
}
