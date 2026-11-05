/// Every kind of token or node that can appear in an elp concrete syntax
/// tree. Grows alongside the parser: only add a variant once something in
/// `crates/parser` actually produces it — this enum should never get ahead
/// of what's implemented.
///
/// Token kinds mirror `lexer::tokens::{LexerTokens, LexerSymbol}`, plus a
/// keyword split the lexer itself doesn't make (the lexer emits every
/// alphabetic run as `Identifier`; the parser/CST builder re-tags keyword
/// text as the matching `_KW` kind once it knows the grammar context).
///
/// `__LAST` is not a real kind — it's a sentinel so `ElpLanguage::kind_from_raw`
/// can bounds-check before transmuting untrusted `u16`s from rowan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
#[non_exhaustive]
#[allow(non_camel_case_types)] // SCREAMING_SNAKE_CASE syntax-kind names are the established convention (rust-analyzer, etc.)
pub enum SyntaxKind {
    // ---- Literal / identifier tokens ----
    IDENT,
    // The lexer emits one `Number` token for both; the parser splits them by
    // checking for a `.` in the text when it re-tags into the tree.
    INTEGER,
    FLOAT,
    STRING,
    BOOLEAN,

    // ---- Keyword tokens ----
    IMPORT_KW,
    FROM_KW,
    AS_KW,
    EXPORT_KW,
    FN_KW,
    PURE_KW,
    VAR_KW,
    CONST_KW,
    OBJECT_KW,
    INTERFACE_KW,
    ENUM_KW,
    IMPLEMENTING_KW,
    IF_KW,
    ELSE_KW,
    MATCH_KW,
    FOR_KW,
    IN_KW,
    RETURN_KW,
    SELF_KW,
    TRUE_KW,
    FALSE_KW,
    NIL_KW,

    // ---- Punctuation / operator tokens ----
    EQ,
    PLUS,
    MINUS,
    STAR,
    SLASH,
    SEMICOLON,
    EQEQ,
    ARROW,
    PLUSEQ,
    MINUSEQ,
    STAREQ,
    SLASHEQ,
    AMPEQ,
    PIPEEQ,
    NEQ,
    LCURLY,
    RCURLY,
    COMMA,
    DOT,
    RANGE,
    COLON,
    AT,
    LT,
    GT,
    LEQ,
    GEQ,
    ANDAND,
    OROR,
    PERCENT,
    AMP,
    BANG,
    QUESTION,
    LPAREN,
    RPAREN,
    LBRACK,
    RBRACK,

    // ---- Lexer-error tokens ----
    ERROR_TOKEN,
    ILLEGAL_TOKEN,

    // ---- Trivia ----
    NEWLINE,
    WHITESPACE,
    LINE_COMMENT,
    DOC_COMMENT,
    BLOCK_COMMENT,

    // ---- Nodes (named after the parser's own rule functions) ----
    SCRIPT,
    IMPORT_STMT,
    IMPORT_LIST,
    IMPORT_NAME,
    MODULE_PATH,

    // ---- Expression nodes ----
    LITERAL_EXPR,
    IDENT_EXPR,
    CONTEXTUAL_VARIANT_EXPR,
    PAREN_EXPR,
    UNARY_EXPR,
    // One node shape for every precedence level (`*`, `+`, `<`, `==`, `&&`,
    // `||`, ...) — the operator token itself records which. A generated
    // parser would need a separate rule per level; a Pratt parser doesn't.
    BINARY_EXPR,
    MEMBER_EXPR,
    CALL_EXPR,
    ARG_LIST,
    ARG,
    INDEX_EXPR,
    ARRAY_LITERAL_EXPR,
    TUPLE_LITERAL_EXPR,
    OBJECT_LITERAL_EXPR,
    FIELD_INIT,
    LAMBDA_EXPR,
    IF_EXPR,
    MATCH_EXPR,
    MATCH_ARMS,
    MATCH_ARM,
    /// The `{ statement* }` trailing a call/primary — `App(...) { Surface { ... } }`.
    /// Ambiguous with `OBJECT_LITERAL_EXPR` by the formal grammar alone; see
    /// `parse_primary`'s docs for how this parser resolves it.
    BUILDER_BLOCK,

    // ---- Statement / block nodes ----
    BLOCK,
    RETURN_STMT,
    ASSIGNMENT,
    FOR_STMT,

    // ---- Pattern nodes (match arms) ----
    WILDCARD_PATTERN,
    RANGE_CATCHALL_PATTERN,
    RANGE_PATTERN,
    VARIANT_PATTERN,
    TUPLE_PATTERN,
    BINDING_PATTERN,
    PATTERN_LIST,

    // ---- Type nodes ----
    // No genericArgs support yet (`namedType`'s trailing `<...>`) — nothing
    // parses generics anywhere yet, this is a whole-grammar gap, not
    // specific to types.
    NAMED_TYPE,
    OPTIONAL_TYPE,
    REF_TYPE,
    ARRAY_TYPE,

    // ---- Declaration nodes ----
    VAR_DECL,
    CONST_DECL,
    VAR_PATTERN,
    OBJECT_DECL,
    INTERFACE_DECL,
    /// `TypeName implementing A + B` — a standalone conformance marker, not
    /// part of the object's own declaration. Carries no method bodies; those
    /// are ordinary dotted free functions (`fn TypeName.method(...)`).
    IMPLEMENTING_DECL,
    ENUM_DECL,
    FN_DECL,
    /// `export *fn onSubmit(...) -> None` — a callback slot declared inside
    /// a block: a signature with no body, held as a value. The `*` marks it
    /// required; `export` (orthogonal, as everywhere) makes it a prop.
    CALLBACK_DECL,
    FIELD_LIST,
    FIELD,
    /// `+`-separated list of interfaces in an `implementing` declaration.
    /// Named for the separator it uses rather than `TYPE_LIST` (comma) so
    /// the two can't be confused when reading a tree.
    INTERFACE_LIST,
    TYPE_LIST,
    INTERFACE_BODY,
    INTERFACE_METHOD,
    ENUM_VARIANTS,
    VARIANT,
    PARAM_LIST,
    PARAM,
    SELF_PARAM,

    /// Wraps a span the parser couldn't make sense of. Not used yet — the
    /// parser currently bails out with `Result<_, ElpParserError>` instead
    /// of recovering — but reserved now since IDE-style error recovery is
    /// the whole reason a CST exists in the first place.
    ERROR,

    __LAST,
}

impl SyntaxKind {
    const LAST: u16 = SyntaxKind::__LAST as u16;
}

/// Marker type tying [`SyntaxKind`] into rowan's generic tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ElpLanguage {}

impl rowan::Language for ElpLanguage {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> SyntaxKind {
        assert!(
            raw.0 < SyntaxKind::LAST,
            "syntax kind {} out of range",
            raw.0
        );
        // SAFETY: `SyntaxKind` is `#[repr(u16)]` with contiguous discriminants
        // starting at 0, and we just bounds-checked `raw.0` against the
        // `__LAST` sentinel, so every value below it names a real variant.
        unsafe { std::mem::transmute::<u16, SyntaxKind>(raw.0) }
    }

    fn kind_to_raw(kind: SyntaxKind) -> rowan::SyntaxKind {
        rowan::SyntaxKind(kind as u16)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rowan::Language;

    #[test]
    fn round_trips_through_raw() {
        for kind in [
            SyntaxKind::IDENT,
            SyntaxKind::IMPORT_KW,
            SyntaxKind::ARROW,
            SyntaxKind::SCRIPT,
            SyntaxKind::ERROR,
        ] {
            let raw = ElpLanguage::kind_to_raw(kind);
            assert_eq!(ElpLanguage::kind_from_raw(raw), kind);
        }
    }

    #[test]
    #[should_panic(expected = "out of range")]
    fn rejects_out_of_range_raw() {
        ElpLanguage::kind_from_raw(rowan::SyntaxKind(SyntaxKind::LAST));
    }
}
