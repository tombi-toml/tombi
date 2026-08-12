# Tombi WebAssembly packages

WebAssembly builds for Tombi's formatter, linter, and Language Server are published
as separate npm packages so applications do not download capabilities they do not use.

The package provides two entry points so applications only load the capabilities they use:

- `@tombi-toml/wasm-lib` contains the formatter and linter.
- `@tombi-toml/wasm-lsp` contains the Language Server and its Web Worker adapter.

Build both variants and run their smoke tests with:

```sh
pnpm --dir rust/tombi-wasm build
pnpm --dir rust/tombi-wasm test
```

The worker accepts standard LSP JSON-RPC messages only. Browser hosts represent their workspace by
opening each file with `textDocument/didOpen`, synchronizing edits with `textDocument/didChange`,
and sending `textDocument/didSave` when appropriate:

```js
worker.postMessage({
  jsonrpc: "2.0",
  method: "textDocument/didOpen",
  params: {
    textDocument: {
      languageId: "toml",
      uri: "file:///workspace/Cargo.toml",
      version: 1,
      text: "[workspace]\nmembers = [\"app\"]\n",
    },
  },
});
```

The server mirrors opened `file:` documents into its in-memory filesystem, allowing extensions to
resolve paths across all opened workspace files. Hosts embedding `serve` directly can use the
exported `set_workspace_entries` JavaScript API before starting the server to preload files that
will remain unopened; this is an embedding API and is not part of the worker protocol.
