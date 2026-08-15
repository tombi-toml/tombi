import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

import init, {
  ServerConfig,
  serve,
  set_workspace_file,
  set_workspace_files,
} from "../../../typescript/@tombi-toml/wasm-lsp/dist/tombi_lsp_wasm.js";

class AsyncByteQueue {
  #closed = false;
  #items = [];
  #waiters = [];

  push(item) {
    const waiter = this.#waiters.shift();
    if (waiter) {
      waiter({ done: false, value: item });
    } else {
      this.#items.push(item);
    }
  }

  close() {
    this.#closed = true;
    for (const waiter of this.#waiters.splice(0)) {
      waiter({ done: true, value: undefined });
    }
  }

  next() {
    const item = this.#items.shift();
    if (item) return Promise.resolve({ done: false, value: item });
    if (this.#closed) return Promise.resolve({ done: true, value: undefined });
    return new Promise((resolve) => this.#waiters.push(resolve));
  }

  [Symbol.asyncIterator]() {
    return this;
  }
}

const encoder = new TextEncoder();
const decoder = new TextDecoder();

function encode(message) {
  const body = JSON.stringify(message);
  return encoder.encode(`Content-Length: ${encoder.encode(body).byteLength}\r\n\r\n${body}`);
}

const wasm = await readFile(new URL("../../../typescript/@tombi-toml/wasm-lsp/dist/tombi_lsp_wasm_bg.wasm", import.meta.url));
await init({ module_or_path: wasm });

set_workspace_files([
  {
    uri: "file:///workspace/tombi.toml",
    text: `toml-version = "v1.1.0"

[format.rules]

[extensions]
"tombi-toml/cargo" = { lsp.document-link.path.enabled = true }
"tombi-toml/pyproject" = { lsp.document-link.pyproject-toml.enabled = true }
`,
  },
  {
    uri: "file:///workspace/local/Cargo.toml",
    text: `[package]
name = "local"
version = "0.1.0"
`,
  },
  {
    uri: "file:///workspace/member/pyproject.toml",
    text: `[project]
name = "member"
version = "0.1.0"
`,
  },
  {
    uri: "file:///workspace/Cargo.toml",
    text: `[dependencies]
local = { path = "local" }
`,
  },
  {
    uri: "file:///workspace/pyproject.toml",
    text: `[project]
dependencies = ["member"]

[tool.uv.workspace]
members = ["member"]

[tool.uv.sources]
member = { workspace = true }
`,
  },
]);

const input = new AsyncByteQueue();
let outputBuffer = new Uint8Array();
const messages = [];
const outputWaiters = [];

function waitForMessages(count) {
  if (messages.length >= count) return Promise.resolve();
  return new Promise((resolve, reject) => {
    const timeout = setTimeout(
      () => reject(new Error(`LSP response ${count} timed out`)),
      5_000,
    );
    outputWaiters.push({
      count,
      resolve() {
        clearTimeout(timeout);
        resolve();
      },
    });
  });
}

async function waitForMessage(predicate) {
  const timeoutAt = Date.now() + 5_000;
  while (Date.now() < timeoutAt) {
    const message = messages.find(predicate);
    if (message) return message;
    await new Promise((resolve) => setTimeout(resolve, 10));
  }
  throw new Error("LSP message timed out");
}

function waitForResponse(id) {
  return waitForMessage((message) => message.id === id);
}

const output = new WritableStream({
  write(chunk) {
    const next = new Uint8Array(outputBuffer.byteLength + chunk.byteLength);
    next.set(outputBuffer);
    next.set(chunk, outputBuffer.byteLength);
    outputBuffer = next;

    while (true) {
      const separator = decoder.decode(outputBuffer).indexOf("\r\n\r\n");
      if (separator < 0) break;
      const header = decoder.decode(outputBuffer.slice(0, separator));
      const length = Number.parseInt(header.match(/Content-Length:\s*(\d+)/i)?.[1] ?? "", 10);
      const bodyStart = separator + 4;
      if (!Number.isFinite(length) || outputBuffer.byteLength < bodyStart + length) break;
      const message = JSON.parse(decoder.decode(outputBuffer.slice(bodyStart, bodyStart + length)));
      messages.push(message);
      if (message.id !== undefined && message.method) {
        input.push(encode({ jsonrpc: "2.0", id: message.id, result: null }));
      }
      outputBuffer = outputBuffer.slice(bodyStart + length);
      for (const waiter of outputWaiters.splice(0)) {
        if (messages.length >= waiter.count) waiter.resolve();
        else outputWaiters.push(waiter);
      }
    }
  },
});

const server = serve(new ServerConfig(input, output));
input.push(
  encode({
    jsonrpc: "2.0",
    id: 1,
    method: "initialize",
    params: { capabilities: {}, processId: null, rootUri: null },
  }),
);

await waitForMessages(1);

assert.equal(messages[0]?.id, 1);
assert.equal(messages[0]?.error, undefined);
assert.equal(messages[0]?.result?.serverInfo?.name, "Tombi LSP");
assert.equal(messages[0]?.result?.capabilities?.hoverProvider, true);
assert.equal(messages[0]?.result?.capabilities?.definitionProvider, true);
assert.equal(messages[0]?.result?.capabilities?.referencesProvider, true);
assert.ok(messages[0]?.result?.capabilities?.completionProvider);
assert.ok(messages[0]?.result?.capabilities?.inlayHintProvider);
assert.ok(messages[0]?.result?.capabilities?.codeActionProvider);
assert.ok(messages[0]?.result?.capabilities?.semanticTokensProvider);

input.push(encode({ jsonrpc: "2.0", method: "initialized", params: {} }));
for (const textDocument of [
  {
    languageId: "toml",
    uri: "file:///workspace/tombi.toml",
    version: 1,
    text: `toml-version = "v1.1.0"

[format.rules]

[extensions]
"tombi-toml/cargo" = { lsp.document-link.path.enabled = true }
"tombi-toml/pyproject" = { lsp.document-link.pyproject-toml.enabled = true }
`,
  },
  {
    languageId: "toml",
    uri: "file:///workspace/local/Cargo.toml",
    version: 1,
    text: `[package]
name = "local"
version = "0.1.0"
`,
  },
  {
    languageId: "toml",
    uri: "file:///workspace/member/pyproject.toml",
    version: 1,
    text: `[project]
name = "member"
version = "0.1.0"
`,
  },
]) {
  input.push(
    encode({
      jsonrpc: "2.0",
      method: "textDocument/didOpen",
      params: { textDocument },
    }),
  );
}
await waitForMessage(
  (message) =>
    message.method === "textDocument/publishDiagnostics" &&
    message.params?.uri === "file:///workspace/member/pyproject.toml",
);
input.push(
  encode({
    jsonrpc: "2.0",
    method: "textDocument/didOpen",
    params: {
      textDocument: {
        languageId: "toml",
        text: "key =",
        uri: "inmemory://model/playground.toml",
        version: 1,
      },
    },
  }),
);
await waitForMessage(
  (message) =>
    message.method === "textDocument/publishDiagnostics" &&
    message.params?.uri === "inmemory://model/playground.toml",
);
input.push(
  encode({
    jsonrpc: "2.0",
    id: 2,
    method: "textDocument/diagnostic",
    params: { textDocument: { uri: "inmemory://model/playground.toml" } },
  }),
);

const diagnosticResponse = await waitForResponse(2);
assert.equal(diagnosticResponse?.error, undefined);
assert.ok(diagnosticResponse?.result?.items?.length > 0);

input.push(
  encode({
    jsonrpc: "2.0",
    method: "textDocument/didChange",
    params: {
      contentChanges: [{ text: "key={nested=1}" }],
      textDocument: { uri: "inmemory://model/playground.toml", version: 2 },
    },
  }),
);
await waitForMessage(
  (message) =>
    message.method === "textDocument/publishDiagnostics" &&
    message.params?.uri === "inmemory://model/playground.toml" &&
    message.params?.version === 2,
);
input.push(
  encode({
    jsonrpc: "2.0",
    id: 3,
    method: "textDocument/formatting",
    params: {
      options: { insertSpaces: true, tabSize: 2 },
      textDocument: { uri: "inmemory://model/playground.toml" },
    },
  }),
);

const formattingResponse = await waitForResponse(3);
assert.equal(formattingResponse?.error, undefined);
assert.match(formattingResponse?.result?.[0]?.newText ?? "", /\{ nested = 1 \}/);

input.push(
  encode({
    jsonrpc: "2.0",
    method: "textDocument/didOpen",
    params: {
      textDocument: {
        languageId: "toml",
        text: `[dependencies]
local = { path = "local" }
`,
        uri: "file:///workspace/Cargo.toml",
        version: 1,
      },
    },
  }),
);
await waitForMessage(
  (message) =>
    message.method === "textDocument/publishDiagnostics" &&
    message.params?.uri === "file:///workspace/Cargo.toml",
);
input.push(
  encode({
    jsonrpc: "2.0",
    id: 4,
    method: "textDocument/documentLink",
    params: { textDocument: { uri: "file:///workspace/Cargo.toml" } },
  }),
);

const documentLinkResponse = await waitForResponse(4);
assert.equal(documentLinkResponse?.error, undefined);
assert.ok(
  documentLinkResponse?.result?.some(
    (link) => link.target?.startsWith("file:///workspace/local/Cargo.toml"),
  ),
  `Cargo extension should resolve a local dependency through the virtual filesystem: ${JSON.stringify(documentLinkResponse)}`,
);

input.push(
  encode({
    jsonrpc: "2.0",
    method: "textDocument/didOpen",
    params: {
      textDocument: {
        languageId: "toml",
        text: `[project]
dependencies = ["member"]

[tool.uv.workspace]
members = ["member"]

[tool.uv.sources]
member = { workspace = true }
`,
        uri: "file:///workspace/pyproject.toml",
        version: 1,
      },
    },
  }),
);
await waitForMessage(
  (message) =>
    message.method === "textDocument/publishDiagnostics" &&
    message.params?.uri === "file:///workspace/pyproject.toml",
);
input.push(
  encode({
    jsonrpc: "2.0",
    id: 5,
    method: "textDocument/documentLink",
    params: { textDocument: { uri: "file:///workspace/pyproject.toml" } },
  }),
);

const pyprojectLinkResponse = await waitForResponse(5);
assert.equal(pyprojectLinkResponse?.error, undefined);
assert.ok(
  pyprojectLinkResponse?.result?.some((link) =>
    link.target?.startsWith("file:///workspace/member/pyproject.toml"),
  ),
  `Pyproject extension should resolve a workspace member through the virtual filesystem: ${JSON.stringify(pyprojectLinkResponse)}`,
);

input.push(
  encode({
    jsonrpc: "2.0",
    id: 6,
    method: "textDocument/hover",
    params: {
      position: { line: 2, character: 5 },
      textDocument: { uri: "file:///workspace/tombi.toml" },
    },
  }),
);

const hoverResponse = await waitForResponse(6);
assert.equal(hoverResponse?.error, undefined);
assert.ok(hoverResponse?.result, `WASM LSP hover should return content: ${JSON.stringify(hoverResponse)}`);

input.push(
  encode({
    jsonrpc: "2.0",
    id: 7,
    method: "textDocument/completion",
    params: {
      context: { triggerKind: 1 },
      position: { line: 3, character: 0 },
      textDocument: { uri: "file:///workspace/tombi.toml" },
    },
  }),
);

const completionResponse = await waitForResponse(7);
assert.equal(completionResponse?.error, undefined);
assert.ok(
  (completionResponse?.result?.items ?? completionResponse?.result ?? []).length > 0,
  `WASM LSP completion should return candidates: ${JSON.stringify(completionResponse)}`,
);

const updatedConfig = `toml-version = "v1.1.0"

[format.rules]
group-blank-lines-limit = 5



line-width = 100
`;
set_workspace_file("file:///workspace/tombi.toml", updatedConfig);
input.push(
  encode({
    jsonrpc: "2.0",
    method: "textDocument/didChange",
    params: {
      contentChanges: [{ text: updatedConfig }],
      textDocument: { uri: "file:///workspace/tombi.toml", version: 2 },
    },
  }),
);
input.push(
  encode({
    jsonrpc: "2.0",
    id: 8,
    method: "tombi/updateConfig",
    params: { uri: "file:///workspace/tombi.toml" },
  }),
);

const updateConfigResponse = await waitForResponse(8);
assert.equal(updateConfigResponse?.error, undefined);
assert.equal(updateConfigResponse?.result, true);

input.push(
  encode({
    jsonrpc: "2.0",
    id: 9,
    method: "textDocument/formatting",
    params: {
      options: { insertSpaces: true, tabSize: 2 },
      textDocument: { uri: "file:///workspace/tombi.toml" },
    },
  }),
);

const configFormattingResponse = await waitForResponse(9);
assert.equal(configFormattingResponse?.error, undefined);
assert.equal(
  configFormattingResponse?.result,
  null,
  "formatting tombi.toml should use its reloaded group-blank-lines-limit",
);

input.close();
await server;
