use glob::glob;
use std::{collections::HashMap, path::PathBuf};

use crate::{ElpScript, ElpScriptType};

#[derive(Debug, PartialEq)]
pub struct ElpParser {
    package_root: PathBuf,

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
    pub fn new(package_path: PathBuf) -> Self {
        if !package_path.exists() {
            panic!("There is no package at path {}", package_path.display());
        }

        let instance = Self {
            package_root: package_path,
            scripts: HashMap::new(),
            view_scripts: HashMap::new(),
            stylesheets: HashMap::new(),
        };

        instance
    }

    fn glob_for_scripts(&mut self, script_type: ElpScriptType) {
        let mut files = glob(
            format!(
                "{}/**/*.{}",
                self.package_root.display(),
                script_type.file_ext()
            )
            .as_str(),
        )
        .expect("Failed to read script glob pattern")
        .into_iter();

        while let Some(entry) = files.next() {
            match entry {
                Ok(path) => println!("{:?}", path.display()),
                Err(e) => println!("{:?}", e),
            }
        }
    }
}
