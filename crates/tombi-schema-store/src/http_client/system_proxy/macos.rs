use system_configuration::{
    core_foundation::{
        base::CFType,
        dictionary::CFDictionary,
        number::CFNumber,
        string::{CFString, CFStringRef},
    },
    dynamic_store::SCDynamicStoreBuilder,
    sys::schema_definitions::{
        kSCPropNetProxiesHTTPEnable, kSCPropNetProxiesHTTPPort, kSCPropNetProxiesHTTPProxy,
        kSCPropNetProxiesHTTPSEnable, kSCPropNetProxiesHTTPSPort, kSCPropNetProxiesHTTPSProxy,
    },
};

use super::ProxySettings;

pub(super) fn settings() -> ProxySettings {
    let Some(proxies) = SCDynamicStoreBuilder::new("tombi")
        .build()
        .and_then(|store| store.get_proxies())
    else {
        return ProxySettings::default();
    };

    ProxySettings {
        http: setting(
            &proxies,
            unsafe { kSCPropNetProxiesHTTPEnable },
            unsafe { kSCPropNetProxiesHTTPProxy },
            unsafe { kSCPropNetProxiesHTTPPort },
        ),
        https: setting(
            &proxies,
            unsafe { kSCPropNetProxiesHTTPSEnable },
            unsafe { kSCPropNetProxiesHTTPSProxy },
            unsafe { kSCPropNetProxiesHTTPSPort },
        ),
        ..ProxySettings::default()
    }
}

fn setting(
    proxies: &CFDictionary<CFString, CFType>,
    enabled_key: CFStringRef,
    host_key: CFStringRef,
    port_key: CFStringRef,
) -> String {
    let enabled = proxies
        .find(enabled_key)
        .and_then(|flag| flag.downcast::<CFNumber>())
        .and_then(|flag| flag.to_i32())
        == Some(1);
    if !enabled {
        return String::new();
    }

    let host = proxies
        .find(host_key)
        .and_then(|host| host.downcast::<CFString>())
        .map(|host| host.to_string());
    let port = proxies
        .find(port_key)
        .and_then(|port| port.downcast::<CFNumber>())
        .and_then(|port| port.to_i32());
    match (host, port) {
        (Some(host), Some(port)) => format!("{host}:{port}"),
        (Some(host), None) => host,
        (None, _) => String::new(),
    }
}
