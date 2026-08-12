# tombi-wasm

WebAssembly bindings for Tombi's formatter, linter, and Language Server.

This crate is built with [`wasm-bindgen`](https://github.com/rustwasm/wasm-bindgen)
and exposes JavaScript-compatible APIs for browser environments. The functionality
is selected with Cargo features:

- `lib` (default) provides the formatter and linter APIs.
- `lsp` provides the Language Server runtime and workspace file APIs.

The `lib` and `lsp` features are mutually exclusive build variants. Build this
crate with `wasm-pack` and select the required feature explicitly:

```sh
wasm-pack build \
  --target web \
  --no-default-features \
  --features lib
```

To build the Language Server variant:

```sh
wasm-pack build \
  --target web \
  --no-default-features \
  --features lsp
```

The Language Server communicates through standard LSP JSON-RPC messages. Browser
hosts can use the exported `serve` API with Web Streams and use
`set_workspace_entries` to preload files that are not opened through LSP.
