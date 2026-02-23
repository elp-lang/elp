use glob::glob;
use lexer::ElpLexer;
use std::{collections::HashMap, fs, path::PathBuf};

use crate::{ElpScript, ElpScriptType, error::ElpParserError};

#[derive(Debug, PartialEq, Default, Clone)]
pub struct ElpParserConfig {
    pub permissive_mode: bool,
}

#[derive(Debug, PartialEq)]
pub struct ElpParser {
    package_root: PathBuf,
    config: ElpParserConfig,

    // Scripts are .elp files.
    pub scripts: HashMap<PathBuf, ElpScript>,

    // view scripts are .velp files.
    pub view_scripts: HashMap<PathBuf, ElpScript>,

    // stylesheets are .selp files.
    pub stylesheets: HashMap<PathBuf, ElpScript>,
}

impl ElpParser {
    // Rather than do a complex "discovery" for scripts that need compiling,
    // Elp just ravages the file system at the package_path for all the scripts.
    // There are some disadvantages to some workflows with this but the benefits are far greater.
    pub fn new(package_path: PathBuf, config: ElpParserConfig) -> Self {
        if !package_path.exists() {
            panic!("There is no package at path {}", package_path.display());
        }

        let mut parsing_errors: Vec<ElpParserError> = vec![];

        let mut instance = Self {
            config: config.clone(),
            package_root: package_path,
            scripts: HashMap::new(),
            view_scripts: HashMap::new(),
            stylesheets: HashMap::new(),
        };

        let scripts_result = instance.parse_all_of_type(ElpScriptType::Script);
        let views_result = instance.parse_all_of_type(ElpScriptType::ViewScript);
        let stylesheets_result = instance.parse_all_of_type(ElpScriptType::Stylesheet);

        match scripts_result {
            Ok(scripts) => instance.scripts = scripts,
            Err(e) => {
                if config.permissive_mode {
                    parsing_errors.push(e);
                } else {
                    panic!("{}", e)
                }
            }
        }

        match views_result {
            Ok(scripts) => instance.view_scripts = scripts,
            Err(e) => {
                if config.permissive_mode {
                    parsing_errors.push(e);
                } else {
                    panic!("{}", e)
                }
            }
        }

        match stylesheets_result {
            Ok(scripts) => instance.view_scripts = scripts,
            Err(e) => {
                if config.permissive_mode {
                    parsing_errors.push(e);
                } else {
                    panic!("{}", e)
                }
            }
        }

        instance
    }

    fn parse_all_of_type(
        &mut self,
        script_type: ElpScriptType,
    ) -> Result<HashMap<PathBuf, ElpScript>, ElpParserError> {
        let found_scripts = self.glob_for_scripts(script_type)?;
        let mut parsed_scripts: HashMap<PathBuf, ElpScript> = HashMap::new();
        let mut parsing_errors: Vec<ElpParserError> = vec![];

        for script_path in found_scripts {
            let parsed = self.parse(&script_path);

            match parsed {
                Ok(script) => {
                    parsed_scripts.insert(script_path, script);
                }
                Err(err) => parsing_errors.push(err),
            };
        }

        match !self.config.permissive_mode && !parsing_errors.is_empty() {
            true => Err(ElpParserError::Vec(parsing_errors)),
            false => Ok(parsed_scripts),
        }
    }

    fn glob_for_scripts(
        &mut self,
        script_type: ElpScriptType,
    ) -> Result<Vec<PathBuf>, ElpParserError> {
        let mut files: Vec<PathBuf> = vec![];

        let files_result = glob(
            format!(
                "{}/**/*.{}",
                self.package_root.display(),
                script_type.file_ext()
            )
            .as_str(),
        );

        match files_result {
            Ok(paths) => {
                for glob_path_result in paths {
                    match glob_path_result {
                        Ok(path) => files.push(path),
                        Err(err) => {
                            return Err(ElpParserError::FileSystemError(None, err.to_string()));
                        }
                    }
                }
            }
            Err(err) => return Err(ElpParserError::FileSystemError(None, err.to_string())),
        };

        Ok(files)
    }

    fn parse(&mut self, path: &PathBuf) -> Result<ElpScript, ElpParserError> {
        let read_result = fs::read_to_string(path);

        match read_result {
            Ok(source) => {
                let mut lexer = ElpLexer::new(&source);
                let mut tokens = lexer.scan();

                ElpScript::from_tokens(&mut tokens)
            }
            Err(err) => Err(ElpParserError::FileSystemError(
                Some(path.clone()),
                err.to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::{ElpParser, ElpParserConfig};
    use pretty_assertions::assert_eq;

    const FIXTURES_PATH: &str = "./test_projects";

    #[test]
    fn test_empty() {
        let project_path = format!("{}/zero_ret", FIXTURES_PATH);
        println!("{}", project_path);
        let project_config = ElpParserConfig::default();

        let project = ElpParser::new(project_path.into(), project_config);

        assert_eq!(project.scripts.len(), 2);
        assert_eq!(project.view_scripts.len(), 0);
        assert_eq!(project.stylesheets.len(), 0);
    }
}
