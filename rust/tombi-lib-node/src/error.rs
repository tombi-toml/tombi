use napi::{Env, JsValue};

/// Convert a [`tombi_lib::Error`] into a JS `Error` whose `name` tells the
/// error kind apart, mirroring `tombi-lib-python`'s `TombiError`/
/// `TombiConfigError`/`TombiSchemaError`.
pub(crate) fn to_napi_error(env: &Env, error: tombi_lib::Error) -> napi::Error {
    let name = match error {
        tombi_lib::Error::Io(_) => "TombiError",
        tombi_lib::Error::Config(_) => "TombiConfigError",
        tombi_lib::Error::Schema(_) => "TombiSchemaError",
    };
    let reason = error.to_string();

    let js_error = env
        .create_error(napi::Error::from_reason(reason.clone()))
        .and_then(|mut js_error| {
            js_error.set("name", name)?;
            Ok(js_error)
        });
    match js_error {
        Ok(js_error) => napi::Error::from(js_error.to_unknown()),
        // Still reject with the message if the JS error object can't be built.
        Err(_) => napi::Error::from_reason(reason),
    }
}
