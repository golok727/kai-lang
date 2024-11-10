pub mod error;
pub mod lexer;
pub mod token;

pub fn parse_int_value(value: &str) -> Option<i32> {
    let (radix, value) = if let Some(value) = value.strip_prefix("0x") {
        (16, value)
    } else if let Some(value) = value.strip_prefix("0o") {
        (8, value)
    } else if let Some(value) = value.strip_prefix("0b") {
        (2, value)
    } else {
        (10, value)
    };

    i32::from_str_radix(value, radix).ok()
}
