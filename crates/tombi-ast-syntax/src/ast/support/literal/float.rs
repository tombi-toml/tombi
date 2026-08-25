#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ParseError {
    #[error(transparent)]
    Std(#[from] std::num::ParseFloatError),
    #[error("Both sides of the underscore must be numbers")]
    Underscore,
    #[error("Leading zeros are not allowed")]
    LeadingZero,
}

pub fn try_from_float(value: &str) -> Result<f64, self::ParseError> {
    if value.chars().enumerate().any(|(i, c)| {
        if c == '_' {
            match (value.chars().nth(i - 1), value.chars().nth(i + 1)) {
                (Some(digit1), Some(digit2)) => {
                    !digit1.is_ascii_digit() || !digit2.is_ascii_digit()
                }
                (None, _) | (_, None) => true,
            }
        } else {
            false
        }
    }) {
        return Err(self::ParseError::Underscore);
    }

    let int_slice = if value.contains('.') {
        value.split('.').next().unwrap()
    } else {
        value.split('e').next().unwrap()
    };

    let int_number = if int_slice.starts_with("+") || int_slice.starts_with("-") {
        &int_slice[1..]
    } else {
        int_slice
    };

    if int_number.len() > 1 && int_number.starts_with('0') {
        return Err(self::ParseError::LeadingZero);
    }

    Ok(value.replace("_", "").parse()?)
}
