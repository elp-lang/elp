#[derive(Debug, PartialEq)]
pub enum Decl {
    Import {
        module_path: Vec<String>, // e.g. ["core", "utils", "helpers"]
        names: Vec<(String, Option<String>)>,
    },
}
