pub fn validate_format(value: &str) -> bool {
    // Local date-time: YYYY-MM-DDThh:mm:ss[.frac] (no timezone)
    let sep_pos = value.bytes().position(|b| matches!(b, b'T' | b't' | b' '));
    let Some(sep_pos) = sep_pos else {
        return false;
    };

    let date_part = &value[..sep_pos];
    let time_part = &value[sep_pos + 1..];

    super::date::validate_format(date_part) && super::local_time::validate_format(time_part)
}
