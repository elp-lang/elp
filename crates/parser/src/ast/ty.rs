//! `type` grammar rule. No generics yet (`namedType`'s
//! trailing `<...>`) — nothing anywhere in the parser handles generics,
//! it's a whole-grammar gap rather than something specific to types. No
//! `OBJECT '{' fieldList '}'` anonymous object types yet either.

use cst::{AstNode, SyntaxKind, SyntaxNode};
use lexer::{span::CodeSpan, tokens::LexerTokens};

use crate::{
    TreeParser,
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

ast_node!(NamedType, SyntaxKind::NAMED_TYPE);
ast_node!(OptionalType, SyntaxKind::OPTIONAL_TYPE);
ast_node!(RefType, SyntaxKind::REF_TYPE);
ast_node!(ArrayType, SyntaxKind::ARRAY_TYPE);

impl NamedType {
    pub fn segments(&self) -> Vec<String> {
        self.0
            .children_with_tokens()
            .filter_map(|el| el.into_token())
            .filter(|t| t.kind() == SyntaxKind::IDENT)
            .map(|t| t.text().to_string())
            .collect()
    }
}

impl OptionalType {
    pub fn inner(&self) -> Option<Type> {
        self.0.children().find_map(Type::cast)
    }
}

impl RefType {
    pub fn inner(&self) -> Option<Type> {
        self.0.children().find_map(Type::cast)
    }
}

impl ArrayType {
    pub fn element(&self) -> Option<Type> {
        self.0.children().find_map(Type::cast)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    Named(NamedType),
    Optional(OptionalType),
    Ref(RefType),
    Array(ArrayType),
}

impl AstNode for Type {
    fn can_cast(kind: SyntaxKind) -> bool {
        matches!(
            kind,
            SyntaxKind::NAMED_TYPE
                | SyntaxKind::OPTIONAL_TYPE
                | SyntaxKind::REF_TYPE
                | SyntaxKind::ARRAY_TYPE
        )
    }

    fn cast(node: SyntaxNode) -> Option<Self> {
        Some(match node.kind() {
            SyntaxKind::NAMED_TYPE => Type::Named(NamedType::cast(node)?),
            SyntaxKind::OPTIONAL_TYPE => Type::Optional(OptionalType::cast(node)?),
            SyntaxKind::REF_TYPE => Type::Ref(RefType::cast(node)?),
            SyntaxKind::ARRAY_TYPE => Type::Array(ArrayType::cast(node)?),
            _ => return None,
        })
    }

    fn syntax(&self) -> &SyntaxNode {
        match self {
            Type::Named(n) => n.syntax(),
            Type::Optional(n) => n.syntax(),
            Type::Ref(n) => n.syntax(),
            Type::Array(n) => n.syntax(),
        }
    }
}

/// `type '?'? | '&' type | '[' type ']' | namedType`. `?` is postfix and
/// left-recursive in the grammar, so — same as expression's binary
/// operators — it's handled by checkpointing before the prefix form and
/// retroactively wrapping in `OPTIONAL_TYPE` once a `?` actually shows up.
pub(crate) fn parse_type(
    p: &mut TreeParser,
    spans: &mut Vec<CodeSpan>,
) -> Result<(), ElpParserError> {
    let checkpoint = p.checkpoint();
    parse_type_prefix(p, spans)?;

    while matches!(
        p.peek().map(|t| &t.token),
        Some(LexerTokens::Symbol(lexer::tokens::LexerSymbol::Question))
    ) {
        p.start_node_at(checkpoint, SyntaxKind::OPTIONAL_TYPE);
        p.bump(SyntaxKind::QUESTION, spans);
        p.finish_node();
    }
    Ok(())
}

fn parse_type_prefix(p: &mut TreeParser, spans: &mut Vec<CodeSpan>) -> Result<(), ElpParserError> {
    use lexer::tokens::LexerSymbol;

    let Some(tok) = p.peek().cloned() else {
        return Err(expected_got(None, "a type"));
    };

    match &tok.token {
        LexerTokens::Symbol(LexerSymbol::Ampersand) => {
            p.start_node(SyntaxKind::REF_TYPE);
            p.bump(SyntaxKind::AMP, spans);
            parse_type(p, spans)?;
            p.finish_node();
            Ok(())
        }
        LexerTokens::Symbol(LexerSymbol::LBracket) => {
            p.start_node(SyntaxKind::ARRAY_TYPE);
            p.bump(SyntaxKind::LBRACK, spans);
            parse_type(p, spans)?;
            let close = p
                .peek()
                .cloned()
                .ok_or_else(|| expected_got(None, "']' to close array type"))?;
            if matches!(close.token, LexerTokens::Symbol(LexerSymbol::RBracket)) {
                p.bump(SyntaxKind::RBRACK, spans);
            } else {
                return Err(expected_got(Some(&close), "']' to close array type"));
            }
            p.finish_node();
            Ok(())
        }
        LexerTokens::Identifier(_) => parse_named_type(p, spans),
        _ => Err(unexpected(&tok, "expected a type")),
    }
}

fn parse_named_type(p: &mut TreeParser, spans: &mut Vec<CodeSpan>) -> Result<(), ElpParserError> {
    use lexer::tokens::LexerSymbol;

    p.start_node(SyntaxKind::NAMED_TYPE);
    p.bump(SyntaxKind::IDENT, spans);

    while matches!(
        p.peek().map(|t| &t.token),
        Some(LexerTokens::Symbol(LexerSymbol::Period))
    ) {
        p.bump(SyntaxKind::DOT, spans);
        let seg = p
            .peek()
            .cloned()
            .ok_or_else(|| expected_got(None, "identifier after '.' in type path"))?;
        if matches!(seg.token, LexerTokens::Identifier(_)) {
            p.bump(SyntaxKind::IDENT, spans);
        } else {
            return Err(expected_got(
                Some(&seg),
                "identifier after '.' in type path",
            ));
        }
    }
    p.finish_node();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TokenCursor;
    use lexer::ElpLexer;
    use pretty_assertions::assert_eq;

    fn parse(src: &str) -> Type {
        let mut lexer = ElpLexer::new(src);
        let tokens = lexer.scan();
        let mut cursor = TokenCursor::new(&tokens);
        let mut spans = vec![];
        let mut p = TreeParser::new(&mut cursor);
        parse_type(&mut p, &mut spans).unwrap();
        Type::cast(p.finish()).unwrap()
    }

    fn parse_err(src: &str) -> ElpParserError {
        let mut lexer = ElpLexer::new(src);
        let tokens = lexer.scan();
        let mut cursor = TokenCursor::new(&tokens);
        let mut spans = vec![];
        let mut p = TreeParser::new(&mut cursor);
        parse_type(&mut p, &mut spans).unwrap_err()
    }

    #[test]
    fn simple_named_type() {
        let Type::Named(named) = parse("string") else {
            panic!("expected named type")
        };
        assert_eq!(named.segments(), vec!["string".to_string()]);
    }

    #[test]
    fn dotted_named_type() {
        let Type::Named(named) = parse("elp.types.Error") else {
            panic!("expected named type")
        };
        assert_eq!(
            named.segments(),
            vec!["elp".to_string(), "types".to_string(), "Error".to_string()]
        );
    }

    #[test]
    fn optional_type() {
        let Type::Optional(opt) = parse("User?") else {
            panic!("expected optional type")
        };
        let Some(Type::Named(named)) = opt.inner() else {
            panic!("expected named inner type")
        };
        assert_eq!(named.segments(), vec!["User".to_string()]);
    }

    #[test]
    fn ref_type() {
        let Type::Ref(r) = parse("&string") else {
            panic!("expected ref type")
        };
        assert!(matches!(r.inner(), Some(Type::Named(_))));
    }

    #[test]
    fn array_type() {
        let Type::Array(arr) = parse("[Message]") else {
            panic!("expected array type")
        };
        let Some(Type::Named(named)) = arr.element() else {
            panic!("expected named element type")
        };
        assert_eq!(named.segments(), vec!["Message".to_string()]);
    }

    #[test]
    fn postfix_question_mark_binds_to_the_nearest_type() {
        // `&[Message]?` is genuinely ambiguous by the formal grammar alone
        // (does `?` apply to `[Message]` or to the whole `&[Message]`?).
        // This recursive-descent implementation resolves it by binding `?`
        // to the innermost type it can reach before returning up the call
        // stack — so this parses as `&([Message]?)`, not `(&[Message])?`.
        // No real example in the corpus exercises this composition; this
        // test exists to document and lock in the choice, not because the
        // grammar mandates it.
        let Type::Ref(r) = parse("&[Message]?") else {
            panic!("expected the outermost type to be a ref")
        };
        let Some(Type::Optional(opt)) = r.inner() else {
            panic!("expected '?' to bind to the array type, not the ref")
        };
        assert!(matches!(opt.inner(), Some(Type::Array(_))));
    }

    #[test]
    fn error_empty_input() {
        let err = parse_err("");
        assert!(
            format!("{err}").contains("a type"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn error_token_that_cannot_start_a_type() {
        // A number, a bare symbol other than `&`/`[`, a keyword-shaped
        // identifier used out of place — none of these start a `type`.
        for src in ["42", "=", "->"] {
            let err = parse_err(src);
            assert!(
                format!("{err}").contains("expected a type"),
                "unexpected error for {src:?}: {err}"
            );
        }
    }

    #[test]
    fn error_ref_with_nothing_after() {
        let err = parse_err("&");
        assert!(
            format!("{err}").contains("a type"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn error_array_type_missing_element() {
        let err = parse_err("[]");
        assert!(
            format!("{err}").contains("a type"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn error_array_type_unclosed() {
        let err = parse_err("[Message");
        assert!(format!("{err}").contains("']'"), "unexpected error: {err}");
    }

    #[test]
    fn error_array_type_wrong_closer() {
        let err = parse_err("[Message)");
        assert!(format!("{err}").contains("']'"), "unexpected error: {err}");
    }

    #[test]
    fn error_dotted_path_missing_segment() {
        let err = parse_err("elp.");
        assert!(
            format!("{err}").contains("identifier after '.'"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn error_dotted_path_dot_followed_by_non_identifier() {
        let err = parse_err("elp.42");
        assert!(
            format!("{err}").contains("identifier after '.'"),
            "unexpected error: {err}"
        );
    }
}
