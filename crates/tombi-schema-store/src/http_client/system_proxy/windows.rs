use super::{ProxySettings, parse_windows_proxy_server, windows_proxy_override_to_no_proxy};

pub(super) fn settings() -> ProxySettings {
    let Ok(key) = windows_registry::CURRENT_USER
        .open("Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings")
    else {
        return ProxySettings::default();
    };
    if key.get_u32("ProxyEnable").unwrap_or(0) == 0 {
        return ProxySettings::default();
    }

    let (http, https) = key
        .get_string("ProxyServer")
        .map(|value| parse_windows_proxy_server(&value))
        .unwrap_or_default();
    let no = key
        .get_string("ProxyOverride")
        .map(|value| windows_proxy_override_to_no_proxy(&value))
        .unwrap_or_default();

    ProxySettings {
        http,
        https,
        no,
        ..ProxySettings::default()
    }
}
