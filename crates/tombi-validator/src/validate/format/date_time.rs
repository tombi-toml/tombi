pub fn validate_format(value: &str) -> bool {
    // RFC 3339 date-time: YYYY-MM-DDThh:mm:ss[.frac]Z or YYYY-MM-DDThh:mm:ss[.frac]+hh:mm
    parse_date_time(value)
}

fn parse_date_time(value: &str) -> bool {
    // Must contain 'T' or 't' separator (or space per RFC 3339)
    let sep_pos = value.bytes().position(|b| matches!(b, b'T' | b't' | b' '));
    let Some(sep_pos) = sep_pos else {
        return false;
    };

    let date_part = &value[..sep_pos];
    let time_part = &value[sep_pos + 1..];

    super::date::validate_format(date_part) && super::time::validate_format(time_part)
}
