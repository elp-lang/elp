use lexer::span::SpannedToken;

#[derive(Debug, PartialEq)]
pub enum Expr {
    Identifier {
        name: String,
        span: SpannedToken,
    },
    Int64 {
        value: i64,
        span: SpannedToken,
    },
    Import {
        module_path: Vec<String>,

        // names are stored as tuples, Import name -> Alias
        names: Vec<(String, Option<String>)>,
        span: SpannedToken,
    },
}

impl Expr {
    fn from_tokens(tokens: &mut Vec<SpannedToken>) -> Self {}
}
