use std::{error::Error, fmt::Display};

use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "elp.pest"]
pub struct ElpParser;

#[derive(Debug, PartialEq, Eq)]
pub enum ElpParseError<'a> {
    Unknown,
    UnexpectedToken {
        msg: &'a str,
        pair: pest::iterators::Pair<'a, Rule>,
    },
    ExpectedButGot {
        msg: &'a str,
        expected: String,
        found: pest::iterators::Pair<'a, Rule>,
    },
}

impl Display for ElpParseError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ElpParseError::Unknown => write!(f, "Unknown error"),
            ElpParseError::UnexpectedToken { msg, pair } => {
                write!(f, "Unexpected token: {} at {:?}", msg, pair)
            }
            ElpParseError::ExpectedButGot {
                msg,
                expected,
                found,
            } => write!(f, "{}: {} expected, found {:?}", msg, expected, found),
        }
    }
}

impl Error for ElpParseError<'_> {}
