pub fn today_offset_date_time() -> String {
    let now = jiff::Zoned::now();
    let today = now
        .time_zone()
        .to_ambiguous_zoned(now.date().at(0, 0, 0, 0))
        .unambiguous()
        .unwrap_or(now);
    today.strftime("%Y-%m-%dT%H:%M:%S%.3f%:z").to_string()
}

pub fn today_local_date_time() -> String {
    let now = jiff::Zoned::now();
    let today = now
        .time_zone()
        .to_ambiguous_zoned(now.date().at(0, 0, 0, 0))
        .unambiguous()
        .unwrap_or(now);
    today.strftime("%Y-%m-%dT%H:%M:%S%.3f").to_string()
}

pub fn today_local_date() -> String {
    jiff::Zoned::now().strftime("%Y-%m-%d").to_string()
}

pub fn today_local_time() -> String {
    let now = jiff::Zoned::now();
    let today = now
        .time_zone()
        .to_ambiguous_zoned(now.date().at(0, 0, 0, 0))
        .unambiguous()
        .unwrap_or(now);
    today.strftime("%H:%M:%S%.3f").to_string()
}
