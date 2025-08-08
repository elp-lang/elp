#[derive(Debug, PartialEq)]
pub struct ImportNode {
    pub module_path: Vec<String>,

    /// names are stored as tuples, Import name -> Alias
    pub names: Vec<(String, Option<String>)>,
}
