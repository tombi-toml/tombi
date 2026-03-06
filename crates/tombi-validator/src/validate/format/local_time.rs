pub fn validate_format(value: &str) -> bool {
    // Local time: hh:mm:ss[.frac] (no timezone)
    let Some(rest) = super::time::parse_time_components(value) else {
        return false;
    };

    // Must have no timezone suffix
    rest.is_empty()
}
