#[allow(clippy::result_unit_err)]
pub fn try_from_boolean(value: &str) -> Result<bool, ()> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(()),
    }
}
