use pest::Span;

use crate::cst::number_value::CSTNumber;

use super::traits::FromCST;

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum ParsedNumber {
    Integer(i64),
    Float(f64),
    Hexadecimal(i64),
    Binary(i64),
    Octal(i64),
}

#[derive(Debug, PartialEq, Clone)]
pub struct ASTNumber<'a> {
    pub span: &'a Span<'a>,
    pub value: ParsedNumber,
}

impl<'a> FromCST<'a, CSTNumber<'a>> for ASTNumber<'a> {
    fn from_cst(cst: &'a CSTNumber) -> Self {
        Self {
            span: &cst.span,
            value: ParsedNumber::from_string(cst.value.as_str()),
        }
    }
}

impl ParsedNumber {
    pub fn from_string(input: &str) -> Self {
        // Check for prefixes first to determine the base
        if input.starts_with("0x") {
            // Hexadecimal
            let clean_input = input.trim_start_matches("0x").replace("_", "");
            if let Ok(value) = i64::from_str_radix(&clean_input, 16) {
                ParsedNumber::Hexadecimal(value)
            } else {
                panic!("Invalid hexadecimal number: {}", input)
            }
        } else if input.starts_with("0b") {
            // Binary
            let clean_input = input.trim_start_matches("0b").replace("_", "");
            if let Ok(value) = i64::from_str_radix(&clean_input, 2) {
                ParsedNumber::Binary(value)
            } else {
                panic!("Invalid binary number: {}", input)
            }
        } else if input.starts_with("0o") {
            // Octal
            let clean_input = input.trim_start_matches("0o").replace("_", "");
            if let Ok(value) = i64::from_str_radix(&clean_input, 8) {
                ParsedNumber::Octal(value)
            } else {
                panic!("Invalid octal number: {}", input)
            }
        } else if input.contains('.') {
            // Float (must contain a decimal point)
            let clean_input = input.replace("_", "");
            if let Ok(value) = clean_input.parse::<f64>() {
                ParsedNumber::Float(value)
            } else {
                panic!("Invalid floating-point number: {}", input)
            }
        } else {
            // Integer (no prefix, no decimal point)
            let clean_input = input.replace("_", "");
            if let Ok(value) = clean_input.parse::<i64>() {
                ParsedNumber::Integer(value)
            } else {
                panic!("Invalid integer: {}", input)
            }
        }
    }
}
