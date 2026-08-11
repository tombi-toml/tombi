# tombi-wasm

WebAssembly builds for Tombi's formatter, linter, and language server.

The package provides two entry points so applications only load the capabilities they use:

- `tombi-wasm/formatter` contains the formatter and linter.
- `tombi-wasm/lsp` contains the language server as well as the direct formatter and linter APIs.
- `tombi-wasm/lsp/worker` runs the language server in a Web Worker and exchanges JSON-RPC objects through `postMessage`.

Build both variants and run their smoke tests with:

```sh
pnpm build
pnpm test
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
