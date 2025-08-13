# Url

This crate is a utility library for cross-platform `Url` compatibility.

In (non-WASI) WASM environments, methods such as `url::Url::from_file_path`, which handle `file://` URLs, are not available.  
This crate provides functions that simply fallback to `Err(())` in such scenarios, improving compatibility within WASM environments.
