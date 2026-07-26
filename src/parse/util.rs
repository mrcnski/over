//! Utility functions used by the parser.

use super::BinaryOp;
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{pow, FromPrimitive};
use std::{
    fs::File,
    io::{self, Read},
};

/// If `ch` preceded by a backslash together form an escape character, then return this char.
/// Otherwise, return None.
pub fn get_escape_char(ch: char) -> Option<char> {
    match ch {
        '\\' => Some('\\'),
        '"' => Some('"'),
        '\'' => Some('\''),
        '$' => Some('$'),
        'n' => Some('\n'),
        'r' => Some('\r'),
        't' => Some('\t'),
        _ => None,
    }
}

/// Returns true if this character signifies the legal end of a value.
pub fn is_value_end_char(ch: char) -> bool {
    is_whitespace(ch) || is_end_delimiter(ch) || BinaryOp::is_op(ch)
}

/// Returns true if the character is either whitespace or '#' (start of a comment).
pub fn is_whitespace(ch: char) -> bool {
    ch.is_whitespace() || ch == '#'
}

pub fn is_end_delimiter(ch: char) -> bool {
    match ch {
        ')' | ']' | '}' | '>' => true,
        _ => false,
    }
}

pub fn is_numeric_char(ch: char) -> bool {
    match ch {
        _ch if is_digit(_ch) => true,
        '.' | ',' => true,
        _ => false,
    }
}

/// Returns true if `ch` is an ASCII decimal digit.
pub fn is_digit(ch: char) -> bool {
    match ch {
        '0'..='9' => true,
        _ => false,
    }
}

pub fn is_reserved(field: &str) -> bool {
    match field {
        "@" | "null" | "true" | "false" | "Obj" | "Str" | "Arr" | "Tup" => true,
        _ => false,
    }
}

pub fn frac_from_whole_and_dec(whole: BigInt, decimal: BigInt, dec_len: usize) -> BigRational {
    // whole + decimal/denom == (whole * denom + decimal) / denom, with a single reduction.
    let denom = pow(BigInt::from_u8(10).unwrap(), dec_len);
    BigRational::new(whole * &denom + decimal, denom)
}

/// Reads a file and returns its contents in a string.
pub fn read_file_str(fname: &str) -> io::Result<String> {
    // Open a file in read-only mode.
    let mut file = File::open(fname)?;

    let mut contents = String::new();
    let _ = file.read_to_string(&mut contents)?;

    Ok(contents)
}
