use napi::{Env, JsTypeError, JsValue};

/// Why a `format`/`lint` call failed, kept until `Task::resolve`, which has
/// the `Env` needed to build the JS error.
pub enum Failure {
    /// Malformed `options`: a caller bug rather than a `TombiError`, so it
    /// rejects with the standard `TypeError`, like the Python/wasm bindings.
    InvalidOptions(String),
    Tombi(tombi_lib::Error),
}

/// Convert a [`Failure`] into the JS error the Promise rejects with: a
/// `TypeError`, or an `Error` named [`tombi_lib::Error::NAME`].
pub(crate) fn to_napi_error(env: Env, failure: Failure) -> napi::Error {
    match failure {
        Failure::InvalidOptions(message) => napi::Error::from(
            JsTypeError::from(napi::Error::from_reason(message)).into_unknown(env),
        ),
        Failure::Tombi(error) => {
            let reason = error.to_string();
            let js_error = env
                .create_error(napi::Error::from_reason(reason.clone()))
                .and_then(|mut js_error| {
                    js_error.set("name", tombi_lib::Error::NAME)?;
                    Ok(js_error)
                });
            match js_error {
                Ok(js_error) => napi::Error::from(js_error.to_unknown()),
                // Still reject with the message if the JS error object can't
                // be built.
                Err(_) => napi::Error::from_reason(reason),
            }
        }
    }
}
