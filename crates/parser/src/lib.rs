pub mod ast;
pub mod parser;

use ast::Module;
use from_pest::FromPest;
use parser::{ElpParser, Rule};
use pest::Parser;

pub fn parse_module_source<'a>(
    module_source: &'a str,
) -> Result<Module, Box<dyn std::error::Error + 'a>> {
    let mut parse_tree = ElpParser::parse(Rule::module, module_source)?;

    let ast = Module::from_pest(&mut parse_tree)?;

    Ok(ast)
}
