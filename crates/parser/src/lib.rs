pub mod ast;
pub mod parser;

use ast::Program;
use from_pest::FromPest;
use parser::{ElpParser, Rule};
use pest::Parser;

pub fn parse_module_source<'a>(
    module_source: &'a str,
) -> Result<Program, Box<dyn std::error::Error + 'a>> {
    let mut parse_tree = ElpParser::parse(Rule::program, module_source)?;

    let ast = Program::from_pest(&mut parse_tree)?;

    Ok(ast)
}
