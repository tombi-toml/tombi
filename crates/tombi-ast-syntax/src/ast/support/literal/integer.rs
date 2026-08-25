pub use std::num::ParseIntError;

pub fn try_from_binary(value: &str) -> Result<i64, ParseIntError> {
    i64::from_str_radix(&value[2..].replace('_', ""), 2)
}

pub fn try_from_octal(value: &str) -> Result<i64, ParseIntError> {
    i64::from_str_radix(&value[2..].replace('_', ""), 8)
}

pub fn try_from_decimal(value: &str) -> Result<i64, ParseIntError> {
    value.replace('_', "").parse::<i64>()
}

pub fn try_from_hexadecimal(value: &str) -> Result<i64, ParseIntError> {
    i64::from_str_radix(&value[2..].replace('_', ""), 16)
}
