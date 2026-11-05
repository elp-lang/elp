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

    // Per-file failures, across all three script types. Always populated
    // (even outside permissive_mode, though then `new` panics before
    // returning, so nothing ever gets to read it in that case). In
    // permissive_mode this is the only way to find out that some files
    // didn't parse — they're just missing from `scripts`/`view_scripts`/
    // `stylesheets` otherwise, with nothing to say why.
    pub parsing_errors: Vec<ElpParserError>,
}

impl ElpParser {
    // Rather than do a complex "discovery" for scripts that need compiling,
    // Elp just ravages the file system at the package_path for all the scripts.
    // There are some disadvantages to some workflows with this but the benefits are far greater.
    pub fn new(package_path: PathBuf, config: ElpParserConfig) -> Self {
        if !package_path.exists() {
            panic!("There is no package at path {}", package_path.display());
        }

        let mut instance = Self {
            config: config.clone(),
            package_root: package_path,
            scripts: HashMap::new(),
            view_scripts: HashMap::new(),
            stylesheets: HashMap::new(),
            parsing_errors: vec![],
        };

        let (scripts, mut errors) = instance.parse_all_of_type(ElpScriptType::Script);
        let (view_scripts, view_errors) = instance.parse_all_of_type(ElpScriptType::ViewScript);
        let (stylesheets, stylesheet_errors) =
            instance.parse_all_of_type(ElpScriptType::Stylesheet);
        errors.extend(view_errors);
        errors.extend(stylesheet_errors);

        instance.scripts = scripts;
        instance.view_scripts = view_scripts;
        instance.stylesheets = stylesheets;
        instance.parsing_errors = errors;

        if !config.permissive_mode && !instance.parsing_errors.is_empty() {
            panic!("{}", ElpParserError::Vec(instance.parsing_errors));
        }

        instance
    }

    /// Parses every file of `script_type` found under the package root.
    /// Always returns whatever parsed successfully alongside whatever
    /// failed — `new` decides what to do with the failures (panic outside
    /// permissive_mode, otherwise collect into `self.parsing_errors`).
    fn parse_all_of_type(
        &mut self,
        script_type: ElpScriptType,
    ) -> (HashMap<PathBuf, ElpScript>, Vec<ElpParserError>) {
        let found_scripts = match self.glob_for_scripts(script_type) {
            Ok(paths) => paths,
            Err(e) => return (HashMap::new(), vec![e]),
        };

        let mut parsed_scripts: HashMap<PathBuf, ElpScript> = HashMap::new();
        let mut parsing_errors: Vec<ElpParserError> = vec![];

        for script_path in found_scripts {
            match self.parse(&script_path) {
                Ok(script) => {
                    parsed_scripts.insert(script_path, script);
                }
                Err(err) => parsing_errors.push(err),
            }
        }

        (parsed_scripts, parsing_errors)
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
    use crate::{
        ElpScriptType,
        error::ElpParserError,
        parser::{ElpParser, ElpParserConfig},
    };
    use pretty_assertions::assert_eq;
    use std::path::PathBuf;

    fn parser_for(package_root: &str) -> ElpParser {
        ElpParser {
            package_root: PathBuf::from(package_root),
            config: ElpParserConfig::default(),
            scripts: Default::default(),
            view_scripts: Default::default(),
            stylesheets: Default::default(),
            parsing_errors: Default::default(),
        }
    }

    #[test]
    fn parse_wraps_a_file_read_failure() {
        // Only reachable via a direct call: `ElpParser::new`'s normal flow
        // never calls `parse` on a path that didn't just come back from a
        // successful glob, so the file always exists at read time. Same
        // "reachable only out-of-band" shape as `Decl::from_cursor`'s
        // exhausted-cursor check.
        let mut parser = parser_for(".");
        let err = parser
            .parse(&PathBuf::from("/definitely/does/not/exist.elp"))
            .unwrap_err();
        assert!(
            matches!(err, ElpParserError::FileSystemError(Some(_), _)),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn glob_for_scripts_wraps_a_malformed_pattern() {
        // The glob pattern is built from `package_root` — a `package_root`
        // containing an unbalanced `[` produces a syntactically invalid
        // glob pattern, which `ElpParser::new`'s normal flow never does
        // (it's always a real, existence-checked directory), but
        // `glob_for_scripts` should still fail cleanly rather than panic.
        let mut parser = parser_for("[");
        let err = parser.glob_for_scripts(ElpScriptType::Script).unwrap_err();
        assert!(
            matches!(err, ElpParserError::FileSystemError(None, _)),
            "unexpected error: {err}"
        );
    }

    const FIXTURES_PATH: &str = "./test_projects";

    #[test]
    fn permissive_mode_drops_files_that_fail_to_parse() {
        // zero_ret/Package.elp uses `@Package(...)`, a standalone
        // macro-attributed item — the macro subsystem isn't implemented,
        // so it fails to parse. Non-permissive mode would panic on that
        // (see `panics_by_default_on_a_parse_error` below); permissive
        // mode instead drops it and keeps the one file that did parse
        // (main.elp). This was previously untestable — the only test here
        // was commented out and asserted `scripts.len() == 2`, which
        // hasn't reflected reality since `@Package` support was scoped out.
        let project_path = format!("{FIXTURES_PATH}/zero_ret");
        let config = ElpParserConfig {
            permissive_mode: true,
        };

        let project = ElpParser::new(project_path.into(), config);

        assert_eq!(project.scripts.len(), 1);
        assert_eq!(project.view_scripts.len(), 0);
        assert_eq!(project.stylesheets.len(), 0);
        // Regression: permissive_mode used to drop per-file errors
        // entirely — `parse_all_of_type` only ever returned `Err` (the
        // only path that fed the old, dead `parsing_errors` vec) when
        // *not* permissive, so there was no way to know Package.elp had
        // failed at all short of noticing it missing from `scripts`.
        assert_eq!(project.parsing_errors.len(), 1);
        assert!(
            matches!(
                &project.parsing_errors[0],
                ElpParserError::ScriptParsingError(_)
            ),
            "unexpected error: {:?}",
            project.parsing_errors[0]
        );
    }

    #[test]
    fn permissive_mode_collects_errors_across_every_script_type() {
        // Same regression as above, specifically for the case that broke
        // before the fix: failures from *different* script types
        // (.elp *and* .velp here) both need to show up, not just
        // whichever type's `Err` happened to reach the outer match first
        // under the old per-type Result plumbing.
        let project_path = format!("{FIXTURES_PATH}/multi_type_errors");
        let config = ElpParserConfig {
            permissive_mode: true,
        };

        let project = ElpParser::new(project_path.into(), config);

        assert_eq!(project.scripts.len(), 0);
        assert_eq!(project.view_scripts.len(), 0);
        assert_eq!(project.parsing_errors.len(), 2);
    }

    #[test]
    #[should_panic]
    fn panics_by_default_on_a_parse_error() {
        // permissive_mode defaults to false — zero_ret/Package.elp fails
        // to parse (see above), so this should panic rather than silently
        // dropping it.
        let project_path = format!("{FIXTURES_PATH}/zero_ret");
        ElpParser::new(project_path.into(), ElpParserConfig::default());
    }

    #[test]
    fn scripts_view_scripts_and_stylesheets_land_in_distinct_fields() {
        // Regression for a real bug: the stylesheets branch of `new`
        // assigned its result into `instance.view_scripts` (copy-pasted
        // from the branch above it) instead of `instance.stylesheets`,
        // so `.selp` files silently vanished and, depending on glob
        // ordering, could clobber real `.velp` results. Invisible before
        // because this was the only test project with all three file
        // kinds, and this was the only test that checked them by content
        // rather than just by count.
        let project_path = format!("{FIXTURES_PATH}/multi_dialect");
        let config = ElpParserConfig::default();

        let project = ElpParser::new(project_path.into(), config);

        assert_eq!(project.scripts.len(), 1);
        assert_eq!(project.view_scripts.len(), 1);
        assert_eq!(project.stylesheets.len(), 1);

        let script = project.scripts.values().next().unwrap();
        let view_script = project.view_scripts.values().next().unwrap();
        let stylesheet = project.stylesheets.values().next().unwrap();

        // Each file's content is genuinely distinct, so a mixup (any pair
        // ending up identical, or a field ending up empty because another
        // type overwrote it) would fail one of these.
        assert_ne!(script, view_script);
        assert_ne!(view_script, stylesheet);
        assert_ne!(script, stylesheet);
    }

    #[test]
    #[should_panic(expected = "no package at path")]
    fn panics_on_missing_package_path() {
        ElpParser::new(
            format!("{FIXTURES_PATH}/does_not_exist").into(),
            ElpParserConfig::default(),
        );
    }
}
