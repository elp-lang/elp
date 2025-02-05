#![allow(dead_code)]
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "elp.pest"]
pub struct ElpParser;
