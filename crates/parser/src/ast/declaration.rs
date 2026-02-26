use lexer::{
    span::SpannedToken,
    tokens::{LexerSymbol, LexerTokens},
};

use crate::{
    Spanned, TokenCursor,
    error::{ElpParserError, ParsingError},
};

#[derive(Debug, PartialEq)]
pub enum Decl {
    Function {},
    Variable {},
    Constant {},
    Import {
        var_names: Vec<String>,
        from_module: Vec<String>,
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

        let decl = match &token.token {
            LexerTokens::Identifier(ident) => decl_from_ident(ident.clone(), cursor),
            _ => Err(ElpParserError::ScriptParsingError(
                ParsingError::UnexpectedToken(token.clone()),
            )),
        };

        decl
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

fn import_from_cursor(cursor: &mut TokenCursor) -> Result<Spanned<Decl>, ElpParserError> {
    let mut var_names: Vec<String> = vec![];
    let mut from_module: Vec<String> = vec![];
    // Skip import and {, we want the names. Collect their spans though.
    let mut spans = vec![
        cursor.next_token().unwrap().span,
        cursor.next_token().unwrap().span,
    ];

    // Get all the imported variable nmes.
    while let Some(token) = cursor.peek() {
        match &token.token {
            LexerTokens::Identifier(ident) => {
                var_names.push(ident.to_owned());
                cursor.next_token();
            }
            LexerTokens::Symbol(symbol) => match symbol {
                LexerSymbol::RCurley => {
                    cursor.next_token();
                    break;
                }
                LexerSymbol::Comma => {
                    cursor.next_token();
                    continue;
                }
                _ => {
                    return Err(ElpParserError::ScriptParsingError(
                        ParsingError::Unexpected(
                            Some(token.span.clone()),
                            format!("unexpected token: '{:#?}'", token.token),
                        ),
                    ));
                }
            },
            _ => {
                return Err(ElpParserError::ScriptParsingError(
                    ParsingError::Unexpected(Some(token.span.clone()), "unexpexted token".into()),
                ));
            }
        }
    }

    // Check we got the "from".
    // rust made me collapse this into an ugly if instead of nested ones. bit odd for readability
    // personally but whatever. warnings be gone!
    if let Some(t) = cursor.next_token()
        && let LexerTokens::Identifier(i) = &t.token
        && i.as_str() != "from"
    {
        return Err(ElpParserError::ScriptParsingError(
            ParsingError::ExpectedGot(Some(t.span.clone()), "expected from".into()),
        ));
    }

    // We should be left with the import path now. ident.ident
    while let Some(token) = cursor.peek() {
        match &token.token {
            LexerTokens::Identifier(ident) => {
                from_module.push(ident.to_owned());
                cursor.next_token();
            }
            LexerTokens::Symbol(symbol) => match symbol {
                LexerSymbol::Period => {
                    cursor.next_token();
                    continue;
                }
                _ => break,
            },
            LexerTokens::EOI => break,
            _ => {
                return Err(ElpParserError::ScriptParsingError(
                    ParsingError::Unexpected(
                        Some(token.span.clone()),
                        format!("unexpected token: '{:#?}'", token.token),
                    ),
                ));
            }
        }
    }

    Ok(Decl::Import {
        var_names,
        from_module,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use lexer::ElpLexer;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_import_decl() {
        let expr = "import { test } from my.module";
        let mut lexer = ElpLexer::new(expr);
        let tokens = lexer.scan();
        let mut cursor = TokenCursor::new(&tokens);

        let import = import_from_cursor(&mut cursor);

        assert_eq!(
            import.unwrap(),
            Decl::Import {
                var_names: vec!["test".into()],
                from_module: vec!["my".into(), "module".into()]
            }
        )
    }
}
