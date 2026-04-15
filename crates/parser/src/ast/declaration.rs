use lexer::{
    span::SpannedToken,
    tokens::{LexerSymbol, LexerTokens},
};

use crate::{
    Spanned, TokenCursor,
    error::{ElpParserError, ParsingError},
    span::spans_extent,
};

#[derive(Debug, PartialEq, Clone)]
pub struct ImportName {
    pub name: String,
    pub alias: Option<String>,
}

impl ImportName {
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

#[derive(Debug, PartialEq)]
pub enum Decl {
    Function {},
    Variable {},
    Constant {},
    Import {
        var_names: Vec<ImportName>,
        from_module: Vec<String>,
        // True for a leading-dot relative import like `from .components.paragraph`.
        relative: bool,
    },
    Object {},
    Interface {},
    Export {},
    Macro {},
}

impl Decl {
    pub fn from_cursor(cursor: &mut TokenCursor) -> Result<Spanned<Self>, ElpParserError> {
        let lexer_token_opt = cursor.peek();
        if lexer_token_opt.is_none() {
            return Err(ElpParserError::ScriptParsingError(ParsingError::Expected(
                "No token to peek at, this is a compiler bug".into(),
            )));
        }

        let token = lexer_token_opt.unwrap();

        match &token.token {
            LexerTokens::Identifier(ident) => decl_from_ident(ident.clone(), cursor),
            _ => Err(ElpParserError::ScriptParsingError(
                ParsingError::UnexpectedToken(token.clone()),
            )),
        }
    }

    pub fn matches_ident(ident: &str) -> bool {
        matches!(
            ident,
            "fn" | "var" | "const" | "import" | "object" | "interface" | "export"
        )
    }
}

fn decl_from_ident(
    ident: String,
    cursor: &mut TokenCursor,
) -> Result<Spanned<Decl>, ElpParserError> {
    match ident.as_str() {
        "import" => import_from_cursor(cursor),
        // TODO get the span for this error
        _ => Err(ElpParserError::ScriptParsingError(
            ParsingError::Unexpected(None, "Unexpected token".into()),
        )),
    }
}

fn unexpected(token: &SpannedToken, ctx: &str) -> ElpParserError {
    ElpParserError::ScriptParsingError(ParsingError::Unexpected(
        Some(token.span.clone()),
        format!("{ctx}: '{:?}'", token.token),
    ))
}

fn expected_got(token: Option<&SpannedToken>, msg: &str) -> ElpParserError {
    ElpParserError::ScriptParsingError(ParsingError::ExpectedGot(
        token.map(|t| t.span.clone()),
        msg.to_string(),
    ))
}

fn import_from_cursor(cursor: &mut TokenCursor) -> Result<Spanned<Decl>, ElpParserError> {
    let mut var_names: Vec<ImportName> = vec![];
    let mut from_module: Vec<String> = vec![];
    let mut relative = false;

    // Consume `import` and `{`, collecting their spans.
    let import_tok = cursor
        .next_token()
        .ok_or_else(|| expected_got(None, "'import' keyword"))?;
    let mut spans = vec![import_tok.span.clone()];

    let brace_tok = cursor
        .next_token()
        .ok_or_else(|| expected_got(None, "'{' after import"))?;
    if !matches!(&brace_tok.token, LexerTokens::Symbol(LexerSymbol::LCurley)) {
        return Err(expected_got(Some(brace_tok), "'{' after import"));
    }
    spans.push(brace_tok.span.clone());

    // Collect `{ Name (as Alias)?, ... }`.
    // State: after a name we may see `as`, `,`, or `}`. After `,` or `{` we expect a name.
    let mut closed = false;
    let mut expect_name = true;
    while let Some(token) = cursor.peek() {
        match &token.token {
            LexerTokens::Identifier(ident) => {
                if !expect_name {
                    return Err(unexpected(token, "expected ',' or '}' between imports"));
                }
                let name = ident.to_owned();
                cursor.consume_span_into(&mut spans);

                // Peek past any newlines to see if there's an `as` alias.
                let mut alias: Option<String> = None;
                while let Some(next) = cursor.peek() {
                    match &next.token {
                        LexerTokens::Newline => {
                            cursor.consume_span_into(&mut spans);
                            continue;
                        }
                        LexerTokens::Identifier(maybe_as) if maybe_as == "as" => {
                            cursor.consume_span_into(&mut spans); // consume `as`
                            // Skip newlines between `as` and the alias identifier.
                            loop {
                                match cursor.peek() {
                                    Some(t) if matches!(t.token, LexerTokens::Newline) => {
                                        cursor.consume_span_into(&mut spans);
                                    }
                                    _ => break,
                                }
                            }
                            let alias_tok = cursor
                                .peek()
                                .ok_or_else(|| expected_got(None, "alias identifier after 'as'"))?;
                            if let LexerTokens::Identifier(a) = &alias_tok.token {
                                alias = Some(a.to_owned());
                                cursor.consume_span_into(&mut spans);
                            } else {
                                return Err(expected_got(
                                    Some(alias_tok),
                                    "alias identifier after 'as'",
                                ));
                            }
                        }
                        _ => {}
                    }
                    break;
                }

                var_names.push(ImportName { name, alias });
                expect_name = false;
            }
            LexerTokens::Symbol(symbol) => match symbol {
                LexerSymbol::RCurley => {
                    cursor.consume_span_into(&mut spans);
                    closed = true;
                    break;
                }
                LexerSymbol::Comma => {
                    if expect_name {
                        return Err(unexpected(token, "unexpected ','"));
                    }
                    cursor.consume_span_into(&mut spans);
                    expect_name = true;
                    continue;
                }
                _ => {
                    return Err(unexpected(token, "unexpected token in import list"));
                }
            },
            LexerTokens::Newline => {
                cursor.consume_span_into(&mut spans);
                continue;
            }
            LexerTokens::EOI => {
                return Err(expected_got(Some(token), "'}' to close import list"));
            }
            _ => {
                return Err(unexpected(token, "unexpected token in import list"));
            }
        }
    }

    if !closed {
        return Err(expected_got(None, "'}' to close import list"));
    }

    // Skip any newlines between `}` and `from`.
    while let Some(t) = cursor.peek() {
        if matches!(t.token, LexerTokens::Newline) {
            cursor.consume_span_into(&mut spans);
        } else {
            break;
        }
    }

    // Expect the `from` keyword.
    let from_tok = cursor
        .next_token()
        .ok_or_else(|| expected_got(None, "'from' after import list"))?;
    match &from_tok.token {
        LexerTokens::Identifier(i) if i == "from" => {
            spans.push(from_tok.span.clone());
        }
        _ => {
            return Err(expected_got(Some(from_tok), "'from' after import list"));
        }
    }

    // Optional leading `.` marks a relative import.
    if let Some(t) = cursor.peek()
        && matches!(t.token, LexerTokens::Symbol(LexerSymbol::Period))
    {
        relative = true;
        cursor.consume_span_into(&mut spans);
    }

    // Collect the module path: ident (`.` ident)*
    let mut expect_segment = true;
    while let Some(token) = cursor.peek() {
        match &token.token {
            LexerTokens::Identifier(ident) => {
                if !expect_segment {
                    break;
                }
                from_module.push(ident.to_owned());
                cursor.consume_span_into(&mut spans);
                expect_segment = false;
            }
            LexerTokens::Symbol(LexerSymbol::Period) => {
                if expect_segment {
                    return Err(unexpected(token, "unexpected '.' in module path"));
                }
                cursor.consume_span_into(&mut spans);
                expect_segment = true;
                continue;
            }
            LexerTokens::Newline | LexerTokens::EOI => break,
            _ => break,
        }
    }

    if from_module.is_empty() {
        return Err(expected_got(None, "module path after 'from'"));
    }

    Ok(Spanned {
        node: Decl::Import {
            var_names,
            from_module,
            relative,
        },
        span: spans_extent(spans)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use lexer::ElpLexer;
    use pretty_assertions::assert_eq;

    fn parse_import(src: &str) -> Result<Spanned<Decl>, ElpParserError> {
        let mut lexer = ElpLexer::new(src);
        let tokens = lexer.scan();
        let mut cursor = TokenCursor::new(&tokens);
        import_from_cursor(&mut cursor)
    }

    fn unwrap_import(decl: Decl) -> (Vec<ImportName>, Vec<String>, bool) {
        match decl {
            Decl::Import {
                var_names,
                from_module,
                relative,
            } => (var_names, from_module, relative),
            other => panic!("expected Decl::Import, got {:?}", other),
        }
    }

    #[test]
    fn single_named_import() {
        let result = parse_import("import { test } from my.module").unwrap();
        let (names, path, relative) = unwrap_import(result.node);
        assert_eq!(names, vec![ImportName::plain("test")]);
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
                ImportName::plain("User"),
                ImportName::plain("Message"),
                ImportName::plain("Actions"),
            ]
        );
        assert_eq!(path, vec!["my".to_string(), "module".to_string()]);
        assert!(!relative);
    }

    #[test]
    fn import_with_alias() {
        let result = parse_import("import { Message as MessageView } from my.module").unwrap();
        let (names, _, _) = unwrap_import(result.node);
        assert_eq!(names, vec![ImportName::aliased("Message", "MessageView")]);
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
                ImportName::plain("User"),
                ImportName::aliased("Message", "MessageView"),
                ImportName::plain("Actions"),
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
                ImportName::plain("OBJC"),
                ImportName::plain("OBJC_Obj"),
                ImportName::plain("Error"),
                ImportName::plain("Either"),
            ]
        );
        assert_eq!(path, vec!["elp".to_string(), "types".to_string()]);
    }

    #[test]
    fn relative_import_leading_dot() {
        let result = parse_import("import { Paragraph } from .components.paragraph").unwrap();
        let (names, path, relative) = unwrap_import(result.node);
        assert_eq!(names, vec![ImportName::plain("Paragraph")]);
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
            assert_eq!(names, vec![ImportName::plain(expected)]);
            assert_eq!(path, vec!["x".to_string(), "y".to_string()]);

            // Advance past the trailing newline, if any.
            if let Some(tok) = cursor.peek() {
                if matches!(tok.token, LexerTokens::Newline) {
                    cursor.next_token();
                }
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
        assert_eq!(names, vec![ImportName::plain("a")]);
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
        assert_eq!(names, vec![ImportName::plain("a"), ImportName::plain("b")]);
    }
}
