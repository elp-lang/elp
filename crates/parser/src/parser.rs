use glob::glob;
use lexer::ElpLexer;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use crate::{ElpScript, ElpScriptType};

#[derive(Debug, PartialEq)]
pub struct ElpParserConfig {
    pub permissive_mode: bool,
}

#[derive(Debug, PartialEq)]
pub struct ElpParser {
    package_root: PathBuf,
    config: ElpParserConfig,

    // Scripts are .elp files.
    pub scripts: HashMap<String, ElpScript>,

    // view scripts are .velp files.
    pub view_scripts: HashMap<String, ElpScript>,

    // stylesheets are .selp files.
    pub stylesheets: HashMap<String, ElpScript>,
}

impl ElpParser {
    // Rather than do a complex "discovery" for scripts that need compiling,
    // Elp just ravages the file system at the package_path for all the scripts.
    // There are some disadvantages to some workflows with this but the benefits are far greater.
    pub fn new(package_path: PathBuf, config: ElpParserConfig) -> Self {
        if !package_path.exists() {
            panic!("There is no package at path {}", package_path.display());
        }

        let instance = Self {
            config,
            package_root: package_path,
            scripts: HashMap::new(),
            view_scripts: HashMap::new(),
            stylesheets: HashMap::new(),
        };

        instance
    }

    fn parse_all_of_type(
        &mut self,
        script_type: ElpScriptType,
    ) -> HashMap<String, anyhow::Result<ElpScript>> {
        let found_scripts = self.glob_for_scripts(script_type);
        found_scripts
            .iter()
            .map(|path| (path, self.parse(path)))
            .collect()
    }

    fn glob_for_scripts(&mut self, script_type: ElpScriptType) -> Vec<PathBuf> {
        glob(
            format!(
                "{}/**/*.{}",
                self.package_root.display(),
                script_type.file_ext()
            )
            .as_str(),
        )
        .expect("Failed to read script glob pattern")
        .into_iter()
    }

    fn parse(&mut self, path: PathBuf) -> anyhow::Result<ElpScript> {
        let source = fs::read_to_string(path)?;
        let mut lexer = ElpLexer::new(source);
        let mut tokens = lexer.scan();

        ElpScript::from_tokens(&self.config, &mut tokens)
    }
}
